use std::sync::Arc;

use async_trait::async_trait;

use opendal::{
    raw::oio::{self, Page},
    Result,
};

use super::policy::Policy;

pub struct OverlayPager<B: Page, O: Page, P: Policy> {
    base: B,
    overlay: O,
    policy: Arc<P>,
}

impl<B: Page, O: Page, P: Policy> OverlayPager<B, O, P> {
    pub fn new(base: B, overlay: O, policy: Arc<P>) -> Self {
        OverlayPager {
            base,
            overlay,
            policy,
        }
    }
}

#[async_trait]
impl<B: Page, O: Page, P: Policy> oio::Page for OverlayPager<B, O, P> {
    async fn next(&mut self) -> Result<Option<Vec<oio::Entry>>> {
        self.policy.next(&mut self.base, &mut self.overlay).await
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
