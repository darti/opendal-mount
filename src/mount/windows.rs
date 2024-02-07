fn build_mount_command(
    ip: String,
    hostport: u16,
    mount_drive: String,
    writable: bool,
) -> Result<Command> {
    debug_assert_eq!(mount_drive.len(), 1);
    debug_assert_eq!(mount_drive, mount_drive.to_uppercase());

    if hostport != 111 {
        return Err(GitXetRepoError::InvalidOperation(
            "NFS mount port must be 111 on windows.".to_owned(),
        ));
    }

    //    let IP = windows_
    let mut ret: Command = Command::new("mount.exe");
    info!(
        "Forming mount command with IP = {:?}, port = {:?}",
        &ip, &hostport
    );

    ret.args([
        "-o",
        &format!(
            // Note: rsize + wsize are in kb.
            "anon,nolock,mtype=soft,fileaccess={},casesensitive,lang=ansi,rsize=128,wsize=128,timeout=60,retry=2",
            if writable { "6" } else { "4" }
        ),
        &format!("\\\\{ip}\\\\"),
        &format!("{}:", &mount_drive),
    ]);

    Ok(ret)
}
