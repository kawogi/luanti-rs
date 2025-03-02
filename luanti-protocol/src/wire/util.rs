//!
//! The crazy exotic serialization methods Luanti uses
//!

use std::str::FromStr;

use anyhow::Result;
use anyhow::bail;
use miniz_oxide::inflate;
use miniz_oxide::inflate::core::DecompressorOxide;
use miniz_oxide::inflate::core::inflate_flags;
use zstd_safe::InBuffer;
use zstd_safe::OutBuffer;

/// Convert an integer type into it's string representation as &[u8]
///
/// For example:
///    123 returns &[49, 50, 51]
///   -100 returns &[45, 49, 48, 48]
///
#[macro_export]
macro_rules! itos {
    ($n: expr) => {
        &($n).to_string().into_bytes()
    };
}

/// Parse byte slice into an integer. The opposite of itos.
/// On error (such as `Utf8Error` or `ParseIntError`) this does
/// `return Err()` implicitly.
///
/// Use return type-inference to specify the integer type, e.g:
///
/// ```rust
/// use luanti_protocol::wire::util::stoi;
/// let val: u16 = stoi(b"123".as_slice()).unwrap();
/// ```
pub fn stoi<T: FromStr>(bytes: &[u8]) -> Result<T>
where
    <T as FromStr>::Err: std::error::Error + Sync + Send + 'static,
{
    let str = std::str::from_utf8(bytes)?;
    Ok(str.parse::<T>()?)
}
/*
#[macro_export]
macro_rules! stoi {
    ($b: expr, $typ: ty) => {{
        let result: anyhow::Result<$typ> = match std::str::from_utf8($b) {
            Ok(v) => match v.parse::<$typ>() {
                Ok(v) => Ok(v),
                Err(e) => Err(anyhow::Error::from(e)),
            },
            Err(e) => Err(anyhow::Error::from(e)),
        };
        result
    }};
}
*/

///
/// Streaming Zstd compress
pub fn zstd_compress<F>(input: &[u8], mut write: F) -> Result<()>
where
    F: FnMut(&[u8]) -> Result<()>,
{
    const BUFSIZE: usize = 0x4000;
    let mut ctx = zstd_safe::CCtx::create();
    let mut buf = [0_u8; BUFSIZE];
    let mut input_buffer = InBuffer { src: input, pos: 0 };
    while input_buffer.pos < input.len() {
        let mut output_buffer = OutBuffer::around(&mut buf);
        match ctx.compress_stream(&mut output_buffer, &mut input_buffer) {
            Ok(_) => {
                let written = output_buffer.as_slice();
                if !written.is_empty() {
                    write(written)?;
                }
            }
            Err(error) => bail!("zstd_compress: {}", zstd_safe::get_error_name(error)),
        }
    }
    loop {
        let mut output_buffer = OutBuffer::around(&mut buf);
        match ctx.end_stream(&mut output_buffer) {
            Ok(code) => {
                let chunk = output_buffer.as_slice();
                if !chunk.is_empty() {
                    write(chunk)?;
                }
                if code == 0 {
                    break;
                }
            }
            Err(ec) => bail!("zstd_compress end: {}", zstd_safe::get_error_name(ec)),
        }
    }
    Ok(())
}

/// Streaming Zstd decompress
///
/// The input is allowed to contain more data than Zstd will consume.
/// Returns the actual number of bytes consumed from the input.
///
pub fn zstd_decompress<F>(input: &[u8], mut write: F) -> Result<usize>
where
    F: FnMut(&[u8]) -> Result<()>,
{
    const BUFSIZE: usize = 0x4000;
    let mut buf = [0_u8; BUFSIZE];
    let mut ctx = zstd_safe::DCtx::create();

    let mut input_buffer = InBuffer { src: input, pos: 0 };
    loop {
        let mut output_buffer = OutBuffer::around(&mut buf);
        match ctx.decompress_stream(&mut output_buffer, &mut input_buffer) {
            Ok(code) => {
                let out = output_buffer.as_slice();
                if !out.is_empty() {
                    write(out)?;
                }
                if code == 0 {
                    break;
                }
            }
            Err(ec) => bail!("zstd_compress: {}", zstd_safe::get_error_name(ec)),
        };
    }
    Ok(input_buffer.pos())
}

/// serializeJsonStringIfNeeded
pub fn serialize_json_string_if_needed<W>(input: &[u8], mut write: W) -> Result<()>
where
    W: FnMut(&[u8]) -> Result<()>,
{
    if input.is_empty()
        || input
            .iter()
            .any(|&ch| ch <= 0x1f || ch >= 0x7f || ch == b' ' || ch == b'\"')
    {
        serialize_json_string(input, write)
    } else {
        write(input)
    }
}

