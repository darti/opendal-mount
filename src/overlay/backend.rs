use std::{io, sync::Arc};

use async_trait::async_trait;

use bytes::Bytes;
use log::debug;
use opendal::{
    raw::{
        oio::{self, Page},
        Accessor, Layer, LayeredAccessor, OpAppend, OpList, OpRead, OpStat, OpWrite, Operation,
        RpAppend, RpList, RpRead, RpStat, RpWrite,
    },
    Builder, Operator, Result,
};

use super::{
    pager::{OverlayBlockingPager, OverlayPager},
    reader::OverlayReader,
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
    type Reader = OverlayReader<B::Reader, O::Reader>;
    type BlockingReader = OverlayWrapper<O::BlockingReader>;
    type Writer = OverlayWrapper<O::Writer>;
    type BlockingWriter = OverlayWrapper<O::BlockingWriter>;
    type Appender = OverlayWrapper<O::Appender>;
    type Pager = OverlayPager<B::Pager, O::Pager>;
    type BlockingPager = OverlayBlockingPager<B::BlockingPager, O::BlockingPager>;

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
        if let Ok((rp, o)) = self.overlay.read(path, args.clone()).await {
            Ok((rp, OverlayReader::Overlay(o)))
        } else {
            self.base
                .read(path, args)
                .await
                .map(|(rp, b)| (rp, OverlayReader::Base(b)))
        }
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

#[async_trait]
impl<R: oio::Append> oio::Append for OverlayWrapper<R> {
    async fn append(&mut self, bs: Bytes) -> Result<()> {
        self.inner.append(bs).await
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}
