use std::marker::PhantomData;

use anyhow::bail;
use log::trace;

use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeError, SerializeResult, Serializer, VecSerializer},
    util::{zstd_compress, zstd_decompress},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ZLibCompressed<T>(PhantomData<T>);

impl<T: Serialize> Serialize for ZLibCompressed<T> {
    type Input = T::Input;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // TODO(paradust): Performance nightmare.

        // Serialize 'value' to a temporary buffer, and then compress
        let mut tmp = VecSerializer::new(ser.context(), 1024);
        <T as Serialize>::serialize(value, &mut tmp)?;
        let tmp = tmp.take();
        let tmp = miniz_oxide::deflate::compress_to_vec_zlib(&tmp, 6);

        // Write the size as a u32, followed by the data
        u32::serialize(&u32::try_from(tmp.len())?, ser)?;
        ser.write_bytes(&tmp)?;
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for ZLibCompressed<T> {
    type Output = T::Output;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let num_bytes = u32::deserialize(deser)? as usize;
        trace!("deserialize {num_bytes} bytes of compressed data");
        let data = deser.take(num_bytes)?;
        // TODO(paradust): DANGEROUS. There is no decompression size bound.
        match miniz_oxide::inflate::decompress_to_vec_zlib(data) {
            Ok(decompressed) => {
                let mut tmp = Deserializer::new(deser.context(), &decompressed);
                Ok(<T as Deserialize>::deserialize(&mut tmp)?)
            }
            Err(err) => bail!(DeserializeError::DecompressionFailed(err.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZStdCompressed<T>(PhantomData<T>);

impl<T: Serialize> Serialize for ZStdCompressed<T> {
    type Input = T::Input;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // Serialize 'value' into a temporary buffer
        // TODO(paradust): Performance concern, could stream instead
        let mut tmp = VecSerializer::new(ser.context(), 0x0001_0000);
        <T as Serialize>::serialize(value, &mut tmp)?;
        let tmp = tmp.take();
        match zstd_compress(&tmp, |chunk| {
            ser.write_bytes(chunk)?;
            Ok(())
        }) {
            Ok(()) => Ok(()),
            Err(err) => bail!(SerializeError::CompressionFailed(err.to_string())),
        }
    }
}

impl<T: Deserialize> Deserialize for ZStdCompressed<T> {
    type Output = T::Output;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        // Decompress to a temporary buffer
        let mut tmp: Vec<u8> = Vec::with_capacity(0x0001_0000);
        match zstd_decompress(deser.peek_all(), |chunk| {
            tmp.extend_from_slice(chunk);
            Ok(())
        }) {
            Ok(consumed) => {
                deser.take(consumed)?;
                let mut tmp_deser = Deserializer::new(deser.context(), &tmp);
                Ok(<T as Deserialize>::deserialize(&mut tmp_deser)?)
            }
            Err(err) => bail!(DeserializeError::DecompressionFailed(err.to_string())),
        }
    }
}
