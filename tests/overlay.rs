use opendal::{services::Fs, Entry, Operator};
use opendal_mount::{overlay::policy::NaivePolicy, Overlay};
use pretty_assertions::{assert_eq, assert_ne};

use futures::TryStreamExt;

const BASE_ROOT: &str = "./tests/samples/base";
const OVERLAY_ROOT: &str = "./tests/samples/overlay";

const ENTRIES: [&str; 2] = ["hello.txt", "world.txt"];

#[tokio::test]
async fn test_overlay_file_union() -> anyhow::Result<()> {
    let mut base_builder = Fs::default();
    base_builder.root(BASE_ROOT);

    let mut overlay_builder = Fs::default();
    overlay_builder.root(OVERLAY_ROOT);

    let overlay = Overlay::new(overlay_builder, NaivePolicy)?;

    let op = Operator::new(base_builder)?.layer(overlay).finish();

    let ds = op.list("/").await?;
    let mut entries: Vec<String> = ds
        .into_stream()
        .map_ok(|e| e.name().to_owned())
        .try_collect()
        .await?;

    entries.sort();

    assert_eq!(entries, ENTRIES);
    Ok(())
}
