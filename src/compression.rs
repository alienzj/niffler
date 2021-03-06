/*
Copyright (c) 2018 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

/* standard use */
use std::io;

/* crates use */
use cfg_if::cfg_if;
use enum_primitive::{enum_from_primitive, enum_from_primitive_impl, enum_from_primitive_impl_ty};

use crate::error::Error;

enum_from_primitive! {
    #[repr(u64)]
    /// `Format` represent a compression format of a file. Currently Gzip, Bzip, Lzma or No are supported.
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Format {
        Gzip = 0x1F8B,
        Bzip = 0x425A,
        Lzma = 0x00FD_377A_585A,
        No,
    }
}

/// `Level` represent the compression level this value is include between 1 to 9.
/// 1 optimize the compression time,
/// 9 optimize the size of the output.
///
/// For bzip2:
///  - `One` is convert in `bzip2::Compression::Fastest`,
///  - `Nine` in `bzip2::Compression::Best`
/// and other value is convert in `bzip2::Compression::Default.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Level {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

pub(crate) fn get_first_five<'a>(
    mut in_stream: Box<dyn io::Read + 'a>,
) -> Result<([u8; 5], Box<dyn io::Read + 'a>), Error> {
    let mut buf = [0u8; 5];
    match in_stream.read_exact(&mut buf) {
        Ok(()) => Ok((buf, in_stream)),
        Err(_) => Err(Error::FileTooShort),
    }
}

cfg_if! {
    if #[cfg(feature = "gz")] {
        pub(crate) fn new_gz_encoder<'a>(out: Box<dyn io::Write + 'a>, level: Level) -> Result<Box<dyn io::Write + 'a>, Error> {
          Ok(Box::new(flate2::write::GzEncoder::new(
            out,
            level.into(),
          )))
        }

        pub(crate) fn new_gz_decoder<'a>(
            inp: Box<dyn io::Read + 'a>,
        ) -> Result<(Box<dyn io::Read + 'a>, Format), Error> {
          Ok((
            Box::new(flate2::read::MultiGzDecoder::new(inp)),
            Format::Gzip,
          ))
        }
    } else {
        pub(crate) fn new_gz_encoder<'a>(_: Box<dyn io::Write + 'a>, _: Level) -> Result<Box<dyn io::Write + 'a>, Error> {
            Err(Error::FeatureDisabled)
        }
        pub(crate) fn new_gz_decoder<'a>(_: Box<dyn io::Read + 'a>) -> Result<(Box<dyn io::Read + 'a>, Format), Error> {
            Err(Error::FeatureDisabled)
        }
}
}

cfg_if! {
    if #[cfg(feature = "bz2")] {
        pub(crate) fn new_bz2_encoder<'a>(out: Box<dyn io::Write + 'a>, level: Level) -> Result<Box<dyn io::Write + 'a>, Error> {
            Ok(Box::new(bzip2::write::BzEncoder::new(
                out,
                level.into(),
            )))
        }

        pub(crate) fn new_bz2_decoder<'a>(
            inp: Box<dyn io::Read + 'a>,
        ) -> Result<(Box<dyn io::Read + 'a>, Format), Error> {
            Ok((
                Box::new(bzip2::read::BzDecoder::new(inp)),
                Format::Bzip,
            ))
        }
    } else {
        pub(crate) fn new_bz2_encoder<'a>(_: Box<dyn io::Write + 'a>, _: Level) -> Result<Box<dyn io::Write + 'a>, Error> {
            Err(Error::FeatureDisabled)
        }

        pub(crate) fn new_bz2_decoder<'a>(_: Box<dyn io::Read + 'a>) -> Result<(Box<dyn io::Read + 'a>, Format), Error> {
            Err(Error::FeatureDisabled)
        }
    }
}

cfg_if! {
    if #[cfg(feature = "lzma")] {
      pub(crate) fn new_lzma_encoder<'a>(out: Box<dyn io::Write + 'a>, level: Level) -> Result<Box<dyn io::Write + 'a>, Error> {
          Ok(Box::new(xz2::write::XzEncoder::new(out, level.into())))
      }

      pub(crate) fn new_lzma_decoder<'a>(
          inp: Box<dyn io::Read + 'a>,
      ) -> Result<(Box<dyn io::Read + 'a>, Format), Error> {
          Ok((
              Box::new(xz2::read::XzDecoder::new(inp)),
              Format::Lzma,
          ))
      }
    } else {
      pub(crate) fn new_lzma_encoder<'a>(_: Box<dyn io::Write + 'a>, _: Level) -> Result<Box<dyn io::Write + 'a>, Error> {
          Err(Error::FeatureDisabled)
      }

      pub(crate) fn new_lzma_decoder<'a>(_: Box<dyn io::Read + 'a>) -> Result<(Box<dyn io::Read + 'a>, Format), Error> {
          Err(Error::FeatureDisabled)
      }
    }
}

