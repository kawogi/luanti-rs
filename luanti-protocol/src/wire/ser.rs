use anyhow::Result;
use anyhow::bail;
use std::num::TryFromIntError;

use crate::types::CommandDirection;
use crate::types::ProtocolContext;

#[derive(Debug, Clone, thiserror::Error)]
pub enum SerializeError {
    #[error("Ran out of space while serializing: {0}")]
    BufferLimit(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("CompressionFailed: {0}")]
    CompressionFailed(String),
}

impl From<TryFromIntError> for SerializeError {
    fn from(other: TryFromIntError) -> SerializeError {
        SerializeError::InvalidValue(format!("{other:?}"))
    }
}

pub type SerializeResult = Result<()>;

pub trait Serializer {
    type Marker;

    fn context(&self) -> ProtocolContext;

    // Serializing a ToServer or ToClient command
    fn direction(&self) -> CommandDirection;

    // Request writing directly to a slice
    // Needed for random access writes
    // It is not guaranteed the 'f' is called.
    fn write<F>(&mut self, length: usize, f: F) -> SerializeResult
    where
        F: FnOnce(&mut [u8]);

    // Write bytes
    fn write_bytes(&mut self, fragment: &[u8]) -> SerializeResult;

    // Reserve some bytes for writing later.
    fn write_marker(&mut self, length: usize) -> Result<Self::Marker, SerializeError>;

    // Write to the marker
    fn set_marker(&mut self, marker: Self::Marker, fragment: &[u8]) -> SerializeResult;

    // Number of bytes written to the stream after the marker (not including the marker itself)
    fn marker_distance(&self, marker: &Self::Marker) -> usize;
}

/// Serialize a Packet to a mutable slice
pub struct SliceSerializer<'data> {
    context: ProtocolContext,
    offset: usize,
    data: &'data mut [u8],
    overflow: bool,
}

impl<'data> SliceSerializer<'data> {
    pub fn new(context: ProtocolContext, data: &'data mut [u8]) -> Self {
        Self {
            context,
            offset: 0,
            data,
            overflow: false,
        }
    }

    /// Returns the finished serialized packet
    /// This is a sub-slice of the original data slice provided
    /// If the serializer ran out of space, returns None.
    pub fn take(&self) -> Result<&[u8]> {
        if self.overflow {
            bail!(SerializeError::BufferLimit(
                "SliceSerializer overflow".into()
            ));
        }
        Ok(&self.data[..self.offset])
    }
}

impl Serializer for SliceSerializer<'_> {
    type Marker = (usize, usize);

    fn context(&self) -> ProtocolContext {
        self.context
    }

    fn direction(&self) -> CommandDirection {
        self.context.dir
    }

    fn write_bytes(&mut self, fragment: &[u8]) -> SerializeResult {
        if self.offset + fragment.len() > self.data.len() {
            self.overflow = true;
            bail!(SerializeError::BufferLimit(
                "SliceSerializer out of space ".into(),
            ));
        }
        self.data[self.offset..self.offset + fragment.len()].copy_from_slice(fragment);
        self.offset += fragment.len();
        Ok(())
    }

    fn write_marker(&mut self, length: usize) -> Result<Self::Marker, SerializeError> {
        if self.offset + length > self.data.len() {
            self.overflow = true;
            Err(SerializeError::BufferLimit(
                "SliceSerializer out of space ".into(),
            ))
        } else {
            let marker = (self.offset, length);
            self.offset += length;
            Ok(marker)
        }
    }

    fn set_marker(&mut self, marker: Self::Marker, fragment: &[u8]) -> SerializeResult {
        let (offset, length) = marker;
        if fragment.len() != length {
            self.overflow = true;
            bail!(SerializeError::InvalidValue("Marker has wrong size".into(),));
        }
        self.data[offset..offset + length].copy_from_slice(fragment);
        Ok(())
    }

    fn marker_distance(&self, marker: &Self::Marker) -> usize {
        let (offset, length) = marker;
        self.offset - (offset + length)
    }

    fn write<F>(&mut self, length: usize, write_slice_fn: F) -> SerializeResult
    where
        F: FnOnce(&mut [u8]),
    {
        if self.offset + length > self.data.len() {
            self.overflow = true;
            bail!(SerializeError::BufferLimit(
                "SliceSerializer out of space ".into(),
            ))
        }
        write_slice_fn(&mut self.data[self.offset..self.offset + length]);
        self.offset += length;
        Ok(())
    }
}

