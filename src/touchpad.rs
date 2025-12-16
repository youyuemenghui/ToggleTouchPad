use crate::perm_guard;
use libudev::{Context, Enumerator};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::exit;

pub struct TouchPad {
    uid: u32,
    gid: u32,
    path: PathBuf,
}
impl TouchPad {
    // tools function
    fn find_touchpad_sysfs() -> Option<PathBuf> {
        //创建udev上下文,没数据
        let udev_ctx = Context::new().ok()?;
        //创建枚举上下文,放入上下文中
        let mut udev_enum = Enumerator::new(&udev_ctx).ok()?;

        // 只列出 input 子系统的设备
        udev_enum.match_subsystem("input").ok()?;

        for dev in udev_enum.scan_devices().ok()? {
            // 我们只关心有事件节点的输入设备
            let dev_path = dev.syspath()?;
            // 跳过路径中不是以事件开头的设备
            if !dev_path.file_name()?.to_str()?.starts_with("event") {
                continue;
            }
            // 读 udev 属性 ID_INPUT_TOUCHPAD
            if dev
                .property_value(OsStr::new("ID_INPUT_TOUCHPAD"))
                .or(Some(OsStr::new("0")))?
                == OsStr::new("1")
            {
                // sysfs 路径就是 /sys + syspath
                return Some(PathBuf::from("/sys").join(dev.syspath().unwrap()));
            }
        }
        None
    }
    pub fn new(uid: u32, gid: u32) -> Self {
        let dev_path = match Self::find_touchpad_sysfs() {
            Some(p) => p,
            None => {
                eprintln!("ERROR: 未找到触摸板");
                exit(1)
            }
        };
        let inhid_file_path = dev_path.join("device/inhibited");
        Self {
            uid,
            gid,
            path: inhid_file_path,
        }
    }
    pub fn status(&self) -> bool {
        let tpd_inhid_file = self.path.clone();
        let s = std::fs::read_to_string(tpd_inhid_file).unwrap_or_else(|_| {
            eprintln!("ERROR: 文件读取错误");
            exit(1)
        });
        let parse = s.trim().parse::<u32>();
        let flag = match parse {
            Ok(f) => f,
            Err(_) => {
                eprintln!("ERROR: 读取标志位错误");
                exit(1)
            }
        };
        match flag {
            0 => false,
            1 => true,
            _ => panic!("不可恢复错误"),
        }
    }
    pub fn toggle(&self) {
        let enable = if self.status() { "0\n" } else { "1\n" };
        std::fs::write(&self.path, enable).unwrap_or_else(|e| {
            eprintln!("write {}: {}", self.path.display(), e);
            exit(1);
        });
    }
    pub fn send_notify(&self) {
        let tpd_status = self.status();
        if tpd_status {
            {
                let _priv_guard = perm_guard::PrivDropGuard::to_user(self.uid, self.gid).unwrap();
                notify::sudo_send("触摸板已关闭", 3000).unwrap_or_else(|e| eprintln!("{}", e));
            }
        } else {
            {
                let _priv_guard = perm_guard::PrivDropGuard::to_user(self.uid, self.gid).unwrap();
                notify::sudo_send("触摸板已开启", 3000).unwrap_or_else(|e| eprintln!("{}", e));
            }
        }
    }
}
pub mod error {
    use std::fmt;
    #[derive(Debug)]
    pub enum TouchpadError {
        Io(std::io::Error),
        Sysfs(&'static str),
        Notify(&'static str),
        PrivDrop(&'static str),
    }
    impl fmt::Display for TouchpadError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(e) => write!(f, "IO 错误: {}", e),
                Self::Sysfs(s) => write!(f, "sysfs 错误: {}", s),
                Self::Notify(s) => write!(f, "通知错误: {}", s),
                Self::PrivDrop(s) => write!(f, "权限切换错误: {}", s),
            }
        }
    }
    impl std::error::Error for TouchpadError {}
    impl From<std::io::Error> for TouchpadError {
        fn from(e: std::io::Error) -> Self {
            Self::Io(e)
        }
    }
}
pub mod notify {
    use super::error::TouchpadError;
    use std::process::Command;

    pub fn sudo_send(msg: &str, ms: u32) -> Result<(), TouchpadError> {
        let user = std::env::var("SUDO_USER").unwrap_or_else(|_| "root".into());
        let uid = std::env::var("SUDO_UID").unwrap_or_else(|_| "1000".into());
        let disp = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".into());
        let dbus = format!("/run/user/{}/bus", uid);

        let st = Command::new("sudo")
            .arg("-u")
            .arg(&user)
            .arg("env")
            .arg(format!("DISPLAY={}", disp))
            .arg(format!("DBUS_SESSION_BUS_ADDRESS=unix:path={}", dbus))
            .arg("notify-send")
            .args(["-a", "TouchPad"])
            .arg(msg)
            .args(["-t", &ms.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|_| TouchpadError::Notify("启动 notify-send 失败"))?;

        if !st.success() {
            return Err(TouchpadError::Notify("notify-send 返回非零"));
        }
        Ok(())
    }
    pub fn pkexec_send(msg: &str, ms: u32) -> Result<(), TouchpadError> {
        let st = Command::new("notify-send")
            .args(["-a", "TouchPad"])
            .arg(msg)
            .args(["-t", &ms.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|_| TouchpadError::Notify("启动 notify-send 失败"))?;

        if !st.success() {
            return Err(TouchpadError::Notify("notify-send 返回非零"));
        }
        Ok(())
    }
}
