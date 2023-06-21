use std::sync::Arc;

use async_trait::async_trait;

use opendal::{
    raw::{
        Accessor, Layer, LayeredAccessor, OpAppend, OpList, OpRead, OpStat, OpWrite, RpAppend,
        RpList, RpRead, RpStat, RpWrite,
    },
    Builder, Result,
};

use super::{
    appender::OverlayAppender,
    pager::{OverlayBlockingPager, OverlayPager},
    policy::Policy,
    reader::{OverlayBlockingReader, OverlayReader},
    writer::{OverlayBlockingWriter, OverlayWriter},
};

pub struct Overlay {}

impl Overlay {
    pub fn new<O: Builder, P: Policy>(
        mut builder: O,
        policy: P,
    ) -> opendal::Result<OverlayLayer<O::Accessor, P>> {
        let overlay = builder.build()?;
        Ok(OverlayLayer {
            overlay: Arc::new(overlay),
            policy: Arc::new(policy),
        })
    }
}

#[derive(Default, Debug)]
pub struct OverlayLayer<O: Accessor, P: Policy> {
    overlay: Arc<O>,
    policy: Arc<P>,
}

impl<B: Accessor, O: Accessor, P: Policy> Layer<B> for OverlayLayer<O, P> {
    type LayeredAccessor = OverlayAccessor<B, O, P>;

    fn layer(&self, inner: B) -> Self::LayeredAccessor {
        OverlayAccessor {
            base: inner,
            overlay: self.overlay.clone(),
            policy: self.policy.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverlayAccessor<B: Accessor, O: Accessor, P: Policy> {
    base: B,
    overlay: Arc<O>,
    policy: Arc<P>,
}

#[async_trait]
impl<B: Accessor, O: Accessor, P: Policy> LayeredAccessor for OverlayAccessor<B, O, P> {
    type Inner = B;
    type Reader = OverlayReader<B::Reader, O::Reader>;
    type BlockingReader = OverlayBlockingReader<B::BlockingReader, O::BlockingReader>;
    type Writer = OverlayWriter<O::Writer, B::Writer>;
    type BlockingWriter = OverlayBlockingWriter<O::BlockingWriter, B::BlockingWriter>;
    type Appender = OverlayAppender<O::Appender, B::Appender>;
    type Pager = OverlayPager<B::Pager, O::Pager>;
    type BlockingPager = OverlayBlockingPager<B::BlockingPager, O::BlockingPager>;

    fn inner(&self) -> &Self::Inner {
        &self.base
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        self.policy.stat(self.base, self.overlay, path, args).await
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        self.policy.read(self.base, self.overlay, path, args).await
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        todo!()
    }

    async fn append(&self, path: &str, args: OpAppend) -> Result<(RpAppend, Self::Appender)> {
        todo!()
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Pager)> {
        let (rp, base) = self.base.list(path, args.clone()).await?;
        let (_, overlay) = self.overlay.list(path, args).await?;

        Ok((rp, OverlayPager::new(base, overlay)))
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
