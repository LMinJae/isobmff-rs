use bytes::{Buf, BufMut, BytesMut};

pub trait IO {
    fn parse(r: &mut BytesMut) -> Self;
    fn as_bytes(self) -> BytesMut;
}

pub struct Box {
    pub box_type: u32,
    pub payload: BytesMut,
}

impl IO for Box {
    fn parse(mut r: &mut BytesMut) -> Self {
        let mut size = r.get_u32() as u64;
        let box_type = r.get_u32();
        if 1 == size {
            size = r.get_u64() - 8;
        }
        if 0 == size {
            Box {
                box_type,
                payload: r.split_to(r.len()),
            }
        } else {
            Box {
                box_type,
                payload: r.split_to((size as usize) - 8),
            }
        }
    }

    fn as_bytes(self) -> BytesMut {
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
        w.put(self.payload);

        w
    }
}

pub struct ftyp {
    pub major_brand: u32,
    pub minor_version: u32,
    pub compatible_brands: Vec<u32>,
}

impl IO for ftyp {
    fn parse(mut r: &mut BytesMut) -> Self {
        ftyp {
            major_brand: r.get_u32(),
            minor_version: r.get_u32(),
            compatible_brands: {
                let mut v = Vec::<u32>::with_capacity(r.len() / 4);
                while 0 < r.len() {
                    v.push(r.get_u32())
                }
                v
            }
        }
    }

    fn as_bytes(self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32(self.major_brand);
        w.put_u32(self.minor_version);
        for it in self.compatible_brands.iter() {
            w.put_u32(*it);
        }

        w
    }
}
