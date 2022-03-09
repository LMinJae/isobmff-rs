use bytes::{Buf, BufMut, BytesMut};
use crate::traits::IO;

// An object in this terminology is a box.
// but, Rust has same name Box for Heap allocation related
pub struct Object {
    pub box_type: u32,
    pub payload: BytesMut,
}

impl IO for Object {
    fn parse(r: &mut BytesMut) -> Self {
        let mut size = r.get_u32() as u64;
        let box_type = r.get_u32();
        if 1 == size {
            size = r.get_u64() - 8;
        }
        if 0 == size {
            Self {
                box_type,
                payload: r.split_to(r.len()),
            }
        } else {
            Self {
                box_type,
                payload: r.split_to((size as usize) - 8),
            }
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        let size = 4 + self.payload.len();
        /*  */ if (u32::MAX as usize) < size {
            w.put_u32(1);
        } else if (u64::MAX as usize) < size {
            w.put_u32(0);
        }
        w.put_u32(self.box_type);
        /*  */ if (u32::MAX as usize) < size && size <= (u64::MAX as usize) {
            w.put_u64(size as u64);
        }
        w.put(self.payload.chunk());

        w
    }
}

#[derive(Clone)]
pub struct FullBox {
    pub(crate) version: u8,
    pub(crate) flags: u32,
}

impl FullBox {
    pub fn new(version: u8, flags: u32) -> Self {
        Self {
            version,
            flags,
        }
    }
}

impl IO for FullBox {
    fn parse(r: &mut BytesMut) -> Self {
        let t = r.get_u32();

        Self {
            version: (t >> 24) as u8,
            flags: t & 0x00FFFFFF,
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32((self.version as u32) << 24 | self.flags);

        w
    }
}
