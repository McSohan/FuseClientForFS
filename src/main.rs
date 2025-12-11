use std::fs;
// use std::io::Write;
// use std::os::unix::net::UnixListener;
use std::path::Path;

use fuse_client_for_fs::protocol::FuseProtocol;
use fuse_client_for_fs::shell::commands::FuseShell;
use fuse_client_for_fs::transport::unix_socket::FuseListener;

/**
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
    // println!("flags = {}, bg={}, readahead={}", init.flags, init.max_background, init.max_readahead);
    let entry = proto.lookup(1, "hello.txt")?;
    println!("Found inode = {}", entry.nodeid);

    let attr = proto.getattr(entry.nodeid)?;
    println!("GETATTR: inode={} size={} mode={:o}",
        attr.attr.ino,
        attr.attr.size,
        attr.attr.mode
    );

    let open_out = proto.open(entry.nodeid, libc::O_RDONLY as u32)?;
    println!("FUSE_OPEN OK: fh={} flags={}",
        open_out.fh, open_out.open_flags
    );

    let data = proto.read(entry.nodeid, open_out.fh, 0, 1024)?;
    println!("File contents: {:?}", String::from_utf8_lossy(&data));

    proto.release(entry.nodeid, open_out.fh)?;


    Ok(())
}
**/

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

    let mut sh = FuseShell::new(proto)?;
    sh.run()?;

    Ok(())
}
