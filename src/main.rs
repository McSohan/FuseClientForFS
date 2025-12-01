use std::fs;
// use std::io::Write;
// use std::os::unix::net::UnixListener;
use std::path::Path;

use fuse_client_for_fs::transport::unix_socket::FuseListener;
use fuse_client_for_fs::protocol::FuseProtocol;

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/fuse.sock";
    let path = Path::new(socket_path);

    // Clean up any stale socket
    if path.exists() {
        fs::remove_file(path)?;
    }

    let transport = FuseListener::bind("/tmp/fuse.sock")?.accept()?;
    let mut proto = FuseProtocol::new(transport);
    proto.send_init()?;

    Ok(())
}
