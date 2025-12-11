pub mod parse;
pub mod touchpad;
pub mod perm_guard {
    use nix::unistd::{Gid, Uid, getegid, geteuid, setegid, seteuid, setresgid, setresuid};
    /// After accepting a regular user's UID and GID and saving them to an internal structure,
    /// the euid and egid are changed to reflect the regular user's identity.
    /// When the structure is destroyed, it is restored to the root user.
    /// RAII 守卫：接受普通用户UID与GID,构造时降级，析构时恢复
    pub struct PrivDropGuard {
        prev_uid: Uid,
        prev_gid: Gid,
    }

    impl PrivDropGuard {
        // 临时降级到指定 uid/gid（仅改 euid/egid）
        /// Temporarily downgrade to the specified uid/gid (only modify euid/egid)
        pub fn to_user(uid: u32, gid: u32) -> nix::Result<Self> {
            let prev_uid = geteuid();
            let prev_gid = getegid();

            setegid(Gid::from_raw(gid))?;
            seteuid(Uid::from_raw(uid))?;

            eprintln!("[PrivDropGuard] 降级 -> UID {} GID {}", uid, gid);
            Ok(Self { prev_uid, prev_gid })
        }
    }

    impl Drop for PrivDropGuard {
        // 恢复到root用户的uid/gid（仅改 euid/egid）
        /// Restore to the root user's uid/gid (only modify euid/egid)
        fn drop(&mut self) {
            let _ = seteuid(self.prev_uid);
            let _ = setegid(self.prev_gid);
            eprintln!(
                "[PrivDropGuard] 恢复 -> UID {} GID {}",
                self.prev_uid, self.prev_gid
            );
        }
    }

    // 彻底切换到指定用户（ruid=euid=suid）
    /// Completely switch to the specified user (ruid=euid=suid)
    pub fn become_user(uid: u32, gid: u32) -> nix::Result<()> {
        let uid = Uid::from_raw(uid);
        let gid = Gid::from_raw(gid);
        setresgid(gid, gid, gid)?;
        setresuid(uid, uid, uid)?;
        Ok(())
    }
}
pub mod user {

    use users::{Groups, Users, UsersCache, get_user_by_name};

    pub fn get_uid_gid(name: &str) -> Option<(u32, u32)> {
        get_user_by_name(name).map(|u| (u.uid(), u.primary_group_id()))
    }

    pub fn user_exists(name: &str) -> bool {
        get_user_by_name(name).is_some()
    }

    pub fn is_root() -> bool {
        UsersCache::new().get_current_uid() == 0
    }
    pub fn get_current_guid() -> (u32, u32) {
        let current_user = UsersCache::new();
        (
            current_user.get_current_uid(),
            current_user.get_current_gid(),
        )
    }
}
