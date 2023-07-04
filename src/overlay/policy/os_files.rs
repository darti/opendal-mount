use std::path::Path;

use async_trait::async_trait;

use opendal::{
    raw::{
        oio::{self, Page},
        AccessorInfo,
    },
    Capability,
};

use super::{Policy, PolicyOperation, Source};

const SPECIAL_FILES: [&str; 11] = [
    ".DS_Store",
    ".Spotlight-V100",
    ".VolumeIcon.icns",
    ".Trash",
    ".Trashes",
    ".fseventsd",
    ".hidden",
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
    fn owner<B, O>(
        &self,
        _base_info: B,
        _overlay_info: O,
        path: &str,
        _op: PolicyOperation,
    ) -> Source
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo,
    {
        if OsFilesPolicy::is_special_file(path) {
            Source::Overlay
        } else {
            Source::Base
        }
    }

    fn capability<B, O>(&self, base_info: B, _overlay_info: O) -> Capability
    where
        B: FnOnce() -> AccessorInfo,
        O: FnOnce() -> AccessorInfo,
    {
        base_info().capability()
    }

    async fn next<B: Page, O: Page>(
        &self,
        base: &mut B,
        overlay: &mut O,
    ) -> opendal::Result<Option<Vec<oio::Entry>>> {
        let entries = overlay.next().await?;

        if let Some(_) = entries {
            return Ok(entries);
        } else {
            let entries = base.next().await;

            entries
        }
    }
}
