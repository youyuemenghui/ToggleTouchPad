use crate::user::{get_current_guid, get_uid_gid, user_exists};
use std::{env, process::exit};
pub enum Permissions {
    SudoRooted { uid: u32, gid: u32 },     //表示使用sudo 获得的root权限
    PkexecNeedRoot { uid: u32, gid: u32 }, //表示需要通过Pkexec 获得的root权限
    PkexecRooted { uid: u32, gid: u32 },   //表示已通过Pkexec 获得的root权限
}
pub fn parse_args() -> Permissions {
    let args: Vec<String> = env::args().collect();
    let is_root = crate::user::is_root();
    match args.get(1).map(|s| s.as_str()) {
        //help 不受权限影响
        Some("--help") | Some("-h") => {
            let prog = args
                .first()
                .map_or_else(|| "toggle_touchpad", |s| s.as_str());
            print_help(prog);
            exit(2);
        }
        Some("-e") | Some("--get-env") => {
            //获取环境变量传进来普通用户U,GID
            let uid = env::var("TPD_UID").unwrap().parse().unwrap();
            let gid = env::var("TPD_GID").unwrap().parse().unwrap();
            Permissions::PkexecRooted { uid, gid }
        }
        // root 情况提供用户名,参数合法,可降权(不缺id,不缺权限)
        Some(username) => {
            if args.len() >= 3 {
                error_exit("ERROR: 多余的参数");
            }
            if !user_exists(username) {
                error_exit("ERROR: 用户没有找到，请确认用户是否存在");
            }
            let (uid, gid) = get_uid_gid(username).unwrap_or_else(|| {
                error_exit("ERROR: 无法获取用户uid与gid");
            });
            if is_root {
                Permissions::SudoRooted { uid, gid }
            } else {
                Permissions::PkexecNeedRoot { uid, gid }
            }
        }
        None if is_root => {
            error_exit("ERROR: 参数不合法,如果要使用root权限运行的话,请提供普通用户名");
        }
        None => {
            let (uid, gid) = get_current_guid();
            Permissions::PkexecNeedRoot { uid, gid }
        }
    }
}
fn error_exit(msg: &str) -> ! {
    eprintln!("{}", msg);
    exit(2);
}
// help需要修改
fn print_help(name: &str) {
    println!("{} - 切换触摸板状态", name);
    println!();
    println!("USAGE:");
    println!("    {} [OPTIONS] [USERNAME]", name);
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       显示帮助信息");
    println!("    -e, --get-env    从环境变量获取用户ID（内部使用）");
    println!();
    println!("ARGS:");
    println!("    USERNAME         指定要降权的普通用户名（root权限运行时）");
    println!();
    println!("说明:");
    println!("    当以普通用户运行时，程序会尝试使用 pkexec 获取root权限。");
    println!("    当以root用户运行时，必须提供要降权的普通用户名。");
    println!();
}
