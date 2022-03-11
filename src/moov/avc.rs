use std::fmt::{Debug, Formatter};

use bytes::{Buf, BufMut, BytesMut};

use crate::traits::IO;

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq)]
pub struct avcC {
    pub(crate) configuration_version: u8,
    pub(crate) profile_indication: u8,
    pub(crate) profile_compatibility: u8,
    pub(crate) level_indication: u8,
    pub(crate) length_size_minus_one: u8,
    pub(crate) sps: Vec<BytesMut>,
    pub(crate) pps: Vec<BytesMut>,

    pub(crate) ext: BytesMut,
}

impl Debug for avcC {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\t\t\t\t\t\t\t\tconfiguration_version: {:?}", self.configuration_version))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tprofile_indication: {:?}", self.profile_indication))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tprofile_compatibility: {:?}", self.profile_compatibility))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tlevel_indication: {:?}", self.level_indication))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tlength_size_minus_one: {:?}", self.length_size_minus_one))?;
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tnb_sps: {:?}", self.sps.len()))?;
        for it in &self.sps {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\t\t{:x?}", it))?;
        }
        f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\tnb_pps: {:?}", self.pps.len()))?;
        for it in &self.pps {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\t\t{:x?}", it))?;
        }
        if 0 < self.ext.len() {
            f.write_fmt(format_args!("\n\t\t\t\t\t\t\t\t\t\text: {:02x?}", self.ext.chunk()))?;
        }

        Ok(())
    }
}

impl IO for avcC {
    fn len(&self) -> usize {
        let mut v = 7 + self.ext.len();

        for it in &self.sps {
            v += 2 + it.len();
        }
        for it in &self.pps {
            v += 2 + it.len();
        }

        v
    }

    fn parse(r: &mut BytesMut) -> Self {
        let configuration_version = r.get_u8();
        let profile_indication = r.get_u8();
        let profile_compatibility = r.get_u8();
        let level_indication = r.get_u8();
        let length_size_minus_one = r.get_u8() & 0b11;

        let nb_sps = r.get_u8() & 0b11111;
        let mut sps = Vec::with_capacity(nb_sps as usize);
        for _ in 0..nb_sps {
            let len = r.get_u16();
            sps.push(r.split_to(len as usize));
        }

        let nb_pps = r.get_u8() & 0b11111;
        let mut pps = Vec::with_capacity(nb_sps as usize);
        for _ in 0..nb_pps {
            let len = r.get_u16();
            pps.push(r.split_to(len as usize));
        }

        Self {
            configuration_version,
            profile_indication,
            profile_compatibility,
            level_indication,
            length_size_minus_one,
            sps,
            pps,
            ext: r.split_to(r.len()),
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put_u8(self.configuration_version);
        w.put_u8(self.profile_indication);
        w.put_u8(self.profile_compatibility);
        w.put_u8(self.level_indication);
        w.put_u8((self.length_size_minus_one & 0b11) | 0b11111100);

        w.put_u8((self.sps.len() & 0b11111) as u8 | 0b11100000);
        for it in &self.sps {
            w.put_u16(it.len() as u16);
            w.put(it.chunk());
        }

        w.put_u8(self.pps.len() as u8);
        for it in &self.pps {
            w.put_u16(it.len() as u16);
            w.put(it.chunk());
        }

        w.put(self.ext.chunk());

        w
    }
}
