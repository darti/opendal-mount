use async_trait::async_trait;
use futures::TryStreamExt;
use opendal::{
    raw::{
        oio::{self, Page},
        AccessorInfo,
    },
    services::Fs,
    Capability, Operator,
};
use opendal_mount::{
    overlay::policy::{Policy, PolicyOperation, Source},
    Overlay,
};
use pretty_assertions::assert_eq;

const BASE_ROOT: &str = "./tests/samples/base";
const OVERLAY_ROOT: &str = "./tests/samples/overlay";

#[derive(Debug, Clone)]
struct BaseOnlyPolicy {}

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

#[tokio::test]
async fn test_base_only() -> anyhow::Result<()> {
    let mut base_builder = Fs::default();
    base_builder.root(BASE_ROOT);

    let mut overlay_builder = Fs::default();
    overlay_builder.root(OVERLAY_ROOT);

    let mut builder = Overlay::default();

    builder
        .base_builder(base_builder)
        .overlay_builder(overlay_builder)
        .policy(BaseOnlyPolicy {});

    let op = Operator::new(builder)?.finish();

    let ds = op.list("/").await?;
    let mut entries: Vec<String> = ds
        .into_stream()
        .map_ok(|e| e.name().to_owned())
        .try_collect()
        .await?;

    entries.sort();

    assert_eq!(entries, &["hello.txt"]);
    Ok(())
}
