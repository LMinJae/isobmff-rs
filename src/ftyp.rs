use std::fmt::{Debug, Formatter};

use bytes::{Buf, BufMut, BytesMut};
use crate::types::types;

use crate::IO;

pub fn parse(r: &mut BytesMut) -> ftyp {
    ftyp::parse(r)
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct ftyp {
    pub major_brand: u32,
    pub minor_version: u32,
    pub compatible_brands: Vec<u32>,
}

impl ftyp {
    pub const BOX_TYPE: u32 = types::ftyp;
}

impl Debug for ftyp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\tmajor_brand:"))?;
        if let Ok(str) = std::str::from_utf8(&self.major_brand.to_be_bytes()) {
            f.write_fmt(format_args!(" {:?}", str))?;
        } else {
            f.write_fmt(format_args!(" 0x{:08?}", self.major_brand))?;
        }
        f.write_fmt(format_args!("\n\tminor_version: {:?}", self.minor_version))?;
        f.write_fmt(format_args!("\n\tcompatible_brands:"))?;
        for it in &self.compatible_brands {
            if let Ok(str) = std::str::from_utf8(&it.to_be_bytes()) {
                f.write_fmt(format_args!(" {:?}", str))?;
            } else {
                f.write_fmt(format_args!(" 0x{:08?}", it))?;
            }
        }

        Ok(())
    }
}

impl IO for ftyp {
    fn len(&self) -> usize {
        8 + 4 * self.compatible_brands.len()
    }

    fn parse(r: &mut BytesMut) -> Self {
        Self {
            major_brand: r.get_u32(),
            minor_version: r.get_u32(),
            compatible_brands: {
                let mut v = Vec::<u32>::with_capacity(r.len() / 4);
                while 0 < r.len() {
                    v.push(r.get_u32())
                }
                v
            },
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u32(self.major_brand);
        w.put_u32(self.minor_version);
        for it in &self.compatible_brands {
            w.put_u32(*it);
        }

        w
    }
}

#[cfg(test)]
mod tests {
    use crate::{IO, Object};
    use crate::ftyp::ftyp;

    #[test]
    fn chk_ftyp() {
        let mut b = ftyp {
            major_brand: 0x69736f35,
            minor_version: 1,
            compatible_brands: vec![
                0x61766331,
                0x69736f35,
                0x64617368,
            ],
        };
        let mut obj = Object::parse(&mut Object {
            box_type: ftyp::BOX_TYPE,
            payload: b.as_bytes(),
        }.as_bytes());

        assert_eq!(ftyp::BOX_TYPE, obj.box_type);
        assert_eq!(b.len(), obj.payload.len());
        assert_eq!(b, ftyp::parse(&mut obj.payload));
    }
}
