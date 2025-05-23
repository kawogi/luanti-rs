use crate::types::CommandDirection;
use crate::types::ProtocolContext;
use anyhow::bail;
use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::Utf8Error;

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error("Bad Packet Type {0:?} type={1}")]
    BadPacketId(CommandDirection, u16),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Invalid Protocol id: {0}")]
    InvalidProtocolId(u32),
    #[error("Invalid channel: {0}")]
    InvalidChannel(u8),
    #[error("Invalid Packet Kind: {0}")]
    InvalidPacketKind(u8),
    #[error("DecompressionFailed: {0}")]
    DecompressionFailed(String),
    #[error("OtherError: {0}")]
    OtherError(String),
    #[error("EOF during deserialization: {0}")]
    Eof(String), // Data ended prematurely
}

impl From<Utf8Error> for DeserializeError {
    fn from(other: Utf8Error) -> DeserializeError {
        DeserializeError::InvalidValue(format!("Utf8Error {other:?}"))
    }
}

impl From<ParseIntError> for DeserializeError {
    fn from(other: ParseIntError) -> DeserializeError {
        DeserializeError::InvalidValue(format!("ParseIntError {other:?}"))
    }
}

impl From<anyhow::Error> for DeserializeError {
    fn from(value: anyhow::Error) -> Self {
        DeserializeError::OtherError(format!("OtherError {value:?}"))
    }
}

pub type DeserializeResult<R> = anyhow::Result<R>;

pub struct Deserializer<'data> {
    pub context: ProtocolContext,
    pub data: &'data [u8], // Remaining data
}

impl<'data> Deserializer<'data> {
    #[must_use]
    pub fn new(context: ProtocolContext, data: &'data [u8]) -> Self {
        Self { context, data }
    }

    /// Take a number of bytes, and return a sub-Deserializer which
    /// only operates on those bytes
    pub fn slice(&mut self, count: usize) -> DeserializeResult<Self> {
        Ok(Self {
            context: self.context,
            data: self.take(count)?,
        })
    }

    #[must_use]
    pub fn context(&self) -> ProtocolContext {
        self.context
    }

    #[must_use]
    pub fn direction(&self) -> CommandDirection {
        self.context.dir
    }

    /// reports whether there are still bytes left for deserialization
    #[must_use]
    pub fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len()
    }

    /// Finds the first occurrence of the byte 'b'
    /// from the current position in the stream.
    pub fn find(&mut self, byte: u8) -> Option<usize> {
        self.data.iter().position(|ch| *ch == byte)
    }

    pub fn peek(&mut self, count: usize) -> DeserializeResult<&'data [u8]> {
        if count > self.data.len() {
            bail!(DeserializeError::Eof(format!(
                "Deserializer::peek({count})"
            )))
        }
        Ok(&self.data[0..count])
    }

    pub fn peek_all(&mut self) -> &'data [u8] {
        self.data
    }

    pub fn take(&mut self, count: usize) -> DeserializeResult<&'data [u8]> {
        if count > self.data.len() {
            bail!(DeserializeError::Eof(format!(
                "Deserializer::take({count})"
            )))
        }
        let (ret, data) = self.data.split_at(count);
        self.data = data;
        Ok(ret)
    }

    pub fn take_n<const N: usize>(&mut self) -> DeserializeResult<[u8; N]> {
        Ok(self.take(N)?.try_into().unwrap())
    }

    pub fn take_all(&mut self) -> &'data [u8] {
        let (ret, data) = self.data.split_at(self.data.len());
        self.data = data;
        ret
    }

    /// Peek the next line (including ending \n, if present)
    /// If the stream is at end, this will be an empty slice.
    pub fn peek_line(&mut self) -> DeserializeResult<&'data [u8]> {
        let line_len = match self.find(b'\n') {
            Some(pos) => pos + 1,
            None => self.remaining(),
        };
        self.peek(line_len)
    }

    /// Take the next line (including ending \n, if present)
    /// If the stream is at end, this will be an empty slice.
    pub fn take_line(&mut self) -> DeserializeResult<&'data [u8]> {
        let line_len = match self.find(b'\n') {
            Some(pos) => pos + 1,
            None => self.remaining(),
        };
        self.take(line_len)
    }

    /// Take bytes until whitespace or end of stream
    /// If `skip_whitespace` is true, skips initial whitespace first.
    /// If `skip_whitespace` is false, and the next byte is a space,
    /// nothing is taken, and the returned will be empty.
    pub fn take_word(&mut self, skip_whitespace: bool) -> &'data [u8] {
        if skip_whitespace {
            self.take_space();
        }
        match self.data.iter().position(|&ch| ch == b' ' || ch == b'\n') {
            Some(end) => {
                let (ret, data) = self.data.split_at(end);
                self.data = data;
                ret
            }
            None => self.take_all(),
        }
    }

    /// Take whitespace from the current cursor.
    /// Repositioning the cursor at the start of the next word (or end of stream)
    pub fn take_space(&mut self) {
        match self.data.iter().position(|&ch| ch != b' ' && ch != b'\n') {
            Some(pos) => {
                (_, self.data) = self.data.split_at(pos);
            }
            None => {
                self.take_all();
            }
        }
    }
}

pub trait Deserialize: Sized + Debug {
    /// Output should be Self, except for wrapper types.
    type Output;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output>;
}
