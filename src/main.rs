use libudev::{Context, Enumerator};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::exit;
struct TouchPad {
    path: PathBuf,
}
impl TouchPad {
    pub fn new() -> Self {
        let path = match Self::find_touchpad_sysfs() {
            Some(p) => p,
            None => {
                eprintln!("未找到触摸板");
                exit(1)
            }
        };
        let new_path = path.join("device/inhibited");
        Self { path: new_path }
    }
    fn find_touchpad_sysfs() -> Option<PathBuf> {
        let ctx = Context::new().ok()?;
        let mut en = Enumerator::new(&ctx).ok()?;

        // 只列出 input 子系统的设备
        en.match_subsystem("input").ok()?;

        for dev in en.scan_devices().ok()? {
            // 我们只关心有事件节点的输入设备
            let devnode = dev.syspath()?;
            if !devnode.file_name()?.to_str()?.starts_with("event") {
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
    fn status(&self) -> bool {
        let tpd_path = self.path.clone();
        let result = std::fs::read_to_string(tpd_path);
        let s = match result {
            Ok(f) => f,
            Err(_) => {
                println!("文件读取错误");
                exit(1)
            }
        };
        let parse = s.trim().parse::<u32>();
        let flag = match parse {
            Ok(f) => f,
            Err(_) => {
                println!("读取标志位错误");
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
        let new = if self.status() { "0\n" } else { "1\n" };
        std::fs::write(&self.path, new).unwrap_or_else(|e| {
            eprintln!("write {}: {}", self.path.display(), e);
            std::process::exit(1);
        });
    }
}

fn main() {
    let tpd = TouchPad::new();
    tpd.toggle();
}
