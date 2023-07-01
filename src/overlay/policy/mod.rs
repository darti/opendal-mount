mod naive;
mod os_files;

pub use naive::NaivePolicy;
pub use os_files::OsFilesPolicy;

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use opendal::raw::oio::{self, Page};
use opendal::raw::OpRead;
use opendal::raw::{Accessor, OpStat, RpRead, RpStat};

use opendal::Result;

use super::reader::OverlayReader;

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

    async fn read<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpRead,
    ) -> Result<(RpRead, OverlayReader<B::Reader, O::Reader>)>;

    async fn next<B: Page, O: Page>(
        &self,
        base: &mut B,
        overlay: &mut O,
    ) -> Result<Option<Vec<oio::Entry>>>;
}
