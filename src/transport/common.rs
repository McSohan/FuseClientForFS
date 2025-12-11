use std::io;

pub trait FuseTransport {
    /// Send a complete FUSE request and receive a complete FUSE reply.
    ///
    /// `req` is already a fully-formed FUSE message:
    ///   [0..4]  = little-endian length (len field in FUSE header)
    ///   [4..]   = header + body
    ///
    /// The returned Vec MUST contain the entire reply in the same format.
    fn roundtrip(&mut self, req: &[u8]) -> io::Result<Vec<u8>>;
}
