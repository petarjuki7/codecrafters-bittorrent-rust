#[repr(C)]
pub struct Handshake {
    pub len: u8,
    pub string: [u8; 19],
    pub reserved: [u8; 8],
    pub sha1_infohash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub unsafe fn as_u8_slice(&mut self) -> &[u8] {
        ::core::slice::from_raw_parts(
            (self as *const Handshake) as *const u8,
            ::core::mem::size_of::<Handshake>(),
        )
    }
}
