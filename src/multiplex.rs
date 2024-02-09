use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use log::{debug, info};
use nfsserve::{
    nfs::{fattr3, fileid3, filename3, nfspath3, nfsstat3, sattr3},
    vfs::{NFSFileSystem, ReadDirResult, VFSCapabilities},
};

use opendal::Operator;
use tokio::sync::RwLock;

use crate::{
    errors::{OpendalMountError, OpendalMountResult},
    mount::{FsMounter, Mounter},
    schema::MountedFs,
};

#[derive(Clone)]
pub struct MultiplexedFs {
    ip: String,
    port: u16,
    ops: Arc<RwLock<HashMap<String, Operator>>>,
}

impl MultiplexedFs {
    pub fn new(ip: &str, port: u16) -> Self {
        Self {
            ip: ip.to_owned(),
            port,
            ops: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn mount_operator(&self, mount_point: &str, op: Operator) -> OpendalMountResult<()> {
        let mut ops = self.ops.write().await;

        if ops.contains_key(mount_point) {
            return Err(OpendalMountError::AlreadyMounted(mount_point.to_owned()));
        }

        info!("Mounting{} at {}", op.info().name(), mount_point);
        ops.insert(mount_point.to_owned(), op);

        FsMounter::mount(&self.ip, self.port, mount_point, true).await?;

        Ok(())
    }

    pub async fn umount(&self, mount_point: &str) {
        let mut cmd = Command::new("/sbin/umount");
    }

    pub async fn mounted_operators(&self) -> Vec<MountedFs> {
        let ops = self.ops.read().await;

        ops.iter()
            .map(|(key, op)| {
                let info = op.info();

                MountedFs {
                    mount_point: key.to_owned(),
                    scheme: info.scheme().to_string(),
                    root: info.root().to_owned(),
                    name: info.name().to_owned(),
                }
            })
            .collect()
    }
}

#[async_trait]
impl NFSFileSystem for MultiplexedFs {
    fn capabilities(&self) -> VFSCapabilities {
        VFSCapabilities::ReadWrite
    }

    fn root_dir(&self) -> fileid3 {
        debug!("Root dir requested");

        1
    }

    async fn lookup(&self, parent: fileid3, name: &filename3) -> Result<fileid3, nfsstat3> {
        debug!("Lookup {} in {}", parent, String::from_utf8_lossy(name));

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn getattr(&self, id: fileid3) -> Result<fattr3, nfsstat3> {
        debug!("Getattr {}", id);

        if id == 1 {
            return Ok(fattr3::default());
        }

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn setattr(&self, id: fileid3, setattr: sattr3) -> Result<fattr3, nfsstat3> {
        debug!("Setattr {} with {:?}", id, setattr);

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn read(
        &self,
        id: fileid3,
        offset: u64,
        count: u32,
    ) -> Result<(Vec<u8>, bool), nfsstat3> {
        debug!("Read {} from {} with {} bytes", id, offset, count);

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn write(&self, id: fileid3, offset: u64, data: &[u8]) -> Result<fattr3, nfsstat3> {
        debug!("Write {} from {} with {} bytes", id, offset, data.len());

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn create(
        &self,
        dirid: fileid3,
        filename: &filename3,
        attr: sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!(
            "Create {} in {} with {:?}",
            String::from_utf8_lossy(filename),
            dirid,
            attr
        );

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn create_exclusive(
        &self,
        dirid: fileid3,
        filename: &filename3,
    ) -> Result<fileid3, nfsstat3> {
        debug!(
            "Create exclusive {} in {}",
            String::from_utf8_lossy(filename),
            dirid
        );

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn mkdir(
        &self,
        dirid: fileid3,
        dirname: &filename3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!("Mkdir {} in {}", String::from_utf8_lossy(dirname), dirid);

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn remove(&self, dirid: fileid3, filename: &filename3) -> Result<(), nfsstat3> {
        debug!("Remove {} in {}", String::from_utf8_lossy(filename), dirid);

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn rename(
        &self,
        from_dirid: fileid3,
        from_filename: &filename3,
        to_dirid: fileid3,
        to_filename: &filename3,
    ) -> Result<(), nfsstat3> {
        debug!(
            "Rename {} in {} to {} in {}",
            String::from_utf8_lossy(from_filename),
            from_dirid,
            String::from_utf8_lossy(to_filename),
            to_dirid
        );

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn readdir(
        &self,
        dirid: fileid3,
        start_after: fileid3,
        max_entries: usize,
    ) -> Result<ReadDirResult, nfsstat3> {
        debug!("Readdir {} with {} entries", dirid, max_entries);

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn symlink(
        &self,
        dirid: fileid3,
        linkname: &filename3,
        symlink: &nfspath3,
        attr: &sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!(
            "Symlink {} in {} to {} with {:?}",
            String::from_utf8_lossy(linkname),
            dirid,
            String::from_utf8_lossy(symlink),
            attr
        );

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn readlink(&self, id: fileid3) -> Result<nfspath3, nfsstat3> {
        debug!("Readlink {}", id);

        Err(nfsstat3::NFS3ERR_NOENT)
    }
}
