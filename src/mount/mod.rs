#[cfg(target_os = "macos")]
mod macos;

use std::path::Path;

#[cfg(target_os = "macos")]
pub use macos::MacosMounter as FsMounter;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

use log::{debug, error};
use tokio::process::Command;

pub trait Mounter {
    fn check() -> bool;
    fn mount_command<P>(
        ip: &str,
        hostport: u16,
        prefix: &str,
        mount_path: P,
        writable: bool,
    ) -> Command
    where
        P: AsRef<Path>;

    fn umount_command<P>(mount_path: P) -> Command
    where
        P: AsRef<Path>;

    async fn mount<P>(
        ip: &str,
        hostport: u16,
        prefix: &str,
        mount_path: P,
        writable: bool,
    ) -> Result<(), std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut cmd = Self::mount_command(ip, hostport, prefix, mount_path, writable);

        debug!("Mounting with: {:?}", cmd);

        let status = cmd.status().await?;
        if !status.success() {
            error!("Failed to mount: {:?}", status);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to mount",
            ));
        }
        Ok(())
    }

    async fn umount<P>(mount_path: P) -> Result<(), std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut cmd = Self::umount_command(mount_path);

        debug!("Unmounting with: {:?}", cmd);

        let status = cmd.status().await?;
        if !status.success() {
            error!("Failed to umount: {:?}", status);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to mount",
            ));
        }
        Ok(())
    }
}
