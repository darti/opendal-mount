use async_trait::async_trait;
use opendal::raw::oio;

pub struct OverlayPager {}

#[async_trait]
impl oio::Page for OverlayPager {
    async fn next(&mut self) -> opendal::Result<Option<Vec<oio::Entry>>> {
        todo!()
    }
}

impl oio::BlockingPage for OverlayPager {
    fn next(&mut self) -> opendal::Result<Option<Vec<oio::Entry>>> {
        todo!()
    }
}
