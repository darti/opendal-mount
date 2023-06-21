use async_trait::async_trait;
use opendal::{raw::oio, Result};

pub struct OverlayPager<B: oio::Page, O: oio::Page> {
    base: B,
    overlay: O,
}

impl<B: oio::Page, O: oio::Page> OverlayPager<B, O> {
    pub fn new(base: B, overlay: O) -> Self {
        OverlayPager { base, overlay }
    }
}

#[async_trait]
impl<B: oio::Page, O: oio::Page> oio::Page for OverlayPager<B, O> {
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

pub struct OverlayBlockingPager<B: oio::BlockingPage, O: oio::BlockingPage> {
    base: B,
    overlay: O,
}

impl<B: oio::BlockingPage, O: oio::BlockingPage> OverlayBlockingPager<B, O> {
    pub fn new(base: B, overlay: O) -> Self {
        OverlayBlockingPager { base, overlay }
    }
}

impl<B: oio::BlockingPage, O: oio::BlockingPage> oio::BlockingPage for OverlayBlockingPager<B, O> {
    fn next(&mut self) -> Result<Option<Vec<oio::Entry>>> {
        let overlay = self.overlay.next()?;
        let base = self.base.next()?;

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