pub fn serialize_json_string<W>(input: &[u8], mut write: W) -> Result<()>
where
    W: FnMut(&[u8]) -> Result<()>,
{
    write(b"\"")?;
    for &ch in input {
        match ch {
            b'"' => write(b"\\\"")?,
            b'\\' => write(b"\\\\")?,
            0x08 => write(b"\\b")?,
            0x0C => write(b"\\f")?,
            b'\n' => write(b"\\n")?,
            b'\r' => write(b"\\r")?,
            b'\t' => write(b"\\t")?,
            other_char => {
                // TODO use range pattern instead
                if (32..=126).contains(&other_char) {
                    write(&[other_char])?;
                } else {
                    // \u00XX style escaping
                    let bytes = &[
                        b'\\',
                        b'u',
                        b'0',
                        b'0',
                        to_hex(other_char >> 4),
                        to_hex(other_char & 0xf),
                    ];
                    write(bytes)?;
                }
            }
        }
    }
    write(b"\"")?;
    Ok(())
}

#[must_use]
pub fn to_hex(index: u8) -> u8 {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    #[expect(clippy::indexing_slicing, reason = "the range is safe")]
    HEX_CHARS[(index & 0x0f) as usize]
}

pub fn from_hex(hex_digit: u8) -> Result<u8> {
    // TODO use functions from std
    if hex_digit.is_ascii_digit() {
        Ok(hex_digit - b'0')
    } else if (b'a'..=b'f').contains(&hex_digit) {
        Ok(10 + (hex_digit - b'a'))
    } else if (b'A'..=b'F').contains(&hex_digit) {
        Ok(10 + (hex_digit - b'A'))
    } else {
        bail!("Invalid hex digit: {}", hex_digit);
    }
}

// deSerializeJsonStringIfNeeded
// Returns number of bytes consumed by the "json" string, so that parsing can continue after.
pub fn deserialize_json_string_if_needed(input: &[u8]) -> Result<(Vec<u8>, usize), anyhow::Error> {
    if input.is_empty() {
        Ok((Vec::new(), 0))
    } else {
        if input[0] == b'"' {
            return deserialize_json_string(input);
        }
        // Just a normal string, consume up until whitespace or eof
        let endpos = input
            .iter()
            .position(|&ch| ch == b' ' || ch == b'\n')
            .unwrap_or(input.len());
        Ok((input[..endpos].to_vec(), endpos))
    }
}

struct MiniReader<'input> {
    input: &'input [u8],
    pos: usize,
}

impl<'input> MiniReader<'input> {
    pub(crate) fn new(input: &'input [u8], pos: usize) -> Self {
        Self { input, pos }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.input.len() - self.pos
    }

    pub(crate) fn take(&mut self, count: usize) -> Result<&'input [u8]> {
        if self.pos + count > self.input.len() {
            bail!("Luanti JSON string ended prematurely");
        }
        let result = &self.input[self.pos..self.pos + count];
        self.pos += count;
        Ok(result)
    }

    pub(crate) fn take1(&mut self) -> Result<u8> {
        self.take(1).map(|ch| ch[0])
    }
}

pub fn deserialize_json_string(input: &[u8]) -> Result<(Vec<u8>, usize), anyhow::Error> {
    let mut result: Vec<u8> = Vec::new();
    assert_eq!(input[0], b'"', "unexpected start of string");
    let mut reader = MiniReader::new(input, 1);
    while reader.remaining() > 0 {
        let ch = reader.take1()?;
        if ch == b'"' {
            return Ok((result, reader.pos));
        } else if ch == b'\\' {
            let code = reader.take1()?;
            match code {
                b'b' => result.push(0x08),
                b'f' => result.push(0x0C),
                b'n' => result.push(b'\n'),
                b'r' => result.push(b'\r'),
                b't' => result.push(b'\t'),
                b'u' => {
                    // "Unicode"
                    let codepoint = reader.take(4)?;
                    if codepoint[0] != b'0' || codepoint[1] != b'0' {
                        bail!("Unsupported unicode in Luanti JSON");
                    }
                    let hi = from_hex(codepoint[2])?;
                    let lo = from_hex(codepoint[3])?;
                    result.push((hi << 4) | lo);
                }
                other_char => result.push(other_char),
            }
        } else {
            result.push(ch);
        }
    }
    bail!("Luanti JSON string ended prematurely");
}

