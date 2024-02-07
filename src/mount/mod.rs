#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::MacosMounter as FsMounter;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

use log::error;
use tokio::process::Command;

pub trait Mounter {
    fn check() -> bool;
    fn mount_command(ip: String, hostport: u16, mount_path: &str, writable: bool) -> Command;

    async fn mount(
        ip: String,
        hostport: u16,
        mount_path: &str,
        writable: bool,
    ) -> Result<(), std::io::Error> {
        let mut cmd = Self::mount_command(ip, hostport, mount_path, writable);
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
}
