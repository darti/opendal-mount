use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use log::debug;
use opendal::raw::{
    oio::{self, Page},
    Accessor, OpRead, OpStat, RpRead, RpStat,
};

use crate::overlay::reader::OverlayReader;

use super::Policy;

const SPECIAL_FILES: [&str; 10] = [
    ".DS_Store",
    ".Spotlight-V100",
    ".VolumeIcon.icns",
    ".Trash",
    ".Trashes",
    ".fseventsd",
    "DCIM",
    ".metadata_never_index_unless_rootfs",
    ".metadata_never_index",
    ".metadata_direct_scope_only",
];
const SPECIAL_PREFIXES: [&str; 1] = ["._"];

#[derive(Debug, Clone)]
pub struct OsFilesPolicy;

impl OsFilesPolicy {
    fn is_special_file(path: &str) -> bool {
        Path::new(path).file_name().map_or(false, |f| {
            SPECIAL_FILES.iter().any(|sf| {
                sf == &f.to_str().unwrap() || SPECIAL_PREFIXES.iter().any(|f| path.starts_with(f))
            })
        })
    }
}

#[async_trait]
impl Policy for OsFilesPolicy {
    async fn stat<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpStat,
    ) -> opendal::Result<RpStat> {
        debug!("NaivePolicy::stat({:?}, {:?})", path, args);

        if OsFilesPolicy::is_special_file(path) {
            overlay.stat(path, args.clone()).await
        } else {
            base.stat(path, args).await
        }
    }

    async fn read<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpRead,
    ) -> opendal::Result<(RpRead, OverlayReader<B::Reader, O::Reader>)> {
        if OsFilesPolicy::is_special_file(path) {
            overlay
                .read(path, args.clone())
                .await
                .map(|(rp, r)| (rp, OverlayReader::Overlay(r)))
        } else {
            base.read(path, args)
                .await
                .map(|(rp, r)| (rp, OverlayReader::Base(r)))
        }
    }

    async fn next<B: Page, O: Page>(
        &self,
        base: &mut B,
        overlay: &mut O,
    ) -> opendal::Result<Option<Vec<oio::Entry>>> {
        let entries = overlay.next().await?;
        debug!(target : "NEXT", "NaivePolicy::next::overlay({:?})", entries);

        if let Some(_) = entries {
            debug!(target : "NEXT", "NaivePolicy::next::overlay({:?})", entries);
            return Ok(entries);
        } else {
            let entries = base.next().await;
            debug!(target : "NEXT", "NaivePolicy::next::base({:?})", entries);

            entries
        }
    }
}
