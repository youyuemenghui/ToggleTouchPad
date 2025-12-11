use crate::user::{get_current_guid, get_uid_gid, user_exists};
use std::{env, process::exit};
pub enum Permissions {
    Rooted { uid: u32, gid: u32 },         //表示使用sudo 获得的root权限
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
                eprintln!("error: 多余的参数");
                exit(2);
            }
            if !user_exists(username) {
                eprintln!("用户没有找到，请确认用户是否存在");
                exit(2);
            }
            let (uid, gid) = get_uid_gid(username).unwrap_or_else(|| {
                eprintln!("error:无法获取用户uid与gid");
                exit(2)
            });
            if is_root {
                Permissions::Rooted { uid, gid }
            } else {
                Permissions::PkexecNeedRoot { uid, gid }
            }
        }
        None if is_root => {
            println!("参数不合法: 如果要使用root权限运行的话,请提供普通用户名");
            exit(2);
        }
        None => {
            let (uid, gid) = get_current_guid();
            Permissions::PkexecNeedRoot { uid, gid }
        }
    }
}

// help需要修改
fn print_help(name: &str) {
    println!("{}", name);
    println!();
    println!("USAGE:");
    println!("    {}  username", name);
    println!();
}
