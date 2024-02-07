fn build_mount_command(
    ip: String,
    hostport: u16,
    mount_path: String,
    writable: bool,
    sudo: bool,
) -> Command {
    let mut ret = if sudo {
        let mut sudocmd = Command::new("sudo");
        sudocmd.arg("mount.nfs");
        sudocmd
    } else {
        Command::new("mount.nfs")
    };
    if writable {
        ret.arg("-o")
        .arg(format!(
            "user,noacl,nolock,vers=3,tcp,wsize=1048576,rsize=131072,actimeo=120,port={hostport},mountport={hostport}"
        ));
    } else {
        ret.arg("-o").arg(format!(
            "user,noacl,nolock,vers=3,tcp,rsize=131072,actimeo=120,port={hostport},mountport={hostport}"
        ));
    }
    ret.arg(format!("{}:/", &ip)).arg(mount_path);
    ret
}
