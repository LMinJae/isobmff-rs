use std::fmt::{Debug, Formatter};
use bytes::{Buf, BufMut, BytesMut};
use crate::{FullBox, IO, Object};

pub fn parse(r: &mut BytesMut) -> moof {
    moof::parse(r)
}

#[allow(non_camel_case_types)]
pub struct moof {
    mfhd: mfhd,
    trafs: Vec<traf>,
}

impl Default for moof {
    fn default() -> Self {
        Self {
            mfhd: Default::default(),
            trafs: vec![],
        }
    }
}

impl Debug for moof {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t0x6d666864: \"mfhd\""))?;
        f.write_fmt(format_args!("\n{:?}", self.mfhd))?;

        for it in &self.trafs {
            f.write_fmt(format_args!("\n\t0x74726166: \"traf\""))?;
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for moof {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // mfhd: Movie Fragment Header
                0x6d666864 => {
                    rst.mfhd = mfhd::parse(&mut b.payload);
                }
                // traf: Track Fragment
                0x74726166 => {
                    rst.trafs.push(traf::parse(&mut b.payload));
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: 0x6d666864,
            payload: self.mfhd.as_bytes(),
        }.as_bytes());

        for it in self.trafs.iter_mut() {
            w.put(Object {
                box_type: 0x74726166,
                payload: it.as_bytes(),
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct mfhd {
    base: FullBox,

    sequence_number: u32,
}

impl Default for mfhd {
    //! extends FullBox(‘mfhd’, 0, 0){
    //!     unsigned int(32) sequence_number;
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            sequence_number: 0,
        }
    }
}

impl Debug for mfhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\tsequence_number: {:?}", self.sequence_number))?;

        Ok(())
    }
}

impl IO for mfhd {
    fn parse(r: &mut BytesMut) -> Self {
        Self {
            base: FullBox::parse(r),
            sequence_number: r.get_u32(),
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(self.base.as_bytes());

        w.put_u32(self.sequence_number);

        w
    }
}

#[allow(non_camel_case_types)]
pub struct traf {
    tfhd: tfhd,
    truns: Vec<trun>,
}

impl Default for traf {
    fn default() -> Self {
        Self {
            tfhd: Default::default(),
            truns: vec![],
        }
    }
}

impl Debug for traf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t0x74666864: \"tfhd\""))?;
        f.write_fmt(format_args!("\n{:?}", self.tfhd))?;

        for it in &self.truns {
            f.write_fmt(format_args!("\n\t\t0x7472756e: \"trun\""))?;
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for traf {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // tfhd: Track Fragment Header
                0x74666864 => {
                    rst.tfhd = tfhd::parse(&mut b.payload);
                }
                // trun: Track Fragment Run
                0x7472756e => {
                    rst.truns.push(trun::parse(&mut b.payload));
                }
                _ => {
                }
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: 0x74666864,
            payload: self.tfhd.as_bytes()
        }.as_bytes());

        for it in self.truns.iter_mut() {
            w.put(Object {
                box_type: 0x7472756e,
                payload: it.as_bytes()
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct tfhd {
    base: FullBox,
    track_id: u32,
    base_data_offset: Option<u64>,
    sample_description_index: Option<u32>,
    default_sample_duration: Option<u32>,
    default_sample_size: Option<u32>,
    default_sample_flags: Option<u32>,
}

impl Default for tfhd {
    //! extends FullBox(‘tfhd’, 0, tf_flags){
    //!     unsigned int(32) track_ID;
    //!     // all the following are optional fields
    //!     unsigned int(64) base_data_offset;
    //!     unsigned int(32) sample_description_index;
    //!     unsigned int(32) default_sample_duration;
    //!     unsigned int(32) default_sample_size;
    //!     unsigned int(32) default_sample_flags
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            track_id: 0,
            base_data_offset: None,
            sample_description_index: None,
            default_sample_duration: None,
            default_sample_size: None,
            default_sample_flags: None,
        }
    }
}

impl Debug for tfhd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tflags: {:?}", self.base.flags))?;

        f.write_fmt(format_args!("\n\t\t\ttrack_id: {:?}", self.track_id))?;

        if 0 != (0x000001 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tbase_data_offset: {:?}", self.base_data_offset))?;
        }
        if 0 != (0x000002 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tsample_description_index: {:?}", self.sample_description_index))?;
        }
        if 0 != (0x000008 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_duration: {:?}", self.default_sample_duration))?;
        }
        if 0 != (0x000010 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_size: {:?}", self.default_sample_size))?;
        }
        if 0 != (0x000020 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_flags: {:?}", self.default_sample_flags))?;
        }

        Ok(())
    }
}

