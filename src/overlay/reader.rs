use std::{
    io::{self},
    task::{Context, Poll},
};

use bytes::Bytes;
use opendal::{raw::oio, Result};

pub enum OverlayReader<B: oio::Read, O: oio::Read> {
    Base(B),
    Overlay(O),
}

impl<B: oio::Read, O: oio::Read> oio::Read for OverlayReader<B, O> {
    fn poll_read(&mut self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        match self {
            Self::Base(b) => b.poll_read(cx, buf),
            Self::Overlay(o) => o.poll_read(cx, buf),
        }
    }

    fn poll_seek(&mut self, cx: &mut Context<'_>, pos: io::SeekFrom) -> Poll<Result<u64>> {
        match self {
            Self::Base(b) => b.poll_seek(cx, pos),
            Self::Overlay(o) => o.poll_seek(cx, pos),
        }
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes>>> {
        match self {
            Self::Base(b) => b.poll_next(cx),
            Self::Overlay(o) => o.poll_next(cx),
        }
    }
}
