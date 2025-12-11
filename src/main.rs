use std::fs;
// use std::io::Write;
// use std::os::unix::net::UnixListener;
use std::path::Path;

use fuse_client_for_fs::protocol::FuseProtocol;
use fuse_client_for_fs::shell::commands::FuseShell;
use fuse_client_for_fs::transport::unix_socket::FuseListener;
use fuse_client_for_fs::virtiofs::VirtioFsImpl;

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/fuse.sock";

    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path)?;
    }

    let transport = FuseListener::bind(socket_path)?.accept()?;
    let mut proto = FuseProtocol::new(transport);

    let init = proto.send_init()?;
    println!(
        "[Debug] FUSE Initialized: major={} minor={}, congestion_threshold={}",
        init.major, init.minor, init.congestion_threshold
    );
    println!("FUSE Init complete, entering shell…");

    let vfs = VirtioFsImpl::new(proto);

    let mut sh = FuseShell::new(vfs);

    //let mut sh = FuseShell::new(proto)?;
    sh.run()?;

    Ok(())
}
