//! Kernel-side VirtioFS PCI device driver

use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

use pcid_interface::PciFunctionHandle;

use common::dma::Dma;
use syscall::error::{Error, Result, EINVAL};

use virtio_core::transport::{Queue};
use virtio_core::spec::{DeviceStatus, DescriptorFlags, Buffer, ChainBuilder};

pub const VIRTIO_FS_DEVICE_ID: u16 = 0x105A; // official virtio-fs ID
const MAX_FUSE_MSG: usize = 128 * 1024;

pub struct VirtioFsDevice<'a> {
    pub queue: Arc<Queue<'a>>,
    request_id: AtomicUsize,
}

impl<'a> VirtioFsDevice<'a> {
    pub fn send_request(&self, req: &[u8], resp: &mut [u8]) -> Result<usize> {
        let _rid = self.request_id.fetch_add(1, Ordering::SeqCst);

        let mut req_dma = unsafe {
            Dma::<[u8]>::zeroed_slice(req.len()).map_err(to_error)?.assume_init()
        };
        req_dma[..req.len()].copy_from_slice(req);

        let mut resp_dma = unsafe {
            Dma::<[u8]>::zeroed_slice(resp.len()).map_err(to_error)?.assume_init()
        };

        let chain = ChainBuilder::new()
            .chain(Buffer::new_sized(&req_dma, req.len()))
            .chain(Buffer::new_unsized(&resp_dma).flags(DescriptorFlags::WRITE_ONLY))
            .build();

        // async wait for IRQ-based completion
        let written = futures::executor::block_on(self.queue.send(chain));

        if written <= 0 {
            return Err(Error::new(EINVAL));
        }

        resp[..written as usize].copy_from_slice(&resp_dma[..written as usize]);
        Ok(written as usize)
    }
}

fn to_error<E: core::fmt::Debug>(_: E) -> Error {
    Error::new(EINVAL)
}

/// Probe PCI and initialize virtio transport for VirtioFS
pub fn virtiofs_probe() -> Result<VirtioFsDevice<'static>> {
    let mut pci = PciFunctionHandle::connect_default();
    let cfg = pci.config();

    if cfg.func.full_device_id.device_id != VIRTIO_FS_DEVICE_ID {
        return Err(Error::new(EINVAL));
    }

    log::info!("virtiofs: PCI device detected");

    let mut dev = virtio_core::probe_device(&mut pci)
        .map_err(|_| Error::new(EINVAL))?;

    let transport = &mut dev.transport;

    // Negotiation
    let f = transport.driver_features();
    transport.ack_driver_features(f);
    transport.finalize_features();

    let queue =
        transport.setup_queue(virtio_core::MSIX_PRIMARY_VECTOR, &dev.irq_handle)
            .map_err(|_| Error::new(EINVAL))?;

    transport.run_device();

    Ok(VirtioFsDevice {
        queue,
        request_id: AtomicUsize::new(1),
    })
}