pub struct VecSerializer {
    context: ProtocolContext,
    data: Vec<u8>,
}

impl VecSerializer {
    #[must_use]
    pub fn new(context: ProtocolContext, initial_capacity: usize) -> Self {
        Self {
            context,
            data: Vec::with_capacity(initial_capacity),
        }
    }

    #[must_use]
    pub fn take(self) -> Vec<u8> {
        self.data
    }
}

impl Serializer for VecSerializer {
    type Marker = (usize, usize);

    fn context(&self) -> ProtocolContext {
        self.context
    }

    fn direction(&self) -> CommandDirection {
        self.context.dir
    }

    fn write_bytes(&mut self, fragment: &[u8]) -> SerializeResult {
        self.data.extend_from_slice(fragment);
        Ok(())
    }

    fn write_marker(&mut self, length: usize) -> Result<Self::Marker, SerializeError> {
        let marker = (self.data.len(), length);
        self.data.resize(self.data.len() + length, 0_u8);
        Ok(marker)
    }

    fn set_marker(&mut self, marker: Self::Marker, fragment: &[u8]) -> SerializeResult {
        let (offset, length) = marker;
        self.data[offset..offset + length].copy_from_slice(fragment);
        Ok(())
    }

    fn marker_distance(&self, marker: &Self::Marker) -> usize {
        let (offset, length) = marker;
        self.data.len() - (offset + length)
    }

    fn write<F>(&mut self, length: usize, write_slice_fn: F) -> SerializeResult
    where
        F: FnOnce(&mut [u8]),
    {
        let offset = self.data.len();
        self.data.resize(offset + length, 0_u8);
        write_slice_fn(&mut self.data.as_mut_slice()[offset..offset + length]);
        Ok(())
    }
}

/// `MockSerializer`
/// Computes the size of the serialized output without storing it
pub struct MockSerializer {
    context: ProtocolContext,
    count: usize,
}

impl MockSerializer {
    #[must_use]
    pub fn new(context: ProtocolContext) -> Self {
        Self { context, count: 0 }
    }

    /// How many bytes have been written so far
    #[must_use]
    #[expect(
        clippy::len_without_is_empty,
        reason = "seems to be unneeded in this context"
    )]
    pub fn len(&self) -> usize {
        self.count
    }
}

impl Serializer for MockSerializer {
    type Marker = (usize, usize);

    fn context(&self) -> ProtocolContext {
        self.context
    }

    fn direction(&self) -> CommandDirection {
        self.context.dir
    }

    fn write_bytes(&mut self, fragment: &[u8]) -> SerializeResult {
        self.count += fragment.len();
        Ok(())
    }

    fn write_marker(&mut self, length: usize) -> Result<Self::Marker, SerializeError> {
        let marker = (self.count, length);
        self.count += length;
        Ok(marker)
    }

    fn set_marker(&mut self, _marker: Self::Marker, _fragment: &[u8]) -> SerializeResult {
        Ok(())
    }

    fn marker_distance(&self, marker: &Self::Marker) -> usize {
        let (offset, length) = marker;
        self.count - (offset + length)
    }

    fn write<F>(&mut self, length: usize, _f: F) -> SerializeResult
    where
        F: FnOnce(&mut [u8]),
    {
        self.count += length;
        Ok(())
    }
}

pub trait Serialize {
    type Input: ?Sized;
    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult;
}
