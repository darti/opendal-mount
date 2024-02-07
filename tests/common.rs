use async_trait::async_trait;
use ctor::ctor;

use opendal::{services::Fs, Operator};

use tempfile::TempDir;

#[ctor]
fn init() {
    #[cfg(feature = "tracing")]
    console_subscriber::init();

    #[cfg(not(feature = "tracing"))]
    pretty_env_logger::init();
}

#[async_trait]
pub trait ListDir {
    async fn entries(&self, path: &str) -> anyhow::Result<Vec<String>>;
}

#[async_trait]
impl ListDir for Operator {
    async fn entries(&self, path: &str) -> anyhow::Result<Vec<String>> {
        let ds = self.list(path).await?;
        let mut entries: Vec<String> = ds.iter().map(|e| e.name().to_owned()).collect();

        entries.sort();

        Ok(entries)
    }
}

pub struct TestFixture {
    pub root: TempDir,
    pub base: Operator,
}

impl TestFixture {
    pub fn new() -> anyhow::Result<Self> {
        let root: tempfile::TempDir = tempfile::tempdir()?;
        let base_root = root.path().join("base");

        let base = {
            let mut builder = Fs::default();
            builder.root(base_root.to_str().unwrap());

            Operator::new(builder)?.finish()
        };

        Ok(Self { root, base })
    }
}
