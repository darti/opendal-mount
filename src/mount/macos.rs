pub struct MacosMounter;

use std::path::Path;

use crate::mount::Mounter;
use tokio::process::Command;

impl Mounter for MacosMounter {
    fn check() -> bool {
        true
    }

    fn mount_command<P>(
        ip: &str,
        hostport: u16,
        prefix: &str,
        mount_path: P,
        writable: bool,
    ) -> Command
    where
        P: AsRef<Path>,
    {
        let mut ret = Command::new("/sbin/mount");
        ret.arg("-t").arg("nfs");
        if writable {
            ret.arg("-o").arg(format!(
                "nolocks,vers=3,tcp,port={hostport},mountport={hostport},soft"
            ));
        } else {
            ret.arg("-o").arg(format!(
                "rdonly,nolocks,vers=3,tcp,rsize=131072,actimeo=120,port={hostport},mountport={hostport}"
            ));
        }

        ret.arg(format!("{}:/{}", &ip, prefix))
            .arg(mount_path.as_ref().as_os_str());
        ret
    }

    fn umount_command<P>(mount_path: P) -> Command
    where
        P: AsRef<Path>,
    {
        let mut cmd = Command::new("diskutil");
        cmd.arg("umount")
            .arg("force")
            .arg(mount_path.as_ref().as_os_str());

        cmd
    }
}
