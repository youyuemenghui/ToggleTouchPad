use std::process::{Command, exit};
use toggle_touchpad::parse::{Permissions, parse_args};
use toggle_touchpad::touchpad::TouchPad;
use toggle_touchpad::touchpad::notify;
fn run_privileged_task(uid: u32, gid: u32) {
    // 直接执行特权操作
    println!("切换到root用户");
    let tpd_enable = TouchPad::new(uid, gid);
    tpd_enable.toggle();
    tpd_enable.send_notify();
}
fn main() {
    let perm = parse_args();
    match perm {
        Permissions::Rooted { uid, gid } => {
            run_privileged_task(uid, gid);
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
            println!("已恢复普通用户代码");
            let tpd_enable = TouchPad::new(uid, gid);
            let msg = if tpd_enable.status() {
                "触摸板已关闭"
            } else {
                "触摸板已开启"
            };
            notify::pkexec_send(msg, 3000).unwrap_or_else(|e| eprintln!("{}", e));
            exit(status.code().unwrap_or(1));
        }
        Permissions::PkexecRooted { uid, gid } => {
            println!("切换到root用户");
            let tpd_enable = TouchPad::new(uid, gid);
            tpd_enable.toggle();
        }
    }
}