impl Into<u32> for Level {
    fn into(self) -> u32 {
        match self {
            Level::One => 1,
            Level::Two => 2,
            Level::Three => 3,
            Level::Four => 4,
            Level::Five => 5,
            Level::Six => 6,
            Level::Seven => 7,
            Level::Eight => 8,
            Level::Nine => 9,
        }
    }
}

#[cfg(feature = "gz")]
impl Into<flate2::Compression> for Level {
    fn into(self) -> flate2::Compression {
        match self {
            Level::One => flate2::Compression::new(1),
            Level::Two => flate2::Compression::new(2),
            Level::Three => flate2::Compression::new(3),
            Level::Four => flate2::Compression::new(4),
            Level::Five => flate2::Compression::new(5),
            Level::Six => flate2::Compression::new(6),
            Level::Seven => flate2::Compression::new(7),
            Level::Eight => flate2::Compression::new(8),
            Level::Nine => flate2::Compression::new(9),
        }
    }
}

#[cfg(feature = "bz2")]
impl Into<bzip2::Compression> for Level {
    fn into(self) -> bzip2::Compression {
        match self {
            Level::One => bzip2::Compression::new(1),
            Level::Two => bzip2::Compression::new(2),
            Level::Three => bzip2::Compression::new(3),
            Level::Four => bzip2::Compression::new(4),
            Level::Five => bzip2::Compression::new(5),
            Level::Six => bzip2::Compression::new(6),
            Level::Seven => bzip2::Compression::new(7),
            Level::Eight => bzip2::Compression::new(8),
            Level::Nine => bzip2::Compression::new(9),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level2u32() {
        let mut tmp: u32 = Level::One.into();
        assert_eq!(tmp, 1);

        tmp = Level::Two.into();
        assert_eq!(tmp, 2);

        tmp = Level::Three.into();
        assert_eq!(tmp, 3);

        tmp = Level::Four.into();
        assert_eq!(tmp, 4);

        tmp = Level::Five.into();
        assert_eq!(tmp, 5);

        tmp = Level::Six.into();
        assert_eq!(tmp, 6);

        tmp = Level::Seven.into();
        assert_eq!(tmp, 7);

        tmp = Level::Eight.into();
        assert_eq!(tmp, 8);

        tmp = Level::Nine.into();
        assert_eq!(tmp, 9);
    }

    #[cfg(feature = "gz")]
    #[test]
    fn level2flate2() {
        let mut tmp: flate2::Compression = Level::One.into();
        assert_eq!(tmp, flate2::Compression::new(1));

        tmp = Level::Two.into();
        assert_eq!(tmp, flate2::Compression::new(2));

        tmp = Level::Three.into();
        assert_eq!(tmp, flate2::Compression::new(3));

        tmp = Level::Four.into();
        assert_eq!(tmp, flate2::Compression::new(4));

        tmp = Level::Five.into();
        assert_eq!(tmp, flate2::Compression::new(5));

        tmp = Level::Six.into();
        assert_eq!(tmp, flate2::Compression::new(6));

        tmp = Level::Seven.into();
        assert_eq!(tmp, flate2::Compression::new(7));

        tmp = Level::Eight.into();
        assert_eq!(tmp, flate2::Compression::new(8));

        tmp = Level::Nine.into();
        assert_eq!(tmp, flate2::Compression::new(9));
    }

    #[test]
    #[cfg(feature = "bz2")]
    fn level2bzip2() {
        let tmp: bzip2::Compression = Level::One.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(1).level());

        let tmp: bzip2::Compression = Level::Two.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(2).level());

        let tmp: bzip2::Compression = Level::Three.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(3).level());

        let tmp: bzip2::Compression = Level::Four.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(4).level());

        let tmp: bzip2::Compression = Level::Five.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(5).level());

        let tmp: bzip2::Compression = Level::Six.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(6).level());

        let tmp: bzip2::Compression = Level::Seven.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(7).level());

        let tmp: bzip2::Compression = Level::Eight.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(8).level());

        let tmp: bzip2::Compression = Level::Nine.into();
        assert_eq!(tmp.level(), bzip2::Compression::new(9).level());
    }
}
