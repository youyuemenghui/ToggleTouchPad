use std::process::{Command, exit};
use toggle_touchpad::parse::{Permissions, parse_args};
use toggle_touchpad::touchpad::TouchPad;
use toggle_touchpad::touchpad::notify;
fn notify_status(uid: u32, gid: u32) {
    let tpd = TouchPad::new(uid, gid);
    let msg = if tpd.status() {
        "触摸板已关闭"
    } else {
        "触摸板已开启"
    };
    notify::pkexec_send(msg, 3000).unwrap_or_else(|e| eprintln!("{}", e));
}
fn main() {
    let perm = parse_args();
    match perm {
        Permissions::SudoRooted { uid, gid } => {
            println!("INFO: 切换到root用户");
            let tpd = TouchPad::new(uid, gid);
            tpd.toggle();
            tpd.send_notify();
        }
        Permissions::PkexecNeedRoot { uid, gid } => {
            let prog_name = std::env::current_exe().unwrap();
            //使用pkexec尝试升级自身的子进程权限
            let status = Command::new("pkexec")
                .args(["env", format!("TPD_UID={}", uid).as_str()])
                .args(["env", format!("TPD_GID={}", gid).as_str()])
                .arg(&prog_name)
                .arg("-e")
                .status()
                .expect("pkexec 执行失败");

            println!("INFO: 已恢复普通用户代码");
            notify_status(uid, gid);
            exit(status.code().unwrap_or(1));
        }
        Permissions::PkexecRooted { uid, gid } => {
            println!("INFO: 切换到root用户");
            TouchPad::new(uid, gid).toggle();
        }
    }
}
