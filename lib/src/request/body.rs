use std::io::{self, Read, Write};

use hyper;
use tokio_io::AsyncRead;
use futures::{Async, Poll, Stream};

pub struct Body {
    body: hyper::Body,

    // Used as a buffer when reading the body through `tokio_io::AsyncRead`. This should
    // hopefully become unneccessary when `hyper::Body` internally
    // implements `tokio_io::AsyncRead`.
    chunk: Option<(hyper::Chunk, usize)>,
}

impl Body {
    pub(crate) fn new(body: hyper::Body) -> Self {
        Body { body, chunk: None }
    }
}

impl Default for Body {
    fn default() -> Self {
        Body::new(Default::default())
    }
}

fn read_from_chunk(
    body: &mut Body,
    chunk: hyper::Chunk,
    mut buf: &mut [u8],
    index: usize,
) -> io::Result<usize> {
    let written = buf.write(&chunk[index..])?;

    body.chunk = if index + written < chunk.len() {
        Some((chunk, index + written))
    } else {
        None
    };

    Ok(written)
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some((chunk, index)) = self.chunk.take() {
            return read_from_chunk(self, chunk, buf, index);
        }

        match self.body.poll() {
            Ok(Async::Ready(chunk)) => Ok(match chunk {
                Some(chunk) => read_from_chunk(self, chunk, buf, 0)?,
                None => 0,
            }),

            Ok(Async::NotReady) => Err(io::ErrorKind::WouldBlock.into()),
            Err(error) => match error {
                hyper::Error::Io(error) => Err(error),
                _ => Err(io::Error::new(io::ErrorKind::Other, Box::new(error))),
            },
        }
    }
}

impl AsyncRead for Body {}

impl Stream for Body {
    type Item = hyper::Chunk;
    type Error = hyper::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Option<hyper::Chunk>, hyper::Error> {
        self.body.poll()
    }
}
