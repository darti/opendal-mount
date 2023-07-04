use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use opendal::{
    raw::{
        Accessor, AccessorInfo, OpAppend, OpCreateDir, OpList, OpRead, OpStat, OpWrite, RpAppend,
        RpCreateDir, RpList, RpRead, RpStat, RpWrite,
    },
    Builder, Capability, Result, Scheme,
};

use super::{
    appender::OverlayAppender,
    pager::{OverlayBlockingPager, OverlayPager},
    policy::Policy,
    reader::{OverlayBlockingReader, OverlayReader},
    writer::{OverlayBlockingWriter, OverlayWriter},
};

pub struct OverlayBuilder<B: Builder, O: Builder, P: Policy> {
    base_builder: Option<B>,
    overlay_builder: Option<O>,
    policy: Option<P>,
}

impl<B: Builder, O: Builder, P: Policy> OverlayBuilder<B, O, P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_builder(&mut self, base_builder: B) -> &mut Self {
        self.base_builder = Some(base_builder);
        self
    }

    pub fn overlay_builder(&mut self, overlay_builder: O) -> &mut Self {
        self.overlay_builder = Some(overlay_builder);
        self
    }

    pub fn policy(&mut self, policy: P) -> &mut Self {
        self.policy = Some(policy);
        self
    }
}

impl<B: Builder, O: Builder, P: Policy> Builder for OverlayBuilder<B, O, P> {
    const SCHEME: Scheme = Scheme::Custom("overlay");
    type Accessor = OverlayBackend<B::Accessor, O::Accessor, P>;

    fn build(&mut self) -> Result<Self::Accessor> {
        let mut base_builder = match self.base_builder.take() {
            Some(b) => Ok(b),
            None => Err(opendal::Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "base builder is not specified",
            )),
        }?;

        let mut overlay_builder = match self.overlay_builder.take() {
            Some(b) => Ok(b),
            None => Err(opendal::Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "overlay builder is not specified",
            )),
        }?;

        let policy = match self.policy.take() {
            Some(p) => Ok(p),
            None => Err(opendal::Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "policy is not specified",
            )),
        }?;

        Ok(OverlayBackend {
            base: Arc::new(base_builder.build()?),
            overlay: Arc::new(overlay_builder.build()?),
            policy: Arc::new(policy),
        })
    }

    fn from_map(_map: HashMap<String, String>) -> Self {
        todo!()
    }
}

impl<B: Builder, O: Builder, P: Policy> Default for OverlayBuilder<B, O, P> {
    fn default() -> Self {
        Self {
            base_builder: None,
            overlay_builder: None,
            policy: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverlayBackend<B: Accessor, O: Accessor, P: Policy> {
    base: Arc<B>,
    overlay: Arc<O>,
    policy: Arc<P>,
}

#[async_trait]
impl<B: Accessor, O: Accessor, P: Policy> Accessor for OverlayBackend<B, O, P> {
    type Reader = OverlayReader<B::Reader, O::Reader>;
    type BlockingReader = OverlayBlockingReader<B::BlockingReader, O::BlockingReader>;
    type Writer = OverlayWriter<O::Writer, B::Writer>;
    type BlockingWriter = OverlayBlockingWriter<O::BlockingWriter, B::BlockingWriter>;
    type Appender = OverlayAppender<O::Appender, B::Appender>;
    type Pager = OverlayPager<B::Pager, O::Pager, P>;
    type BlockingPager = OverlayBlockingPager<B::BlockingPager, O::BlockingPager>;

    fn info(&self) -> AccessorInfo {
        let mut info = AccessorInfo::default();
        info.set_scheme(opendal::Scheme::Custom("overlay"));
        info.set_capability(Capability {
            stat: true,

            read: true,
            read_can_seek: true,
            read_with_range: true,

            write: true,
            write_without_content_length: true,
            create_dir: true,
            delete: true,

            append: true,

            list: true,
            list_with_delimiter_slash: true,

            copy: true,
            rename: true,
            blocking: true,

            ..Default::default()
        });

        info
    }

    async fn create_dir(&self, path: &str, args: OpCreateDir) -> Result<RpCreateDir> {
        self.policy
            .create_dir(self.base.clone(), self.overlay.clone(), path, args)
            .await
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        self.policy
            .stat(self.base.clone(), self.overlay.clone(), path, args)
            .await
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        self.policy
            .read(self.base.clone(), self.overlay.clone(), path, args)
            .await
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        self.policy
            .write(self.overlay.clone(), self.base.clone(), path, args)
            .await
    }

    async fn append(&self, path: &str, args: OpAppend) -> Result<(RpAppend, Self::Appender)> {
        todo!()
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Pager)> {
        let (_, b) = self.base.list(path, args.clone()).await?;
        let (_, o) = self.overlay.list(path, args).await?;

        Ok((RpList {}, OverlayPager::new(b, o, self.policy.clone())))
    }

    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        todo!()
    }

    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        todo!()
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingPager)> {
        let (rp, base) = self.base.blocking_list(path, args.clone())?;
        let (_, overlay) = self.overlay.blocking_list(path, args)?;

        Ok((rp, OverlayBlockingPager::new(base, overlay)))
    }
}
