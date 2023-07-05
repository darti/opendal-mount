use async_trait::async_trait;
use opendal::{
    raw::{
        oio::{self, Page},
        AccessorInfo,
    },
    Capability,
};
use opendal_mount::overlay::policy::{Policy, PolicyOperation, Source};

#[derive(Debug, Clone)]
pub struct BaseOnlyPolicy {}

#[async_trait]
impl Policy for BaseOnlyPolicy {
    fn owner<B, O>(
        &self,
        _base_info: B,
        _overlay_info: O,
        _path: &str,
        _op: PolicyOperation,
    ) -> Source
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo,
    {
        Source::Base
    }

    fn capability<B, O>(&self, base_info: B, _overlay_info: O) -> Capability
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo,
    {
        base_info().capability()
    }

    async fn next<B: Page, O: Page>(
        &self,
        base: &mut B,
        _overlay: &mut O,
    ) -> opendal::Result<Option<Vec<oio::Entry>>> {
        base.next().await
    }
}

#[derive(Debug, Clone)]
pub struct UnionPolicy {}

#[async_trait]
impl Policy for UnionPolicy {
    fn owner<B, O>(
        &self,
        _base_info: B,
        _overlay_info: O,
        _path: &str,
        _op: PolicyOperation,
    ) -> Source
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo,
    {
        Source::Base
    }

    fn capability<B, O>(&self, base_info: B, _overlay_info: O) -> Capability
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo,
    {
        base_info().capability()
    }

    async fn next<B: Page, O: Page>(
        &self,
        base: &mut B,
        overlay: &mut O,
    ) -> opendal::Result<Option<Vec<oio::Entry>>> {
        let entries = overlay.next().await?;
        if let Some(_) = entries {
            return Ok(entries);
        } else {
            let entries = base.next().await;

            entries
        }
    }
}
