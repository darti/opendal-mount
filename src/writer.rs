use async_trait::async_trait;
use bytes::Bytes;
use opendal::raw::oio;

pub struct OverlayWriter {}

#[async_trait]
impl oio::Write for OverlayWriter {
    async fn write(&mut self, bs: Bytes) -> opendal::Result<()> {
        todo!()
    }

    /// Abort the pending writer.
    async fn abort(&mut self) -> opendal::Result<()> {
        todo!()
    }

    /// Close the writer and make sure all data has been flushed.
    async fn close(&mut self) -> opendal::Result<()> {
        todo!()
    }
}

impl oio::BlockingWrite for OverlayWriter {
    fn write(&mut self, bs: Bytes) -> opendal::Result<()> {
        todo!()
    }

    fn close(&mut self) -> opendal::Result<()> {
        todo!()
    }
}