/// This is needed to handle the crazy inventory parsing.
#[must_use]
pub fn split_by_whitespace(line: &[u8]) -> Vec<&[u8]> {
    line.split(|ch| *ch == b' ' || *ch == b'\n')
        .filter(|item| !item.is_empty())
        .collect()
}

#[must_use]
pub fn skip_whitespace(line: &[u8]) -> &[u8] {
    match line.iter().position(|ch| *ch != b' ' && *ch != b'\n') {
        Some(pos) => &line[pos..],
        None => &line[line.len()..],
    }
}

/// Returns the next word (non-whitespace chunk) in u8 slice,
/// and the remainder (which may still have whitespace)
///
/// Returns None when the remainder is empty or all whitespace.
#[must_use]
pub fn next_word(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let line = skip_whitespace(line);
    match line.iter().position(|ch| *ch == b' ' || *ch == b'\n') {
        Some(endpos) => Some((&line[..endpos], &line[endpos..])),
        None => {
            if line.is_empty() {
                None
            } else {
                Some((line, &line[line.len()..]))
            }
        }
    }
}

#[must_use]
pub fn compress_zlib(uncompressed: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(uncompressed, 6)
}

/// This method must detect the end of the stream.
/// 'uncompressed' may have more data past the end of the zlib stream
/// Returns (`bytes_consumed`, `uncompressed_data`)
pub fn decompress_zlib(input: &[u8]) -> Result<(usize, Vec<u8>)> {
    let flags = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
        | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
    let mut ret: Vec<u8> = vec![0; input.len().saturating_mul(2)];

    let mut decompressor = Box::<DecompressorOxide>::default();

    let mut in_pos = 0;
    let mut out_pos = 0;
    loop {
        // Wrap the whole output slice so we know we have enough of the
        // decompressed data for matches.
        let (status, in_consumed, out_consumed) = inflate::core::decompress(
            &mut decompressor,
            &input[in_pos..],
            &mut ret,
            out_pos,
            flags,
        );
        in_pos += in_consumed;
        out_pos += out_consumed;

        match status {
            inflate::TINFLStatus::Done => {
                ret.truncate(out_pos);
                return Ok((in_pos, ret));
            }

            inflate::TINFLStatus::HasMoreOutput => {
                // if the buffer has already reached the size limit, return an error
                // calculate the new length, capped at `max_output_size`
                let new_len = ret.len().saturating_mul(2);
                ret.resize(new_len, 0);
            }

            err => bail!(
                "zlib decompression error: in_pos={}, out_pos={}, {:?}",
                in_pos,
                out_pos,
                err
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;
    use log::error;
    use rand;
    use rand::Rng;
    use rand::RngCore;
    use rand::rng;

    fn rand_bytes(range: Range<usize>) -> Vec<u8> {
        let mut rng = rng();
        let length = rng.random_range(range);
        let mut input = vec![0_u8; length];
        rng.fill_bytes(input.as_mut_slice());
        input
    }

    fn serialize_to_vec(input: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        serialize_json_string_if_needed(input, |chunk| {
            out.extend(chunk);
            Ok(())
        })
        .unwrap();
        out
    }

    #[test]
    fn json_serialize_deserialize_fuzz() {
        for _ in 0..10000 {
            let input = rand_bytes(0..100);
            let serialized = serialize_to_vec(&input);
            // At some junk on the end to make sure it doesn't take more than it should
            let serialized_plus_junk =
                [serialized.as_slice(), &[32], rand_bytes(0..20).as_slice()].concat();

            let (result, consumed) =
                deserialize_json_string_if_needed(&serialized_plus_junk).unwrap();
            if input != result {
                error!("input = {:?}", input);
                error!("serialized = {:?}", serialized);
                error!("serialized_plus_junk = {:?}", serialized_plus_junk);
                error!("result = {:?}", result);
                error!("consumed = {}", consumed);
                panic!();
            }
            assert_eq!(input, result);
            assert_eq!(consumed, serialized.len());
        }
    }

    #[test]
    fn itos_test() {
        assert_eq!(itos!(123), &[49, 50, 51]);
        assert_eq!(itos!(-100), &[45, 49, 48, 48]);
        assert_eq!(itos!(0), &[48]);
    }

    #[test]
    fn itos_stoi_fuzz() {
        for i in -10000..10000 {
            let str = itos!(i);
            let integer: i32 = stoi(str).expect("Should not have failed");
            assert_eq!(integer, i);
        }
    }
}
