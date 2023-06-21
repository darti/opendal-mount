use std::{
    io,
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

pub enum OverlayBlockingReader<B: oio::BlockingRead, O: oio::BlockingRead> {
    Base(B),
    Overlay(O),
}

impl<B: oio::BlockingRead, O: oio::BlockingRead> oio::BlockingRead for OverlayBlockingReader<B, O> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Base(b) => b.read(buf),
            Self::Overlay(o) => o.read(buf),
        }
    }

    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64> {
        match self {
            Self::Base(b) => b.seek(pos),
            Self::Overlay(o) => o.seek(pos),
        }
    }

    fn next(&mut self) -> Option<Result<Bytes>> {
        match self {
            Self::Base(b) => b.next(),
            Self::Overlay(o) => o.next(),
        }
    }
}
