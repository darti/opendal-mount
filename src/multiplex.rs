use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bimap::BiMap;
use log::{debug, error, info, warn};
use nfsserve::{
    nfs::{fattr3, fileid3, filename3, ftype3, nfspath3, nfsstat3, nfstime3, sattr3, specdata3},
    vfs::{NFSFileSystem, ReadDirResult, VFSCapabilities},
};

use opendal::Operator;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    errors::OpendalMountResult,
    mount::{FsMounter, Mounter},
    schema::MountedFs,
};

struct MountedOperator {
    mount_point: String,
    op: Operator,
}

#[derive(Clone)]
pub struct MultiplexedFs {
    ip: String,
    port: u16,
    ops: Arc<RwLock<HashMap<String, MountedOperator>>>,
    inodes: Arc<RwLock<BiMap<u64, String>>>,
}

impl MultiplexedFs {
    pub fn new(ip: &str, port: u16) -> Self {
        let mut inodes = BiMap::new();
        inodes.insert(1, "/".to_string());

        Self {
            ip: ip.to_owned(),
            port,
            ops: Arc::new(RwLock::new(HashMap::new())),
            inodes: Arc::new(RwLock::new(inodes)),
        }
    }

    async fn inode_to_path(&self, inode: u64) -> Option<String> {
        self.inodes.read().await.get_by_left(&inode).cloned()
    }

    async fn path_to_inode(&self, path: &str, insert: bool) -> Result<u64, nfsstat3> {
        let ino = self.inodes.read().await.get_by_right(path).copied();

        match ino {
            Some(ino) => Ok(ino),
            None if insert => {
                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                let ino = hasher.finish();

                let mut inodes = self.inodes.write().await;
                (*inodes).insert(ino, path.to_owned());

                Ok(ino)
            }
            _ => Err(nfsstat3::NFS3ERR_NOENT),
        }
    }

    async fn path_to_attr(&self, ino: u64, path: &str) -> Result<fattr3, nfsstat3> {
        let meta = self.operator.stat(&path).await.map_err(|e| {
            warn!("unable to get metadata for {:?}: {}", path, e);
            nfsstat3::NFS3ERR_NOENT
        })?;

        let kind = if meta.is_dir() {
            ftype3::NF3DIR
        } else {
            ftype3::NF3REG
        };

        let mtime = if let Some(mtime) = meta.last_modified() {
            nfstime3 {
                seconds: mtime.timestamp() as u32,
                nseconds: 0,
            }
        } else {
            nfstime3::default()
        };

        let mode = if meta.is_dir() { 0o777 } else { 0o755 };

        Ok(fattr3 {
            ftype: kind,
            mode,
            nlink: 0,
            uid: 507,
            gid: 507,
            size: meta.content_length(),
            used: meta.content_length(),
            rdev: specdata3::default(),
            fsid: 0,
            fileid: ino,
            atime: mtime,
            mtime,
            ctime: mtime,
        })
    }

    pub async fn mount_operator(&self, mount_point: &str, op: Operator) -> OpendalMountResult<()> {
        let mut ops = self.ops.write().await;

        let prefix = Uuid::new_v4().to_string();

        info!("Mounting{} at {}", op.info().name(), mount_point);
        ops.insert(
            mount_point.to_owned(),
            MountedOperator {
                mount_point: mount_point.to_owned(),
                op,
            },
        );

        FsMounter::mount(&self.ip, self.port, &prefix, mount_point, true).await?;

        Ok(())
    }

    pub async fn umount(&self, mount_point: &str) -> OpendalMountResult<()> {
        FsMounter::umount(mount_point).await?;

        Ok(())
    }

    pub async fn umount_all(&self) -> OpendalMountResult<()> {
        debug!("Unmounting all operators at {}:{}", self.ip, self.port);

        let ops = self.ops.read().await;

        for (mount_point, _) in ops.iter() {
            match self.umount(mount_point).await {
                Ok(_) => info!("Unmounted {}", mount_point),
                Err(e) => error!("Failed to unmount {}: {}", mount_point, e),
            }
        }

        Ok(())
    }

    pub async fn mounted_operators(&self) -> Vec<MountedFs> {
        let ops = self.ops.read().await;

        ops.iter()
            .map(|(key, op)| {
                let info = op.op.info();

                MountedFs {
                    id: key.to_owned(),
                    mount_point: op.mount_point.to_owned(),
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
            let mtime = nfstime3::default();

            return Ok(fattr3 {
                ftype: ftype3::NF3DIR,
                mode: 0o777,
                nlink: 0,
                uid: 507,
                gid: 507,
                size: 0,
                used: 0,
                rdev: specdata3::default(),
                fsid: 0,
                fileid: 1,
                atime: mtime,
                mtime,
                ctime: mtime,
            });
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
