use futures::TryStreamExt;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use bimap::BiMap;
use log::debug;
use nfsserve::{
    nfs::{fattr3, fileid3, filename3, ftype3, nfspath3, nfsstat3, nfstime3, sattr3, specdata3},
    vfs::{DirEntry, NFSFileSystem, ReadDirResult, VFSCapabilities},
};
use opendal::{Entry, Metakey, Operator};
use tokio::sync::RwLock;

pub struct OpendalFs {
    operator: Operator,
    inodes: Arc<RwLock<BiMap<u64, String>>>,
}

impl OpendalFs {
    pub fn new(operator: Operator) -> Self {
        let mut inodes = BiMap::new();
        inodes.insert(1, "/".to_string());

        OpendalFs {
            operator,
            inodes: Arc::new(RwLock::new(inodes)),
        }
    }

    async fn inode_to_path(&self, inode: u64) -> Option<String> {
        self.inodes.read().await.get_by_left(&inode).cloned()
    }

    async fn path_to_inode(&self, path: &str) -> u64 {
        let ino = self.inodes.read().await.get_by_right(path).copied();

        match ino {
            Some(ino) => ino,
            None => {
                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                let ino = hasher.finish();

                let mut inodes = self.inodes.write().await;
                (*inodes).insert(ino, path.to_owned());

                ino
            }
        }
    }

    async fn path_to_attr(&self, path: &str) -> Result<fattr3, nfsstat3> {
        let entry = Entry::new(path);
        // let meta = self.operator.metadata(&entry, Metakey::Mode).await;

        let ino = self.path_to_inode(path).await;

        if let Ok(meta) = self.operator.metadata(&entry, Metakey::Mode).await {
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
        } else {
            Err(nfsstat3::NFS3ERR_NOTSUPP)
        }
    }
}

#[async_trait]
impl NFSFileSystem for OpendalFs {
    fn root_dir(&self) -> fileid3 {
        debug!("root_dir");

        1
    }

    fn capabilities(&self) -> VFSCapabilities {
        debug!("capabilities");
        VFSCapabilities::ReadWrite
    }

    async fn write(&self, id: fileid3, offset: u64, data: &[u8]) -> Result<fattr3, nfsstat3> {
        debug!("write {:?} {:?} {:?}", id, offset, data);
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }

    async fn create(
        &self,
        dirid: fileid3,
        filename: &filename3,
        _attr: sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!("create {:?} {:?}", dirid, filename);
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }

    async fn create_exclusive(
        &self,
        _dirid: fileid3,
        _filename: &filename3,
    ) -> Result<fileid3, nfsstat3> {
        debug!("create_exclusive");
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }

    async fn lookup(&self, dirid: fileid3, filename: &filename3) -> Result<fileid3, nfsstat3> {
        debug!("lookup {:?} {:?}", dirid, filename);

        if let Ok(filename) = std::str::from_utf8(&filename.0) {
            if let Some(path) = self.inode_to_path(dirid).await {
                let path = Path::new(&path).join(filename);

                self.path_to_inode(&path.display().to_string()).await;

                return Err(nfsstat3::NFS3ERR_NOENT);
            }
        }

        Err(nfsstat3::NFS3ERR_NOENT)
    }

    async fn getattr(&self, id: fileid3) -> Result<fattr3, nfsstat3> {
        debug!("getattr {:?}", id);

        let path = self.inodes.read().await;
        let path = path.get_by_left(&id).ok_or(nfsstat3::NFS3ERR_NOENT)?;

        self.path_to_attr(path).await
    }
    async fn setattr(&self, id: fileid3, setattr: sattr3) -> Result<fattr3, nfsstat3> {
        debug!("setattr {:?} {:?}", id, setattr);
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }

    async fn read(
        &self,
        id: fileid3,
        offset: u64,
        count: u32,
    ) -> Result<(Vec<u8>, bool), nfsstat3> {
        debug!("read {:?} {:?} {:?}", id, offset, count);
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }

    async fn readdir(
        &self,
        dirid: fileid3,
        start_after: fileid3,
        max_entries: usize,
    ) -> Result<ReadDirResult, nfsstat3> {
        debug!("readdir {:?} {:?} {:?}", dirid, start_after, max_entries);

        let path = self
            .inode_to_path(dirid)
            .await
            .ok_or(nfsstat3::NFS3ERR_NOENT)?;

        let mut ds = self
            .operator
            .list(&path)
            .await
            .map_err(|_| nfsstat3::NFS3ERR_NOENT)?;

        let mut entries = Vec::new();

        while let Some(de) = ds.try_next().await.unwrap() {
            let id = self.path_to_inode(de.path()).await;

            if let Ok(attr) = self.getattr(id).await {
                entries.push(DirEntry {
                    attr,
                    fileid: id,
                    name: de.name().as_bytes().into(),
                })
            }
        }

        Ok(ReadDirResult { entries, end: true })
    }

    /// Removes a file.
    /// If not supported dur to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    #[allow(unused)]
    async fn remove(&self, dirid: fileid3, filename: &filename3) -> Result<(), nfsstat3> {
        debug!("remove {:?} {:?}", dirid, filename);
        return Err(nfsstat3::NFS3ERR_NOTSUPP);
    }

    /// Removes a file.
    /// If not supported dur to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    #[allow(unused)]
    async fn rename(
        &self,
        from_dirid: fileid3,
        from_filename: &filename3,
        to_dirid: fileid3,
        to_filename: &filename3,
    ) -> Result<(), nfsstat3> {
        debug!(
            "rename {:?} {:?} {:?} {:?}",
            from_dirid, from_filename, to_dirid, to_filename
        );
        return Err(nfsstat3::NFS3ERR_NOTSUPP);
    }

    #[allow(unused)]
    async fn mkdir(
        &self,
        _dirid: fileid3,
        _dirname: &filename3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!("mkdir");
        Err(nfsstat3::NFS3ERR_ROFS)
    }

    async fn symlink(
        &self,
        _dirid: fileid3,
        _linkname: &filename3,
        _symlink: &nfspath3,
        _attr: &sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!("symlink");
        Err(nfsstat3::NFS3ERR_ROFS)
    }
    async fn readlink(&self, _id: fileid3) -> Result<nfspath3, nfsstat3> {
        debug!("readlink");
        return Err(nfsstat3::NFS3ERR_NOTSUPP);
    }
}
