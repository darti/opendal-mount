mod common;
mod policies;

use std::str::from_utf8;

use pretty_assertions::assert_eq;

use crate::{
    common::{ListDir, TestFixture},
    policies::{BaseOnlyPolicy, UnionPolicy},
};

#[tokio::test]
async fn list_base_only() -> anyhow::Result<()> {
    let fixture = TestFixture::new(BaseOnlyPolicy {})?;

    fixture.base.write("/hello.txt", "hello").await?;
    fixture.overlay.write("/world.txt", "world").await?;

    assert_eq!(fixture.composite.entries("/").await?, &["hello.txt"]);
    Ok(())
}

#[tokio::test]
async fn list_union() -> anyhow::Result<()> {
    let fixture = TestFixture::new(UnionPolicy {})?;

    fixture.base.write("/hello.txt", "hello").await?;
    fixture.overlay.write("/world.txt", "world").await?;

    assert_eq!(
        fixture.composite.entries("/").await?,
        &["hello.txt", "world.txt"]
    );

    Ok(())
}

#[tokio::test]
async fn read_base_only() -> anyhow::Result<()> {
    let fixture = TestFixture::new(BaseOnlyPolicy {})?;

    let ref_content = "hello";

    fixture.base.write("/hello.txt", ref_content).await?;
    fixture.overlay.write("/world.txt", "world").await?;

    let content = fixture.composite.read("/hello.txt").await?;
    let content = from_utf8(&content)?;

    assert_eq!(content, ref_content);
    Ok(())
}

#[tokio::test]
async fn write_base() -> anyhow::Result<()> {
    let fixture = TestFixture::new(BaseOnlyPolicy {})?;

    fixture.composite.write("/hello.txt", "hello").await?;
    assert_eq!(fixture.composite.entries("/").await?, &["hello.txt"]);

    Ok(())
}
