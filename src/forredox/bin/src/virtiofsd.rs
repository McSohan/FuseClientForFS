//! virtiofsd: scheme daemon exposing a virtio-fs-backed filesystem as a Redox scheme.
//
// Conceptual flow:
//  1. Probe virtio device via pcid_interface + virtio_core.
//  2. Create virtio-fs request queue and wrap it in VirtioFsTransport.
//  3. Build FuseProtocol on top of that transport.
//  4. Construct VirtiofsScheme<FuseTransport>.
//  5. Register scheme + event handle and enter event loop, similar to virtio-netd.

mod scheme;      // your VirtiofsScheme
mod protocol;    // your FuseProtocol<T>
mod transport;   // contains VirtioFsTransport
mod virtiofs;    // resource types etc.

use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::sync::Arc;

use redox_daemon::Daemon;
use redox_scheme::Socket;
use syscall::{self, Event};

use pcid_interface::PciFunctionHandle;
use virtio_core::spec::Queue;
use virtio_core::MSIX_PRIMARY_VECTOR;

use crate::protocol::FuseProtocol;
use crate::scheme::VirtiofsScheme;
use crate::transport::virtio::VirtioFsTransport;

/// This is the inner daemon body. It is intentionally close in structure to virtio-netd.
fn daemon(daemon: Daemon) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Probe the virtio-fs PCI function (device ID depends on your setup).
    let mut pcid_handle = PciFunctionHandle::connect_default();

    let pci_config = pcid_handle.config();

    // Check for virtio-fs (device ID 26 decimal = 0x1A)
    assert_eq!(
        pci_config.func.full_device_id.device_id,
        0x001A,
        "virtiofsd: incorrect PCI device, expected virtio-fs (device ID 0x1A)"
    );

    log::info!(
        "virtiofsd: found virtio-fs PCI device {}:{}",
        pci_config.func.full_device_id.vendor_id,
        pci_config.func.full_device_id.device_id
    );

    // Probe the virtio device
    let device = virtio_core::probe_device(&mut pcid_handle)?;
    let device_space = device.device_space;

    // Initialize features
    device.transport.finalize_features();
    device.transport.run_device();

    let fs_queue = device
        .transport
        .setup_queue(virtio_core::MSIX_PRIMARY_VECTOR, &device.irq_handle)?;


    let queue = Arc::new(fs_queue);
    let transport = VirtioFsTransport::new(queue);

    let proto = FuseProtocol::new(transport);

    let mut scheme = VirtiofsScheme::new(proto);

    let scheme_name = "virtiofs".to_string();

    // 5. Create a scheme socket bound to this name.
    //
    // This is the part where you mirror the pattern from redoxfs mount code.
    // The exact constructor may differ, so treat this as a structural template:
    let socket = Socket::create(&scheme_name, daemon.clone())
        .expect("virtiofsd: failed to create scheme socket");

    // 6. Register this scheme's event handle with /scheme/event, same as virtio-netd.
    let mut event_queue = File::open("/scheme/event")?;
    event_queue.write(&Event {
        id: socket.event_handle().raw(),
        flags: syscall::EVENT_READ,
        data: 0,
    })?;

    // Enter null namespace (same as virtio-netd).
    libredox::call::setrens(0, 0).expect("virtiofsd: failed to enter null namespace");

    // 7. Initial tick to drain any pending messages.
    socket
        .handle(&mut scheme)
        .expect("virtiofsd: initial handle() failed");

    // 8. Main event loop: wait on /scheme/event and dispatch to the scheme.
    loop {
        let mut ev_buf = [0u8; mem::size_of::<Event>()];
        event_queue.read(&mut ev_buf)?; // blocks until an event

        // Handle all pending scheme messages.
        socket
            .handle(&mut scheme)
            .expect("virtiofsd: scheme handle() failed");
    }
}

/// Wrapper needed by redox-daemon.
fn daemon_runner(daemon: Daemon) -> ! {
    daemon(daemon).unwrap();
    unreachable!();
}

pub fn main() {
    common::setup_logging(
        "fs",
        "pci",
        "virtiofsd",
        common::output_level(),
        common::file_level(),
    );
    redox_daemon::Daemon::new(daemon_runner).expect("virtiofsd: failed to daemonize");
}
