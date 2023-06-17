use async_trait::async_trait;
use bytes::Bytes;
use opendal::raw::oio;

pub struct OverlayAppender {}

#[async_trait]
impl oio::Append for OverlayAppender {
    async fn append(&mut self, bs: Bytes) -> opendal::Result<()> {
        todo!()
    }

    async fn close(&mut self) -> opendal::Result<()> {
        todo!()
    }
}
