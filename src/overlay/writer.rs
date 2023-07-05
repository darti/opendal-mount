use async_trait::async_trait;
use bytes::Bytes;
use log::debug;
use opendal::{raw::oio, Result};

pub enum OverlayWriter<B: oio::Write, O: oio::Write> {
    Base(B),
    Overlay(O),
}

#[async_trait]
impl<B: oio::Write, O: oio::Write> oio::Write for OverlayWriter<B, O> {
    async fn write(&mut self, bs: Bytes) -> Result<()> {
        debug!("OverlayWriter::write({:?})", bs.len());
        match self {
            Self::Base(b) => b.write(bs).await,
            Self::Overlay(o) => o.write(bs).await,
        }
    }

    async fn abort(&mut self) -> Result<()> {
        match self {
            Self::Base(b) => b.abort().await,
            Self::Overlay(o) => o.abort().await,
        }
    }

    async fn close(&mut self) -> Result<()> {
        match self {
            Self::Base(b) => b.close().await,
            Self::Overlay(o) => o.close().await,
        }
    }

    async fn sink(&mut self, size: u64, s: oio::Streamer) -> Result<()> {
        match self {
            Self::Base(b) => b.sink(size, s).await,
            Self::Overlay(o) => o.sink(size, s).await,
        }
    }
}

pub enum OverlayBlockingWriter<B: oio::BlockingWrite, O: oio::BlockingWrite> {
    Base(B),
    Overlay(O),
}

impl<B: oio::BlockingWrite, O: oio::BlockingWrite> oio::BlockingWrite
    for OverlayBlockingWriter<B, O>
{
    fn write(&mut self, bs: Bytes) -> Result<()> {
        match self {
            Self::Base(b) => b.write(bs),
            Self::Overlay(o) => o.write(bs),
        }
    }

    fn close(&mut self) -> Result<()> {
        match self {
            Self::Base(b) => b.close(),
            Self::Overlay(o) => o.close(),
        }
    }
}
