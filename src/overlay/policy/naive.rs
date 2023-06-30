use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, info};
use opendal::raw::{
    oio::{self, Page},
    Accessor, OpRead, OpStat, RpRead, RpStat,
};

use crate::overlay::reader::OverlayReader;

use super::Policy;

#[derive(Debug, Clone)]
pub struct NaivePolicy;

#[async_trait]
impl Policy for NaivePolicy {
    async fn stat<B: Accessor, O: Accessor>(
        &self,
        base: Arc<B>,
        overlay: Arc<O>,
        path: &str,
        args: OpStat,
    ) -> opendal::Result<RpStat> {
        debug!("NaivePolicy::stat({:?}, {:?})", path, args);

        if let Ok(o) = overlay.stat(path, args.clone()).await {
            Ok(o)
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
        if let Ok((rp, r)) = overlay.read(path, args.clone()).await {
            Ok((rp, OverlayReader::Overlay(r)))
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

        if let Some(_) = overlay.next().await? {
            debug!(target : "NEXT", "NaivePolicy::next::overlay({:?})", entries);
            return Ok(entries);
        } else {
            let entries = base.next().await;
            debug!(target : "NEXT", "NaivePolicy::next::base({:?})", entries);

            entries
        }
    }
}
