pub struct MacosMounter;

use crate::mount::Mounter;
use tokio::process::Command;

impl Mounter for MacosMounter {
    fn check() -> bool {
        true
    }

    fn mount_command(
        ip: &str,
        hostport: u16,
        prefix: &str,
        mount_path: &str,
        writable: bool,
    ) -> Command {
        let mut ret = Command::new("/sbin/mount");
        ret.arg("-t").arg("nfs");
        if writable {
            ret.arg("-o").arg(format!(
                "nolocks,vers=3,tcp,rsize=131072,wsize=1048576,actimeo=120,port={hostport},mountport={hostport}"
            ));
        } else {
            ret.arg("-o").arg(format!(
                "rdonly,nolocks,vers=3,tcp,rsize=131072,actimeo=120,port={hostport},mountport={hostport}"
            ));
        }

        ret.arg(format!("{}:/{}", &ip, prefix)).arg(mount_path);
        ret
    }
}
