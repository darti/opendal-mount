// use opendal::{services::Fs, Operator};
// use opendal_mount::Overlay;
// use pretty_assertions::assert_eq;

// use futures::TryStreamExt;

// const BASE_ROOT: &str = "./tests/samples/base";
// const OVERLAY_ROOT: &str = "./tests/samples/overlay";

// const ENTRIES: [&str; 2] = ["hello.txt", "world.txt"];

use opendal::services::Memory;

#[tokio::test]
async fn test_overlay_file_union() -> anyhow::Result<()> {
    let mut base_builder = Memory::default();

    let mut overlay_builder = Memory::default();

    //     let overlay = Overlay::new(overlay_builder, NaivePolicy)?;

    //     let op = Operator::new(base_builder)?.layer(overlay).finish();

    //     let ds = op.list("/").await?;
    //     let mut entries: Vec<String> = ds
    //         .into_stream()
    //         .map_ok(|e| e.name().to_owned())
    //         .try_collect()
    //         .await?;

    //     entries.sort();

    //     assert_eq!(entries, ENTRIES);
    Ok(())
}
