use async_trait::async_trait;

use opendal::{
    raw::{Accessor, AccessorInfo, OpList, OpStat, Operation, RpList, RpStat},
    Builder, Capability, Operator, Scheme,
};

use crate::{
    appender::OverlayAppender, pager::OverlayPager, reader::OverlayReader, writer::OverlayWriter,
};

type SourcePolicy = fn(&str, Operator, Operator, Operation) -> opendal::Result<Operator>;

#[derive(Default, Debug)]
pub struct Overlay {
    overlay: Option<Operator>,
    base: Option<Operator>,
    policy: Option<SourcePolicy>,
}

impl Overlay {
    pub fn base(&mut self, remote: Operator) -> &mut Self {
        self.base = Some(remote);
        self
    }

    pub fn overlay(&mut self, overlay: Operator) -> &mut Self {
        self.overlay = Some(overlay);
        self
    }

    pub fn policy(&mut self, policy: SourcePolicy) -> &mut Self {
        self.policy = Some(policy);
        self
    }
}

impl Builder for Overlay {
    const SCHEME: opendal::Scheme = Scheme::Custom("overlay");

    type Accessor = OverlayBackend;

    fn from_map(map: std::collections::HashMap<String, String>) -> Self {
        todo!()
    }

    fn build(&mut self) -> opendal::Result<Self::Accessor> {
        let base = match self.base.take() {
            Some(base) => Ok(base),
            None => Err(opendal::Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "base is not specified",
            )),
        }?;

        let overlay = match self.overlay.take() {
            Some(overlay) => Ok(overlay),
            None => Err(opendal::Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "overlay is not specified",
            )),
        }?;

        let policy = match self.policy.take() {
            Some(policy) => Ok(policy),
            None => Err(opendal::Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "policy is not specified",
            )),
        }?;

        Ok(OverlayBackend {
            overlay,
            base,
            policy,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OverlayBackend {
    overlay: Operator,
    base: Operator,
    policy: SourcePolicy,
}

#[async_trait]
impl Accessor for OverlayBackend {
    type Reader = OverlayReader;
    type BlockingReader = OverlayReader;
    type Writer = OverlayWriter;
    type BlockingWriter = OverlayWriter;
    type Appender = OverlayAppender;
    type Pager = Option<OverlayPager>;
    type BlockingPager = Option<OverlayPager>;

    fn info(&self) -> AccessorInfo {
        let mut am = AccessorInfo::default();
        am.set_scheme(Scheme::Custom("overlay"))
            .set_capability(Capability {
                ..Default::default()
            });

        am
    }

    async fn stat(&self, path: &str, op: OpStat) -> opendal::Result<RpStat> {
        let operator = (self.policy)(
            path,
            self.overlay.clone(),
            self.base.clone(),
            Operation::Stat,
        )?;

        let meta = operator.stat(path).await?;

        Ok(RpStat::new(meta))
    }

    async fn list(&self, path: &str, args: OpList) -> opendal::Result<(RpList, Self::Pager)> {
        let operator = (self.policy)(
            path,
            self.overlay.clone(),
            self.base.clone(),
            Operation::List,
        )?;

        let list = operator.list(path).await?;

        Ok(list)
    }
}
