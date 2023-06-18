use std::{
    io,
    task::{Context, Poll},
};

use async_trait::async_trait;

use bytes::Bytes;
use opendal::{
    raw::{
        oio, Accessor, Layer, LayeredAccessor, OpAppend, OpList, OpRead, OpWrite, Operation,
        RpAppend, RpList, RpRead, RpWrite,
    },
    Builder, Operator, Result,
};

type SourcePolicy = fn(&str, Operator, Operator, Operation) -> opendal::Result<Operator>;

pub struct Overlay {}

impl Overlay {
    pub fn new<B: Builder>(mut builder: B) -> opendal::Result<OverlayLayer<B::Accessor>> {
        let overlay = builder.build()?;
        Ok(OverlayLayer { overlay })
    }
}

#[derive(Default, Debug)]
pub struct OverlayLayer<O: Accessor> {
    overlay: O,
}

impl<A: Accessor, O: Accessor> Layer<A> for OverlayLayer<O> {
    type LayeredAccessor = OverlayAccessor<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccessor {
        OverlayAccessor { inner }
    }
}

#[derive(Debug, Clone)]
pub struct OverlayAccessor<A: Accessor> {
    // overlay: Operator,
    inner: A,
    // policy: SourcePolicy,
}

#[async_trait]
impl<A: Accessor> LayeredAccessor for OverlayAccessor<A> {
    type Inner = A;
    type Reader = OverlayWrapper<A::Reader>;
    type BlockingReader = OverlayWrapper<A::BlockingReader>;
    type Writer = OverlayWrapper<A::Writer>;
    type BlockingWriter = OverlayWrapper<A::BlockingWriter>;
    type Appender = OverlayWrapper<A::Appender>;
    type Pager = Option<OverlayWrapper<A::Pager>>;
    type BlockingPager = Option<OverlayWrapper<A::BlockingPager>>;

    fn inner(&self) -> &Self::Inner {
        &self.inner
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
        todo!()
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

#[async_trait]
impl<R: oio::Page> oio::Page for OverlayWrapper<R> {
    async fn next(&mut self) -> Result<Option<Vec<oio::Entry>>> {
        self.inner.next().await
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
