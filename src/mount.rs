use std::ffi::CString;

use log::{debug, info};

pub fn mount() {}

pub fn umount(mount_point: &str) -> anyhow::Result<()> {
    info!("Unmounting {}", mount_point);

    std::fs::create_dir_all(mount_point)?;
    let mnt = CString::new(mount_point)?.into_raw();

    unsafe {
        match libc::unmount(mnt, libc::MNT_FORCE) {
            libc::EAGAIN => {
                debug!("unmount {}: EAGAIN", mount_point);
            }
            libc::EBUSY => {
                debug!("unmount {}: EBUSY", mount_point);
            }
            libc::EFAULT => {
                debug!("unmount {}: EFAULT", mount_point);
            }
            libc::EINVAL => {
                debug!("unmount {}: EINVAL", mount_point);
            }
            libc::ENAMETOOLONG => {
                debug!("unmount {}: ENAMETOOLONG", mount_point);
            }
            libc::ENOENT => {
                debug!("unmount {}: ENOENT", mount_point);
            }
            libc::ENOMEM => {
                debug!("unmount {}: ENOMEM", mount_point);
            }
            libc::EPERM => {
                debug!("unmount {}: EPERM", mount_point);
            }
            status => {
                debug!("unmount {}: {}", mount_point, status)
            }
        }
    }

    info!("Unmounted {}", mount_point);

    Ok(())
}
