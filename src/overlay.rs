use std::{
    io,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;

use bytes::Bytes;
use log::debug;
use opendal::{
    raw::{
        oio::{self},
        Accessor, Layer, LayeredAccessor, OpAppend, OpList, OpRead, OpStat, OpWrite, Operation,
        RpAppend, RpList, RpRead, RpStat, RpWrite,
    },
    Builder, Operator, Result,
};

type SourcePolicy = fn(&str, Operator, Operator, Operation) -> opendal::Result<Operator>;

pub struct Overlay {}

impl Overlay {
    pub fn new<B: Builder>(mut builder: B) -> opendal::Result<OverlayLayer<B::Accessor>> {
        let overlay = builder.build()?;
        Ok(OverlayLayer {
            overlay: Arc::new(overlay),
        })
    }
}

#[derive(Default, Debug)]
pub struct OverlayLayer<B: Accessor> {
    overlay: Arc<B>,
}

impl<A: Accessor, B: Accessor> Layer<A> for OverlayLayer<B> {
    type LayeredAccessor = OverlayAccessor<A, B>;

    fn layer(&self, inner: A) -> Self::LayeredAccessor {
        OverlayAccessor {
            base: inner,
            overlay: self.overlay.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverlayAccessor<B: Accessor, O: Accessor> {
    base: B,
    overlay: Arc<O>,
    // policy: SourcePolicy,
}

#[async_trait]
impl<B: Accessor, O: Accessor> LayeredAccessor for OverlayAccessor<B, O> {
    type Inner = B;
    type Reader = OverlayWrapper<O::Reader>;
    type BlockingReader = OverlayWrapper<O::BlockingReader>;
    type Writer = OverlayWrapper<O::Writer>;
    type BlockingWriter = OverlayWrapper<O::BlockingWriter>;
    type Appender = OverlayWrapper<O::Appender>;
    type Pager = OverlayPager<B::Pager, O::Pager>;
    type BlockingPager = OverlayWrapper<O::BlockingPager>;

    fn inner(&self) -> &Self::Inner {
        &self.base
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        if let Ok(meta) = self.overlay.stat(path, args.clone()).await {
            Ok(meta)
        } else {
            self.inner().stat(path, args).await
        }
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        todo!()
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
        todo!()
    }
}

pub struct OverlayWrapper<R> {
    inner: R,
}

impl<R> OverlayWrapper<R> {
    fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: oio::Read> oio::Read for OverlayWrapper<R> {
    fn poll_read(&mut self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        self.inner.poll_read(cx, buf)
    }

    fn poll_seek(&mut self, cx: &mut Context<'_>, pos: io::SeekFrom) -> Poll<Result<u64>> {
        self.inner.poll_seek(cx, pos)
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes>>> {
        self.inner.poll_next(cx)
    }
}

impl<R: oio::BlockingRead> oio::BlockingRead for OverlayWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }

    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }

    fn next(&mut self) -> Option<Result<Bytes>> {
        self.inner.next()
    }
}

#[async_trait]
impl<R: oio::Write> oio::Write for OverlayWrapper<R> {
    async fn write(&mut self, bs: Bytes) -> Result<()> {
        self.inner.write(bs).await
    }

    async fn abort(&mut self) -> Result<()> {
        self.inner.abort().await
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl<R: oio::BlockingWrite> oio::BlockingWrite for OverlayWrapper<R> {
    fn write(&mut self, bs: Bytes) -> Result<()> {
        self.inner.write(bs)
    }

    fn close(&mut self) -> Result<()> {
        self.inner.close()
    }
}

pub struct OverlayPager<B: oio::Page, O: oio::Page> {
    base: B,
    overlay: O,
}

impl<B: oio::Page, O: oio::Page> OverlayPager<B, O> {
    fn new(base: B, overlay: O) -> Self {
        OverlayPager { base, overlay }
    }
}

#[async_trait]
impl<A: oio::Page, B: oio::Page> oio::Page for OverlayPager<A, B> {
    async fn next(&mut self) -> Result<Option<Vec<oio::Entry>>> {
        let overlay = self.overlay.next().await?;
        let base = self.base.next().await?;

        let entries = match (overlay, base) {
            (Some(ov), None) => Some(ov),
            (None, Some(ba)) => Some(ba),
            (None, None) => None,
            (Some(ov), Some(ba)) => {
                let mut entries = ov;
                entries.extend(ba);
                Some(entries)
            }
        };

        Ok(entries)
    }
}

impl<R: oio::BlockingPage> oio::BlockingPage for OverlayWrapper<R> {
    fn next(&mut self) -> Result<Option<Vec<oio::Entry>>> {
        self.inner.next()
    }
}

#[async_trait]
impl<R: oio::Append> oio::Append for OverlayWrapper<R> {
    async fn append(&mut self, bs: Bytes) -> Result<()> {
        self.inner.append(bs).await
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}
