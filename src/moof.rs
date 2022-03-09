use std::fmt::{Debug, Formatter};

use bytes::{Buf, BufMut, BytesMut};

use crate::{FullBox, IO, Object};

pub fn parse(r: &mut BytesMut) -> moof {
    moof::parse(r)
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub struct moof {
    mfhd: mfhd,
    trafs: Vec<traf>,
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

    sequence_number: u32,
}

impl mfhd {
    pub const BOX_TYPE: u32 = 0x6d666864;
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
#[derive(PartialEq)]
pub struct traf {
    tfhd: tfhd,
    truns: Vec<trun>,
}

impl traf {
    pub const BOX_TYPE: u32 = 0x74726166;
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
        f.write_fmt(format_args!("\t\t0x{:08x?}: \"tfhd\"", tfhd::BOX_TYPE))?;
        f.write_fmt(format_args!("\n{:?}", self.tfhd))?;

        for it in &self.truns {
            f.write_fmt(format_args!("\n\t\t0x{:08x?}: \"trun\"", trun::BOX_TYPE))?;
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
                tfhd::BOX_TYPE => {
                    rst.tfhd = tfhd::parse(&mut b.payload);
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
    track_id: u32,
    base_data_offset: Option<u64>,
    sample_description_index: Option<u32>,
    default_sample_duration: Option<u32>,
    default_sample_size: Option<u32>,
    default_sample_flags: Option<u32>,
}

impl tfhd {
    pub const BOX_TYPE: u32 = 0x74666864;
}

mod tfhd_flags {
    pub const BASE_DATA_OFFSET_PRESENT: u32 = 0x000001;
    pub const SAMPLE_DESCRIPTION_INDEX_PRESENT: u32 = 0x000002;
    pub const DEFAULT_SAMPLE_DURATION_PRESENT: u32 = 0x000008;
    pub const DEFAULT_SAMPLE_SIZE_PRESENT: u32 = 0x000010;
    pub const DEFAULT_SAMPLE_FLAG_PRESENT: u32 = 0x000020;
    pub const DURATION_IS_EMPTY: u32 = 0x010000;
    pub const DEFAULT_BASE_IS_MOOF: u32 = 0x020000;
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
            f.write_fmt(format_args!("\n\t\t\tdefault_sample_flags: {:?}", v))?;
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
#[derive(PartialEq)]
pub struct trun {
    base: FullBox,
    data_offset: Option<u32>,
    first_sample_flags: Option<u32>,
    samples: Vec<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)>,
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
            f.write_fmt(format_args!("\n\t\t\tfirst_sample_flags: {:?}", n))?;
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

#[cfg(test)]
mod tests {
    use crate::{FullBox, IO, Object};
    use crate::moof::{mfhd, moof, tfhd, traf, trun};

    #[test]
    fn chk_moof() {
        let mut b = moof {
            mfhd: mfhd {
                base: FullBox::new(0, 0),
                sequence_number: 1,
            },
            trafs: vec![
                traf {
                    tfhd: tfhd {
                        base: FullBox::new(0, 0x020000),
                        track_id: 1,
                        base_data_offset: None,
                        sample_description_index: None,
                        default_sample_duration: None,
                        default_sample_size: None,
                        default_sample_flags: None,
                    },
                    truns: vec![
                        trun {
                            base: FullBox::new(0, 0x000001 | 0x000004 | 0x000100 | 0x000200 | 0x000800),
                            data_offset: Some(520),
                            first_sample_flags: Some(0),
                            samples: vec![
                                (Some(3000), Some(9814), None, Some(0)),
                                (Some(1), Some(817), None, Some(3000)),
                                (Some(5999), Some(598), None, Some(0)),
                                (Some(1), Some(656), None, Some(3000)),
                                (Some(5999), Some(506), None, Some(0)),
                                (Some(1), Some(703), None, Some(3000)),
                                (Some(5999), Some(437), None, Some(0)),
                                (Some(1), Some(550), None, Some(3000)),
                                (Some(5999), Some(459), None, Some(0)),
                                (Some(1), Some(1008), None, Some(3150)),
                                (Some(6149), Some(431), None, Some(0)),
                                (Some(1), Some(723), None, Some(3000)),
                                (Some(5999), Some(475), None, Some(0)),
                                (Some(1), Some(607), None, Some(3000)),
                                (Some(5999), Some(509), None, Some(0)),
                                (Some(1), Some(680), None, Some(3000)),
                                (Some(5999), Some(428), None, Some(0)),
                                (Some(1), Some(584), None, Some(3000)),
                                (Some(5999), Some(473), None, Some(0)),
                                (Some(1), Some(891), None, Some(3000)),
                                (Some(5999), Some(421), None, Some(0)),
                                (Some(1), Some(636), None, Some(3000)),
                                (Some(2999), Some(440), None, Some(0)),
                                (Some(3000), Some(562), None, Some(3000)),
                            ],
                        }
                    ],
                },
                traf {
                    tfhd: tfhd {
                        base: FullBox { version: 0, flags: 0x000020 | 0x020000 },
                        track_id: 2,
                        base_data_offset: None,
                        sample_description_index: None,
                        default_sample_duration: None,
                        default_sample_size: None,
                        default_sample_flags: Some(33554432),
                    },
                    truns: vec![
                        trun {
                            base: FullBox::new(0, 0x000001 | 0x000200),
                            data_offset: Some(23928),
                            first_sample_flags: None,
                            samples: vec![
                                (Some(6), None, None, None),
                                (Some(169), None, None, None),
                                (Some(145), None, None, None),
                                (Some(24), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                                (Some(6), None, None, None),
                            ],
                        }
                    ],
                },
            ],
        };
        let mut obj = Object::parse(&mut Object {
            box_type: moof::BOX_TYPE,
            payload: b.as_bytes(),
        }.as_bytes());

        assert_eq!(moof::BOX_TYPE, obj.box_type);
        assert_eq!(b, moof::parse(&mut obj.payload));
    }
}
