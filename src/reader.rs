use bytes::Bytes;
use opendal::raw::oio;

pub struct OverlayReader {}

impl oio::Read for OverlayReader {
    fn poll_read(
        &mut self,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<opendal::Result<usize>> {
        todo!()
    }

    fn poll_seek(
        &mut self,
        cx: &mut std::task::Context<'_>,
        pos: std::io::SeekFrom,
    ) -> std::task::Poll<opendal::Result<u64>> {
        todo!()
    }

    fn poll_next(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<opendal::Result<Bytes>>> {
        todo!()
    }
}

impl oio::BlockingRead for OverlayReader {
    fn read(&mut self, buf: &mut [u8]) -> opendal::Result<usize> {
        todo!()
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> opendal::Result<u64> {
        todo!()
    }

    fn next(&mut self) -> Option<opendal::Result<Bytes>> {
        todo!()
    }
}