impl IO for tfhd {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            track_id: r.get_u32(),
            base_data_offset: None,
            sample_description_index: None,
            default_sample_duration: None,
            default_sample_size: None,
            default_sample_flags: None,
        };

        if 0 != (0x000001 & rst.base.flags) {
            rst.base_data_offset = Some(r.get_u64());
        }
        if 0 != (0x000002 & rst.base.flags) {
            rst.sample_description_index = Some(r.get_u32());
        }
        if 0 != (0x000008 & rst.base.flags) {
            rst.default_sample_duration = Some(r.get_u32());
        }
        if 0 != (0x000010 & rst.base.flags) {
            rst.default_sample_size = Some(r.get_u32());
        }
        if 0 != (0x000020 & rst.base.flags) {
            rst.default_sample_flags = Some(r.get_u32());
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        self.base.flags = 0;
        if let Some(_) = self.base_data_offset {
            self.base.flags |= 0x000001;
        }
        if let Some(_) = self.sample_description_index {
            self.base.flags |= 0x000002;
        }
        if let Some(_) = self.default_sample_duration {
            self.base.flags |= 0x000008;
        }
        if let Some(_) = self.default_sample_size {
            self.base.flags |= 0x000010;
        }
        if let Some(_) = self.default_sample_flags {
            self.base.flags |= 0x000020;
        }

        w.put(self.base.as_bytes());

        w.put_u32(self.track_id);

        if let Some(v) = self.base_data_offset {
            w.put_u64(v);
        }
        if let Some(v) = self.sample_description_index {
            w.put_u32(v);
        }
        if let Some(v) = self.default_sample_duration {
            w.put_u32(v);
        }
        if let Some(v) = self.default_sample_size {
            w.put_u32(v);
        }
        if let Some(v) = self.default_sample_flags {
            w.put_u32(v);
        }

        w
    }
}

#[allow(non_camel_case_types)]
pub struct trun {
    base: FullBox,
    data_offset: Option<u32>,
    first_sample_flags: Option<u32>,
    samples: Vec<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)>
}

impl Default for trun {
    //! extends FullBox(‘trun’, 0, tr_flags) {
    //!     unsigned int(32) sample_count;
    //!     // the following are optional fields
    //!     signed int(32) data_offset;
    //!     unsigned int(32) first_sample_flags;
    //!     // all fields in the following array are optional
    //!     {
    //!         unsigned int(32) sample_duration;
    //!         unsigned int(32) sample_size;
    //!         unsigned int(32) sample_flags
    //!         unsigned int(32) sample_composition_time_offset;
    //!     }[ sample_count ]
    //! }
    fn default() -> Self {
        Self {
            base: FullBox::new(0, 0),
            data_offset: None,
            first_sample_flags: None,
            samples: vec![],
        }
    }
}

impl Debug for trun {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tflags: 0x{:08x?}", self.base.flags))?;

        f.write_fmt(format_args!("\n\t\t\tsample_count: {:?}", self.samples.len()))?;
        if 0 != (0x000001 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tdata_offset: {:?}", self.data_offset))?;
        }
        if 0 != (0x000004 & self.base.flags) {
            f.write_fmt(format_args!("\n\t\t\tfirst_sample_flags: {:?}", self.first_sample_flags))?;
        }

        f.write_fmt(format_args!("\n\t\t\t["))?;
        for (
            sample_duration,
            sample_size,
            sample_flags,
            sample_composition_time_offset,
        ) in &self.samples {
            f.write_fmt(format_args!("\n\t\t\t\t{{"))?;
            if let Some(n) = sample_duration {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_duration: {:?}", n))?;
            }
            if let Some(n) = sample_size {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_size: {:?}", n))?;
            }
            if let Some(n) = sample_flags {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_flags: {:?}", n))?;
            }
            if let Some(n) = sample_composition_time_offset {
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_composition_time_offset: {:?}", n))?;
            }
            f.write_fmt(format_args!("\n\t\t\t\t}}"))?;
        }
        f.write_fmt(format_args!("\n\t\t\t]"))?;

        Ok(())
    }
}

impl IO for trun {
    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            data_offset: None,
            first_sample_flags: None,
            samples: vec![]
        };

        let sample_count = r.get_u32();

        if 0 != (0x000001 & rst.base.flags) {
            rst.data_offset = Some(r.get_u32());
        }
        if 0 != (0x000004 & rst.base.flags) {
            rst.first_sample_flags = Some(r.get_u32());
        }

        for _ in 0..sample_count {
            let sample_duration = if 0 != (0x000100 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_size = if 0 != (0x000200 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_flags = if 0 != (0x000400 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_composition_time_offset = if 0 != (0x000800 & rst.base.flags) {
                Some(r.get_u32())
            } else {
                None
            };

            rst.samples.push((sample_duration, sample_size, sample_flags, sample_composition_time_offset));
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        self.base.flags = 0;
        if let Some(_) = self.data_offset {
            self.base.flags |= 0x000001;
        }
        if let Some(_) = self.first_sample_flags {
            self.base.flags |= 0x000004;
        }
        if let Some((
                        sample_duration,
                        sample_size,
                        sample_flags,
                        sample_composition_time_offset,
                    )) = self.samples.first() {
            if let Some(_) = *sample_duration {
                self.base.flags |= 0x000100;
            }
            if let Some(_) = *sample_size {
                self.base.flags |= 0x000200;
            }
            if let Some(_) = *sample_flags {
                self.base.flags |= 0x000400;
            }
            if let Some(_) = *sample_composition_time_offset {
                self.base.flags |= 0x000800;
            }
        }

        w.put(self.base.as_bytes());

        w.put_u32(self.samples.len() as u32);

        if let Some(v) = self.data_offset {
            w.put_u32(v);
        }

        if let Some(v) = self.first_sample_flags {
            w.put_u32(v);
        }

        for (
            sample_duration,
            sample_size,
            sample_flags,
            sample_composition_time_offset,
        ) in &self.samples {
            if let Some(n) = sample_duration {
                w.put_u32(*n);
            }
            if let Some(n) = sample_size {
                w.put_u32(*n);
            }
            if let Some(n) = sample_flags {
                w.put_u32(*n);
            }
            if let Some(n) = sample_composition_time_offset {
                w.put_u32(*n);
            }
        }

        w
    }
}
