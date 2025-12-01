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

    // create a transport layer 
    let transport = FuseListener::bind("/tmp/fuse.sock")?.accept()?;

    // currently the protocol layer just sends a message
    let mut proto = FuseProtocol::new(transport);
    let init = proto.send_init()?;
    println!("FUSE Initialized: major={} minor={}, congestion_threshold={}", init.major, init.minor, init.congestion_threshold);
    println!("flags = {}, bg={}, readahead={}", init.flags, init.max_background, init.max_readahead);


    Ok(())
}
