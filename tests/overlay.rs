mod common;
mod policies;

use futures::TryStreamExt;

use pretty_assertions::assert_eq;

use crate::{
    common::TestFixture,
    policies::{BaseOnlyPolicy, UnionPolicy},
};

#[tokio::test]
async fn test_base_only() -> anyhow::Result<()> {
    let fixture = TestFixture::new(BaseOnlyPolicy {})?;

    fixture.base.write("/hello.txt", "hello").await?;
    fixture.overlay.write("/world.txt", "world").await?;

    let ds = fixture.composite.list("/").await?;
    let mut entries: Vec<String> = ds
        .into_stream()
        .map_ok(|e| e.name().to_owned())
        .try_collect()
        .await?;

    entries.sort();

    assert_eq!(entries, &["hello.txt"]);
    Ok(())
}

#[tokio::test]
async fn test_union() -> anyhow::Result<()> {
    let fixture = TestFixture::new(UnionPolicy {})?;

    fixture.base.write("/hello.txt", "hello").await?;
    fixture.overlay.write("/world.txt", "world").await?;

    let ds = fixture.composite.list("/").await?;
    let mut entries: Vec<String> = ds
        .into_stream()
        .map_ok(|e| e.name().to_owned())
        .try_collect()
        .await?;

    entries.sort();

    assert_eq!(entries, &["hello.txt", "world.txt"]);

    Ok(())
}
