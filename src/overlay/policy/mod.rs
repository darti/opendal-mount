mod os_files;

use log::debug;
pub use os_files::OsFilesPolicy;

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use opendal::raw::oio::{self, Page};
use opendal::raw::{
    Accessor, AccessorInfo, OpAppend, OpCreateDir, OpStat, OpWrite, RpAppend, RpCreateDir, RpRead,
    RpStat,
};
use opendal::raw::{OpRead, RpWrite};

use opendal::{Capability, Result};

use super::appender::OverlayAppender;
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

    Append(OpAppend),
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

impl From<OpAppend> for PolicyOperation {
    fn from(op: OpAppend) -> Self {
        PolicyOperation::Append(op)
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

    async fn stat<B, O>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpStat,
    ) -> Result<RpStat>
    where
        B: Accessor,
        O: Accessor,
    {
        match self.owner(|| base.info(), || overlay.info(), path, args.clone().into()) {
            Source::Base => base.stat(path, args).await,
            Source::Overlay => overlay.stat(path, args).await,
        }
    }

    async fn read<B, O>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpRead,
    ) -> Result<(RpRead, OverlayReader<B::Reader, O::Reader>)>
    where
        B: Accessor,
        O: Accessor,
    {
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

    async fn write<B, O>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpWrite,
    ) -> Result<(RpWrite, OverlayWriter<B::Writer, O::Writer>)>
    where
        B: Accessor,
        O: Accessor,
    {
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

    async fn create_dir<B, O>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpCreateDir,
    ) -> Result<RpCreateDir>
    where
        B: Accessor,
        O: Accessor,
    {
        match self.owner(|| base.info(), || overlay.info(), path, args.clone().into()) {
            Source::Base => base.create_dir(path, args).await,
            Source::Overlay => overlay.create_dir(path, args).await,
        }
    }

    async fn append<B, O>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpAppend,
    ) -> Result<(RpAppend, OverlayAppender<B::Appender, O::Appender>)>
    where
        B: Accessor,
        O: Accessor,
    {
        match self.owner(|| base.info(), || overlay.info(), path, args.clone().into()) {
            Source::Base => base
                .append(path, args)
                .await
                .map(|(rp, b)| (rp, OverlayAppender::Base(b))),
            Source::Overlay => overlay
                .append(path, args)
                .await
                .map(|(rp, o)| (rp, OverlayAppender::Overlay(o))),
        }
    }

    async fn next<B, O>(&self, base: &mut B, overlay: &mut O) -> Result<Option<Vec<oio::Entry>>>
    where
        B: Page,
        O: Page;
}
