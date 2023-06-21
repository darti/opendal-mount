use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use opendal::raw::{oio::Read, OpRead};
use opendal::raw::{Accessor, OpStat, RpRead, RpStat};

use opendal::Result;

pub enum Source {
    Base,
    Overlay,
}

#[async_trait]
pub trait Policy: Debug + Send + Sync + 'static {
    async fn stat<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpStat,
    ) -> Result<RpStat>;

    async fn read<B: Accessor, O: Accessor, R: Read>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpRead,
    ) -> Result<(RpRead, R)>;
}
