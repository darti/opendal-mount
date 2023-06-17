use async_trait::async_trait;

use opendal::{
    raw::{Accessor, AccessorInfo},
    Builder, Operator, Scheme,
};

use crate::{
    appender::OverlayAppender, pager::OverlayPager, reader::OverlayReader, writer::OverlayWriter,
};

#[derive(Default, Debug)]
pub struct OverlayBuilder {
    remote: Option<Operator>,
    overlay: Option<Operator>,
}

impl OverlayBuilder {
    pub fn remote(mut self, remote: Operator) -> Self {
        self.remote = Some(remote);
        self
    }

    pub fn overlay(mut self, overlay: Operator) -> Self {
        self.overlay = Some(overlay);
        self
    }
}

impl Builder for OverlayBuilder {
    const SCHEME: opendal::Scheme = Scheme::Custom("overlay");

    type Accessor = OverlayBackend;

    fn from_map(map: std::collections::HashMap<String, String>) -> Self {
        todo!()
    }

    fn build(&mut self) -> opendal::Result<Self::Accessor> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct OverlayBackend {
    remote: Operator,
    overlay: Operator,
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
        AccessorInfo::default()
    }
}
