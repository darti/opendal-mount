mod os_files;

use log::debug;
pub use os_files::OsFilesPolicy;

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use opendal::raw::oio::{self, Page};
use opendal::raw::{
    Accessor, AccessorInfo, OpCreateDir, OpStat, OpWrite, RpCreateDir, RpRead, RpStat,
};
use opendal::raw::{OpRead, RpWrite};

use opendal::{Capability, Result};

use super::reader::OverlayReader;
use super::writer::OverlayWriter;

#[derive(Debug, Clone)]
pub enum Source {
    Base,
    Overlay,
}

#[derive(Debug, Clone)]
pub enum PolicyOperation {
    Stat(OpStat),
    Read(OpRead),
    Write(OpWrite),

    CreateDir(OpCreateDir),
}

impl From<OpStat> for PolicyOperation {
    fn from(op: OpStat) -> Self {
        PolicyOperation::Stat(op)
    }
}

impl From<OpRead> for PolicyOperation {
    fn from(op: OpRead) -> Self {
        PolicyOperation::Read(op)
    }
}

impl From<OpWrite> for PolicyOperation {
    fn from(op: OpWrite) -> Self {
        PolicyOperation::Write(op)
    }
}

impl From<OpCreateDir> for PolicyOperation {
    fn from(op: OpCreateDir) -> Self {
        PolicyOperation::CreateDir(op)
    }
}

#[async_trait]
pub trait Policy: Debug + Send + Sync + 'static {
    fn owner<B, O>(&self, base_info: B, overlay_info: O, path: &str, op: PolicyOperation) -> Source
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo;

    fn capability<B, O>(&self, base_info: B, overlay_info: O) -> Capability
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo;

    async fn stat<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpStat,
    ) -> Result<RpStat> {
        match self.owner(|| base.info(), || overlay.info(), path, args.clone().into()) {
            Source::Base => base.stat(path, args).await,
            Source::Overlay => overlay.stat(path, args).await,
        }
    }

    async fn read<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpRead,
    ) -> Result<(RpRead, OverlayReader<B::Reader, O::Reader>)> {
        match self.owner(|| base.info(), || overlay.info(), path, args.clone().into()) {
            Source::Base => base
                .read(path, args)
                .await
                .map(|(rp, r)| (rp, OverlayReader::Base(r))),
            Source::Overlay => overlay
                .read(path, args)
                .await
                .map(|(rp, r)| (rp, OverlayReader::Overlay(r))),
        }
    }

    async fn write<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpWrite,
    ) -> Result<(RpWrite, OverlayWriter<B::Writer, O::Writer>)> {
        let owner = self.owner(|| base.info(), || overlay.info(), path, args.clone().into());

        debug!("write to {:?}: path: {}, args: {:?}", owner, path, args);

        match owner {
            Source::Base => base
                .write(path, args)
                .await
                .map(|(rp, r)| (rp, OverlayWriter::Base(r))),
            Source::Overlay => overlay
                .write(path, args)
                .await
                .map(|(rp, r)| (rp, OverlayWriter::Overlay(r))),
        }
    }

    async fn create_dir<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpCreateDir,
    ) -> Result<RpCreateDir> {
        match self.owner(|| base.info(), || overlay.info(), path, args.clone().into()) {
            Source::Base => base.create_dir(path, args).await,
            Source::Overlay => overlay.create_dir(path, args).await,
        }
    }

    async fn next<B: Page, O: Page>(
        &self,
        base: &mut B,
        overlay: &mut O,
    ) -> Result<Option<Vec<oio::Entry>>>;
}
