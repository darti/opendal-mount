use bytes::Bytes;
use opendal::raw::*;
use opendal::*;
use tracing::info;

#[cfg(target_os = "macos")]
const VOLUME_ICON_PATH: &str = "/.VolumeIcon.icns";

#[cfg(target_os = "windows")]
const VOLUME_ICON_PATH: &str = ".VolumeIcon.icns";

#[cfg(target_os = "linux")]
const VOLUME_ICON_PATH: &str = ".VolumeIcon.icns";

pub struct VolumeIconLayer {
    icon: Bytes,
}

impl VolumeIconLayer {
    pub fn new(icon: Bytes) -> Self {
        Self { icon }
    }
}

impl<A: Access> Layer<A> for VolumeIconLayer {
    type LayeredAccess = VolumeIconAccessor<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccess {
        VolumeIconAccessor {
            inner,
            icon: self.icon.clone(),
        }
    }
}

#[derive(Debug)]
pub struct VolumeIconAccessor<A: Access> {
    inner: A,
    icon: Bytes,
}

impl<A: Access> LayeredAccess for VolumeIconAccessor<A> {
    type Inner = A;
    type Reader = VolumeIconReader<A::Reader>;
    type BlockingReader = VolumeIconReader<A::BlockingReader>;
    type Writer = A::Writer;
    type BlockingWriter = A::BlockingWriter;
    type Lister = A::Lister;
    type BlockingLister = A::BlockingLister;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    #[tracing::instrument]
    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        if path == VOLUME_ICON_PATH {
            info!("Reading volume icon");
            let len = self.icon.len() as u64;
            let rp_read = RpRead::new().with_size(Some(len));

            Ok((rp_read, VolumeIconReader::Icon(self.icon.clone())))
        } else {
            self.inner
                .read(path, args)
                .await
                .map(|(rp, r)| (rp, VolumeIconReader::Inner(r)))
        }
    }

    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        if path == VOLUME_ICON_PATH {
            info!("Reading volume icon");
            let len = self.icon.len() as u64;
            let rp_read = RpRead::new().with_size(Some(len));

            Ok((rp_read, VolumeIconReader::Icon(self.icon.clone())))
        } else {
            self.inner
                .blocking_read(path, args)
                .map(|(rp, r)| (rp, VolumeIconReader::Inner(r)))
        }
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        self.inner.write(path, args).await
    }

    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        self.inner.blocking_write(path, args)
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        self.inner.list(path, args).await
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        self.inner.blocking_list(path, args)
    }
}

pub enum VolumeIconReader<R> {
    Inner(R),
    Icon(Bytes),
}

impl<R: oio::Read> oio::Read for VolumeIconReader<R> {
    async fn read(&mut self) -> Result<Buffer> {
        match self {
            Self::Inner(r) => r.read().await,
            Self::Icon(icon) => Ok(icon.clone().into()),
        }
    }
}

impl<R: oio::BlockingRead> oio::BlockingRead for VolumeIconReader<R> {
    fn read(&mut self) -> Result<Buffer> {
        match self {
            Self::Inner(r) => r.read(),
            Self::Icon(icon) => Ok(icon.clone().into()),
        }
    }
}
