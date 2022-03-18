use std::fmt::{Debug, Formatter};

use bytes::{Buf, BufMut, BytesMut};

use crate::{FullBox, IO, Object};

pub fn parse(r: &mut BytesMut) -> moof {
    moof::parse(r)
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct moof {
    pub mfhd: mfhd,
    pub trafs: Vec<traf>,
}

impl moof {
    pub const BOX_TYPE: u32 = 0x6d6f6f66;
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
        f.write_fmt(format_args!("\t0x{:08x?}: \"mfhd\"", mfhd::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.mfhd))?;

        for it in &self.trafs {
            f.write_fmt(format_args!("\n\t0x{:08x?}: \"traf\"", traf::BOX_TYPE))?;
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for moof {
    fn len(&self) -> usize {
        let mut v = 8 + self.mfhd.len();

        for it in &self.trafs {
            v += 8 + it.len();
        }

        v
    }

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
                _ => {}
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: mfhd::BOX_TYPE,
            payload: self.mfhd.as_bytes(),
        }.as_bytes());

        for it in self.trafs.iter_mut() {
            w.put(Object {
                box_type: traf::BOX_TYPE,
                payload: it.as_bytes(),
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct mfhd {
    base: FullBox,

    pub sequence_number: u32,
}

impl mfhd {
    pub const BOX_TYPE: u32 = 0x6d666864;

    pub fn new(sequence_number: u32) -> Self {
        Self {
            base: FullBox::new(0, 0),
            sequence_number,
        }
    }
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
    fn len(&self) -> usize {
        self.base.len() + 4
    }

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
#[derive(PartialEq)]
pub struct traf {
    pub tfhd: tfhd,
    pub tfdt: Option<tfdt>,
    pub truns: Vec<trun>,
}

impl traf {
    pub const BOX_TYPE: u32 = 0x74726166;
}

impl Default for traf {
    fn default() -> Self {
        Self {
            tfhd: Default::default(),
            tfdt: None,
            truns: vec![],
        }
    }
}

impl Debug for traf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t0x{:08x?}: \"tfhd\"", tfhd::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.tfhd))?;

        if let Some(tfdt) = &self.tfdt {
            f.write_fmt(format_args!("\n\t\t0x{:08x?}: \"tfdt\"", tfdt::BOX_TYPE))?;
            f.write_fmt(format_args!("\n{:?}", tfdt))?;
        }

        for it in &self.truns {
            f.write_fmt(format_args!("\n\t\t0x{:08x?}: \"trun\"", trun::BOX_TYPE))?;
            f.write_fmt(format_args!("\n{:?}", it))?;
        }

        Ok(())
    }
}

impl IO for traf {
    fn len(&self) -> usize {
        let mut v = 8 + self.tfhd.len();

        if let Some(tfdt) = &self.tfdt {
            v += 8 + tfdt.len();
        }

        for it in &self.truns {
            v += 8 + it.len();
        }

        v
    }

    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self::default();

        while 0 < r.len() {
            let mut b = Object::parse(r);

            match b.box_type {
                // tfhd: Track Fragment Header
                tfhd::BOX_TYPE => {
                    rst.tfhd = tfhd::parse(&mut b.payload);
                }
                // tfdt: Track Fragment decode time
                tfdt::BOX_TYPE => {
                    rst.tfdt = Some(tfdt::parse(&mut b.payload));
                }
                // trun: Track Fragment Run
                trun::BOX_TYPE => {
                    rst.truns.push(trun::parse(&mut b.payload));
                }
                _ => {}
            }
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        w.put(Object {
            box_type: tfhd::BOX_TYPE,
            payload: self.tfhd.as_bytes(),
        }.as_bytes());

        if let Some(mut tfdt) = self.tfdt.clone() {
            w.put(Object {
                box_type: tfdt::BOX_TYPE,
                payload: tfdt.as_bytes(),
            }.as_bytes());
        }

        if let Some(trun) = self.truns.first_mut() {
            trun.base.flags = trun_flags::FIRST_SAMPLE_FLAGS_PRESENT;
        }

        for it in self.truns.iter_mut() {
            w.put(Object {
                box_type: trun::BOX_TYPE,
                payload: it.as_bytes(),
            }.as_bytes());
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct tfhd {
    base: FullBox,
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<u32>,
}

impl tfhd {
    pub const BOX_TYPE: u32 = 0x74666864;
}

#[allow(dead_code)]
mod tfhd_flags {
    pub(crate) const BASE_DATA_OFFSET_PRESENT: u32 = 0x000001;
    pub(crate) const SAMPLE_DESCRIPTION_INDEX_PRESENT: u32 = 0x000002;
    pub(crate) const DEFAULT_SAMPLE_DURATION_PRESENT: u32 = 0x000008;
    pub(crate) const DEFAULT_SAMPLE_SIZE_PRESENT: u32 = 0x000010;
    pub(crate) const DEFAULT_SAMPLE_FLAG_PRESENT: u32 = 0x000020;
    pub(crate) const DURATION_IS_EMPTY: u32 = 0x010000;
    pub(crate) const DEFAULT_BASE_IS_MOOF: u32 = 0x020000;
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
            base: FullBox::new(0, tfhd_flags::DEFAULT_BASE_IS_MOOF),
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
        f.write_fmt(format_args!("\t\t\tflags: 0x{:08x?}", self.base.flags))?;

        f.write_fmt(format_args!("\n\t\t\ttrack_id: {:?}", self.track_id))?;

        if let Some(v) = self.base_data_offset {
            f.write_fmt(format_args!("\n\t\t\tbase_data_offset: {:?}", v))?;
        }
        if let Some(v) = self.sample_description_index {
            f.write_fmt(format_args!("\n\t\t\tsample_description_index: {:?}", v))?;
        }
        if let Some(v) = self.default_sample_duration {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_duration: {:?}", v))?;
        }
        if let Some(v) = self.default_sample_size {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_size: {:?}", v))?;
        }
        if let Some(v) = self.default_sample_flags {
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_flags: 0x{:06x?}", v))?;
        }

        Ok(())
    }
}

impl IO for tfhd {
    fn len(&self) -> usize {
        let mut v = self.base.len() + 4;

        if let Some(_) = self.base_data_offset {
            v += 4;
        }
        if let Some(_) = self.sample_description_index {
            v += 4;
        }
        if let Some(_) = self.default_sample_duration {
            v += 4;
        }
        if let Some(_) = self.default_sample_size {
            v += 4;
        }
        if let Some(_) = self.default_sample_flags {
            v += 4;
        }

        v
    }

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

        if 0 != (rst.base.flags & tfhd_flags::BASE_DATA_OFFSET_PRESENT) {
            rst.base_data_offset = Some(r.get_u64());
        }
        if 0 != (rst.base.flags & tfhd_flags::SAMPLE_DESCRIPTION_INDEX_PRESENT) {
            rst.sample_description_index = Some(r.get_u32());
        }
        if 0 != (rst.base.flags & tfhd_flags::DEFAULT_SAMPLE_DURATION_PRESENT) {
            rst.default_sample_duration = Some(r.get_u32());
        }
        if 0 != (rst.base.flags & tfhd_flags::DEFAULT_SAMPLE_SIZE_PRESENT) {
            rst.default_sample_size = Some(r.get_u32());
        }
        if 0 != (rst.base.flags & tfhd_flags::DEFAULT_SAMPLE_FLAG_PRESENT) {
            rst.default_sample_flags = Some(r.get_u32());
        }

        rst
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        self.base.flags &= tfhd_flags::DEFAULT_BASE_IS_MOOF;    // Keeping base is moof
        if let Some(_) = self.base_data_offset {
            self.base.flags |= tfhd_flags::BASE_DATA_OFFSET_PRESENT;
        }
        if let Some(_) = self.sample_description_index {
            self.base.flags |= tfhd_flags::SAMPLE_DESCRIPTION_INDEX_PRESENT;
        }
        if let Some(_) = self.default_sample_duration {
            self.base.flags |= tfhd_flags::DEFAULT_SAMPLE_DURATION_PRESENT;
        }
        if let Some(_) = self.default_sample_size {
            self.base.flags |= tfhd_flags::DEFAULT_SAMPLE_SIZE_PRESENT;
        }
        if let Some(_) = self.default_sample_flags {
            self.base.flags |= tfhd_flags::DEFAULT_SAMPLE_FLAG_PRESENT;
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
#[derive(PartialEq, Clone)]
pub struct tfdt {
    base: FullBox,
    pub base_media_decode_time: u64,
}

impl tfdt {
    pub const BOX_TYPE: u32 = 0x74666474;

    pub fn new(base_media_decode_time: u64) -> Self {
        Self {
            base: FullBox::new(
                if (u32::MAX as u64) < base_media_decode_time { 1 } else { 0 },
                0,
            ),
            base_media_decode_time,
        }
    }
}

impl Default for tfdt {
    fn default() -> Self {
        Self {
            base: FullBox::new(1, 0),
            base_media_decode_time: 0,
        }
    }
}

impl Debug for tfdt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t\t\tbase_media_decode_time: {:?}", self.base_media_decode_time))?;

        Ok(())
    }
}

impl IO for tfdt {
    fn len(&self) -> usize {
        self.base.len() + if 1 == self.base.version { 8 } else { 4 }
    }

    fn parse(r: &mut BytesMut) -> Self {
        let base = FullBox::parse(r);
        let base_media_decode_time = if 1 == base.version {
            r.get_u64()
        } else {
            r.get_u32() as u64
        };

        Self {
            base,
            base_media_decode_time,
        }
    }

    fn as_bytes(&mut self) -> BytesMut {
        let mut w = BytesMut::new();

        //self.base.version = if (u32::MAX as u64) < self.base_media_decode_time { 1 } else { 0 };

        w.put(self.base.as_bytes());

        if 1 == self.base.version {
            w.put_u64(self.base_media_decode_time);
        } else {
            w.put_u32(self.base_media_decode_time as u32);
        }

        w
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct trun {
    base: FullBox,
    pub data_offset: Option<u32>,
    pub first_sample_flags: Option<u32>,
    pub samples: Vec<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)>,
}

impl trun {
    pub const BOX_TYPE: u32 = 0x7472756e;
}

mod trun_flags {
    pub const DATA_OFFSET_PRESENT: u32 = 0x000001;
    pub const FIRST_SAMPLE_FLAGS_PRESENT: u32 = 0x000004;
    pub const SAMPLE_DURATION_PRESENT: u32 = 0x000100;
    pub const SAMPLE_SIZE_PRESENT: u32 = 0x000200;
    pub const SAMPLE_FLAGS_PRESENT: u32 = 0x000400;
    pub const SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT: u32 = 0x000800;
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
        if let Some(n) = self.data_offset {
            f.write_fmt(format_args!("\n\t\t\tdata_offset: {:?}", n))?;
        }
        if let Some(n) = self.first_sample_flags {
            f.write_fmt(format_args!("\n\t\t\tfirst_sample_flags: 0x{:06x?}", n))?;
        }
        f.write_fmt(format_args!("\n\t\t\tsample_count: {:?}", self.samples.len()))?;
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
                f.write_fmt(format_args!("\n\t\t\t\t\tsample_flags: 0x{:06x?}", n))?;
            }
            if let Some(n) = sample_composition_time_offset {
                if 1 == self.base.version && (0 != (0x80000000 & n)) {
                    f.write_fmt(format_args!("\n\t\t\t\t\tsample_composition_time_offset: {:?}", -((0x7fffffff & n) as i32)))?;
                } else {
                    f.write_fmt(format_args!("\n\t\t\t\t\tsample_composition_time_offset: {:?}", n))?;
                }
            }
            f.write_fmt(format_args!("\n\t\t\t\t}}"))?;
        }
        f.write_fmt(format_args!("\n\t\t\t]"))?;

        Ok(())
    }
}

impl IO for trun {
    fn len(&self) -> usize {
        let mut v = self.base.len() + 4;

        if let Some(_) = self.data_offset {
            v += 4;
        }
        if let Some(_) = self.first_sample_flags {
            v += 4;
        }
        v += self.samples.len() * {
            let mut v = 0;

            if let Some((
                sample_duration,
                sample_size,
                sample_flags,
                sample_composition_time_offset,
            )) = self.samples.first() {
                if let Some(_) = *sample_duration {
                    v += 4;
                }
                if let Some(_) = *sample_size {
                    v += 4;
                }
                if let Some(_) = *sample_flags {
                    v += 4;
                }
                if let Some(_) = *sample_composition_time_offset {
                    v += 4;
                }
            }

            v
        };

        v
    }

    fn parse(r: &mut BytesMut) -> Self {
        let mut rst = Self {
            base: FullBox::parse(r),
            data_offset: None,
            first_sample_flags: None,
            samples: vec![],
        };

        let sample_count = r.get_u32();

        if 0 != (rst.base.flags & trun_flags::DATA_OFFSET_PRESENT) {
            rst.data_offset = Some(r.get_u32());
        }
        if 0 != (rst.base.flags & trun_flags::FIRST_SAMPLE_FLAGS_PRESENT) {
            rst.first_sample_flags = Some(r.get_u32());
        }

        for _ in 0..sample_count {
            let sample_duration = if 0 != (rst.base.flags & trun_flags::SAMPLE_DURATION_PRESENT) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_size = if 0 != (rst.base.flags & trun_flags::SAMPLE_SIZE_PRESENT) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_flags = if 0 != (rst.base.flags & trun_flags::SAMPLE_FLAGS_PRESENT) {
                Some(r.get_u32())
            } else {
                None
            };
            let sample_composition_time_offset = if 0 != (rst.base.flags & trun_flags::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT) {
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
            self.base.flags |= trun_flags::DATA_OFFSET_PRESENT;
        }
        if let Some(_) = self.first_sample_flags {
            self.base.flags |= trun_flags::FIRST_SAMPLE_FLAGS_PRESENT;
        }
        if let Some((
                        sample_duration,
                        sample_size,
                        sample_flags,
                        sample_composition_time_offset,
                    )) = self.samples.first() {
            if let Some(_) = *sample_duration {
                self.base.flags |= trun_flags::SAMPLE_DURATION_PRESENT;
            }
            if let Some(_) = *sample_size {
                self.base.flags |= trun_flags::SAMPLE_SIZE_PRESENT;
            }
            if let Some(_) = *sample_flags {
                self.base.flags |= trun_flags::SAMPLE_FLAGS_PRESENT;
            }
            if let Some(_) = *sample_composition_time_offset {
                self.base.flags |= trun_flags::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT;
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
            if 0 != self.base.flags & trun_flags::SAMPLE_DURATION_PRESENT {
                w.put_u32(if let Some(n) = sample_duration {
                    *n
                } else { 0 });
            }
            if 0 != self.base.flags & trun_flags::SAMPLE_SIZE_PRESENT {
                w.put_u32(if let Some(n) = sample_size {
                    *n
                } else { 0 });
            }
            if 0 != self.base.flags & trun_flags::SAMPLE_FLAGS_PRESENT {
                w.put_u32(if let Some(n) = sample_flags {
                    *n
                } else { 0 });
            }
            if 0 != self.base.flags & trun_flags::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT {
                w.put_u32(if let Some(n) = sample_composition_time_offset {
                    *n
                } else { 0 });
            }
        }

        w
    }
}

#[cfg(test)]
mod tests {
    use crate::{IO, Object};
    use crate::moof::{mfhd, moof, tfdt, traf, trun};

    #[test]
    fn chk_moof() {
        let mut b = moof {
            mfhd: {
                let mut v = mfhd::default();

                v.sequence_number = 1;

                v
            },
            trafs: vec![
                {
                    let mut v = traf::default();

                    v.tfhd.track_id = 1;
                    v.tfhd.default_sample_duration = Some(200);
                    v.tfhd.default_sample_size = Some(3815);
                    v.tfhd.default_sample_flags = Some(0);

                    v.tfdt = Some({
                        let mut v = tfdt::default();

                        v.base_media_decode_time = 60000;

                        v
                    });

                    v.truns.push({
                        let mut v = trun::default();

                        v.data_offset = Some(196);
                        v.samples.push((None, Some(3815), Some(0), None));
                        v.samples.push((None, Some(344), Some(0x810000), None));

                        v
                    });

                    v
                },
                {
                    let mut v = traf::default();

                    v.tfhd.track_id = 2;
                    v.tfhd.default_sample_duration = Some(33554432);
                    v.tfhd.default_sample_size = Some(12);
                    v.truns.push({
                        let mut v = trun::default();

                        v.data_offset = Some(23928);
                        v.samples.push((Some(6), None, None, None));
                        v.samples.push((Some(169), None, None, None));

                        v
                    });

                    v
                },
            ],
        };
        let mut obj = Object::parse(&mut Object {
            box_type: moof::BOX_TYPE,
            payload: b.as_bytes(),
        }.as_bytes());

        assert_eq!(moof::BOX_TYPE, obj.box_type);
        assert_eq!(b.len(), obj.payload.len());
        assert_eq!(b, moof::parse(&mut obj.payload));
    }
}
