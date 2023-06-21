use async_trait::async_trait;
use bytes::Bytes;
use opendal::{raw::oio, Result};

pub enum OverlayAppender<B: oio::Append, O: oio::Append> {
    Base(B),
    Overlay(O),
}

#[async_trait]
impl<B: oio::Append, O: oio::Append> oio::Append for OverlayAppender<O, B> {
    async fn append(&mut self, bs: Bytes) -> Result<()> {
        match self {
            OverlayAppender::Base(b) => b.append(bs).await,
            OverlayAppender::Overlay(o) => o.append(bs).await,
        }
    }

    async fn close(&mut self) -> Result<()> {
        match self {
            OverlayAppender::Base(b) => b.close().await,
            OverlayAppender::Overlay(o) => o.close().await,
        }
    }
}
