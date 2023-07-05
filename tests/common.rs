use ctor::ctor;
use opendal::{
    services::{Fs, Memory},
    Operator,
};
use opendal_mount::{
    overlay::{self, policy::Policy},
    Overlay,
};
use tempfile::TempDir;

#[ctor]
fn init() {
    #[cfg(feature = "tracing")]
    console_subscriber::init();

    #[cfg(not(feature = "tracing"))]
    pretty_env_logger::init();
}

pub struct TestFixture {
    pub root: TempDir,
    pub base: Operator,
    pub overlay: Operator,
    pub composite: Operator,
}

impl TestFixture {
    pub fn new<P>(policy: P) -> anyhow::Result<Self>
    where
        P: Policy,
    {
        let root: tempfile::TempDir = tempfile::tempdir()?;
        let base_root = root.path().join("base");
        let overlay_root = root.path().join("overlay");

        let composite = {
            let mut base_builder = Fs::default();
            base_builder.root(base_root.to_str().unwrap());

            let mut overlay_builder = Fs::default();
            overlay_builder.root(overlay_root.to_str().unwrap());

            let mut builder = Overlay::default();
            builder
                .base_builder(base_builder)
                .overlay_builder(overlay_builder)
                .policy(policy);

            Operator::new(builder)?.finish()
        };

        let base = {
            let mut builder = Fs::default();
            builder.root(base_root.to_str().unwrap());

            Operator::new(builder)?.finish()
        };

        let overlay = {
            let mut builder = Fs::default();
            builder.root(overlay_root.to_str().unwrap());

            Operator::new(builder)?.finish()
        };

        Ok(Self {
            root,
            base,
            overlay,
            composite,
        })
    }
}
