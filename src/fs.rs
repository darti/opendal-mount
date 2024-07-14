use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use bimap::BiMap;
use log::{debug, warn};
use nfsserve::{
    nfs::{fattr3, fileid3, filename3, ftype3, nfspath3, nfsstat3, nfstime3, sattr3, specdata3},
    vfs::{DirEntry, NFSFileSystem, ReadDirResult, VFSCapabilities},
};
use opendal::Operator;
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
}

#[async_trait]
impl NFSFileSystem for OpendalFs {
    fn root_dir(&self) -> fileid3 {
        debug!("root_dir");

        1
    }

    fn capabilities(&self) -> VFSCapabilities {
        debug!("capabilities");

        if self.operator.info().full_capability().write {
            VFSCapabilities::ReadWrite
        } else {
            VFSCapabilities::ReadOnly
        }
    }

    async fn write(&self, id: fileid3, offset: u64, data: &[u8]) -> Result<fattr3, nfsstat3> {
        debug!("write {:?} {:?} {:?}", id, offset, data);

        let path = self.inode_to_path(id).await;

        if let Some(path) = path {
            if offset == 0 {
                self.operator
                    .write(&path, data.to_vec())
                    .await
                    .map_err(|_| {
                        warn!("unable to write to {:?}", path);
                        nfsstat3::NFS3ERR_IO
                    })
            } else {
                self.operator
                    .write_with(&path, data.to_vec())
                    .append(true)
                    .await
                    .map_err(|_| {
                        warn!("unable to append to {:?}", path);
                        nfsstat3::NFS3ERR_IO
                    })
            }?;

            let attr = self.path_to_attr(id, &path).await?;

            Ok(attr)
        } else {
            Err(nfsstat3::NFS3ERR_NOENT)
        }
    }

    async fn create(
        &self,
        dirid: fileid3,
        filename: &filename3,
        _attr: sattr3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!("create {:?} {:?}", dirid, filename);

        let filename = std::str::from_utf8(&filename.0);
        let path = self.inode_to_path(dirid).await;

        if let (Ok(filename), Some(path)) = (filename, path) {
            let path = Path::new(&path).join(filename);
            let ino = self
                .path_to_inode(&path.display().to_string(), true)
                .await?;

            self.write(ino, 0, &[]).await.map(|attr| (ino, attr))
        } else {
            warn!("unable to create file {:?} {:?}", dirid, filename);
            Err(nfsstat3::NFS3ERR_NOENT)
        }
    }

    async fn create_exclusive(
        &self,
        dirid: fileid3,
        filename: &filename3,
    ) -> Result<fileid3, nfsstat3> {
        debug!("create_exclusive {:?} {:?}", dirid, filename);

        let filename = std::str::from_utf8(&filename.0);
        let path = self.inode_to_path(dirid).await;

        if let (Ok(filename), Some(path)) = (filename, path) {
            let path = Path::new(&path).join(filename);
            let ino = self
                .path_to_inode(&path.display().to_string(), true)
                .await?;

            self.write(ino, 0, &[0]).await?;

            Ok(ino)
        } else {
            warn!("unable to create file {:?} {:?}", dirid, filename);
            Err(nfsstat3::NFS3ERR_NOENT)
        }
    }

    async fn lookup(&self, dirid: fileid3, filename: &filename3) -> Result<fileid3, nfsstat3> {
        debug!("lookup {:?} {:?}", dirid, filename);

        let filename = std::str::from_utf8(&filename.0);
        let path = self.inode_to_path(dirid).await;

        if let (Ok(filename), Some(path)) = (filename, path) {
            let path = Path::new(&path).join(filename);

            self.path_to_inode(&path.display().to_string(), false).await
        } else {
            Err(nfsstat3::NFS3ERR_NOENT)
        }
    }

    async fn getattr(&self, id: fileid3) -> Result<fattr3, nfsstat3> {
        debug!("getattr {:?}", id);

        let path = self
            .inode_to_path(id)
            .await
            .ok_or(nfsstat3::NFS3ERR_NOENT)?;

        self.path_to_attr(id, &path).await
    }
    async fn setattr(&self, id: fileid3, setattr: sattr3) -> Result<fattr3, nfsstat3> {
        debug!("setattr {:?} {:?}", id, setattr);

        let path = self
            .inode_to_path(id)
            .await
            .ok_or(nfsstat3::NFS3ERR_NOENT)?;

        let attrs = self.path_to_attr(id, &path).await?;

        Ok(attrs)
    }

    async fn read(
        &self,
        id: fileid3,
        offset: u64,
        count: u32,
    ) -> Result<(Vec<u8>, bool), nfsstat3> {
        debug!("read {:?} {:?} {:?}", id, offset, count);

        let path = self
            .inode_to_path(id)
            .await
            .ok_or(nfsstat3::NFS3ERR_NOENT)?;

        let data = self
            .operator
            .read_with(&path)
            .range(offset..offset + count as u64)
            .await;

        match data {
            Ok(data) => {
                let eof = data.len() < count as usize;
                Ok((data.to_vec(), eof))
            }
            Err(e) => {
                warn!("read error: {:?}", e);
                Err(nfsstat3::NFS3ERR_NOENT)
            }
        }
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

        let ds = self
            .operator
            .list(&path)
            .await
            .map_err(|_| nfsstat3::NFS3ERR_NOENT)?;

        let mut entries = Vec::new();

        let mut capture: bool = start_after == 0;

        for de in ds {
            let id = self.path_to_inode(de.path(), true).await?;

            if capture {
                if let Ok(attr) = self.getattr(id).await {
                    entries.push(DirEntry {
                        attr,
                        fileid: id,
                        name: de.name().trim_end_matches('/').as_bytes().into(),
                    });

                    if entries.len() >= max_entries {
                        break;
                    }
                }
            }

            capture = capture || id == start_after;
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
        dirid: fileid3,
        dirname: &filename3,
    ) -> Result<(fileid3, fattr3), nfsstat3> {
        debug!("mkdir {:?} {:?}", dirid, dirname);

        let dirname = std::str::from_utf8(&dirname.0);
        let path = self.inode_to_path(dirid).await;

        if let (Ok(dirname), Some(path)) = (dirname, path) {
            let path = Path::new(&path).join(dirname);
            let path = path.to_str().ok_or(nfsstat3::NFS3ERR_NOENT)?;
            let ino = self.path_to_inode(&path, true).await?;

            self.operator.create_dir(path).await.map_err(|e| {
                warn!("unable to create dir {:?} {:?}: {:?}", dirid, dirname, e);
                nfsstat3::NFS3ERR_NOENT
            })?;

            let attr = self.path_to_attr(ino, &path).await?;

            Ok((ino, attr))
        } else {
            warn!("unable to create file {:?} {:?}", dirid, dirname);
            Err(nfsstat3::NFS3ERR_NOENT)
        }
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
