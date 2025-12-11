#![cfg(all(target_os = "redox", feature = "virtio-fs"))]

use std::io;
use std::sync::Arc;

use common::dma::Dma;
use virtio_core::spec::{Buffer, ChainBuilder, DescriptorFlags};
use virtio_core::transport::Queue;

use crate::transport::common::FuseTransport;

/// Upper bound on a single FUSE reply size.
/// You can tune this later once you know real limits.
const MAX_FUSE_MSG: usize = 64 * 1024;

pub struct VirtioFsTransport<'a> {
    queue: Arc<Queue<'a>>,
}

impl<'a> VirtioFsTransport<'a> {
    pub fn new(queue: Arc<Queue<'a>>) -> Self {
        Self { queue }
    }
}

impl<'a> FuseTransport for VirtioFsTransport<'a> {
    fn roundtrip(&mut self, req: &[u8]) -> io::Result<Vec<u8>> {
        // NOTE: This is a first-cut skeleton, not a complete virtio-fs spec implementation.

        let req_len = req.len();

        // 1. DMA for FUSE request
        let mut req_dma = unsafe {
            Dma::<[u8]>::zeroed_slice(req_len)
                .map_err(to_io_err)?
                .assume_init()
        };
        req_dma[..req_len].copy_from_slice(req);

        // 2. DMA for FUSE reply
        let mut resp_buf = unsafe {
            Dma::<[u8]>::zeroed_slice(MAX_FUSE_MSG)
                .map_err(to_io_err)?
                .assume_init()
        };

        // 3. Build a simple descriptor chain:
        //    [FUSE request][FUSE reply buffer (WRITE_ONLY)]
        let chain = ChainBuilder::new()
            .chain(Buffer::new_sized(&req_dma, req_len))
            .chain(Buffer::new_unsized(&resp_buf).flags(DescriptorFlags::WRITE_ONLY))
            .build();

        // 4. Submit and wait synchronously.
        let written = futures::executor::block_on(self.queue.send(chain)) as usize;

        if written < 4 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short FUSE reply from virtio-fs device",
            ));
        }

        // First 4 bytes are still the FUSE length field (same framing as Unix transport).
        let len_field =
            u32::from_le_bytes([resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3]]) as usize;

        if len_field > written {
            // For now, treat this as an error instead of trying to read another chain.
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "FUSE len larger than bytes written by device",
            ));
        }

        Ok(resp_buf[..len_field].to_vec())
    }
}

fn to_io_err<E: core::fmt::Debug>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("{e:?}"))
}
