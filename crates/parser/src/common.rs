//! Common parsing utilities for OverSprite

use crate::ParserResult;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use oversprite_core::{BlendMode, CelType, LayerType};
use std::io::{Cursor, Read, Write};

/// Convert Aseprite layer type value to OverSprite LayerType
/// Aseprite: 0=Image, 1=Group, 2=Tilemap
pub fn layer_type_from_aseprite(value: u16) -> LayerType {
    LayerType::from_u16(value).unwrap_or(LayerType::Image)
}

/// Convert Aseprite blend mode value to OverSprite BlendMode
/// Aseprite blend modes match our enum exactly (0-19)
pub fn blend_mode_from_aseprite(value: u16) -> BlendMode {
    BlendMode::from_u8(value as u8).unwrap_or(BlendMode::Normal)
}

/// Convert Aseprite cel type value to OverSprite CelType
/// Aseprite: 0=Raw, 1=Linked, 2=Compressed, 3=Tilemap
pub fn cel_type_from_aseprite(value: u16) -> CelType {
    CelType::from_u16(value).unwrap_or(CelType::Compressed)
}

/// Read a little-endian u16
pub fn read_u16<R: Read>(reader: &mut R) -> ParserResult<u16> {
    Ok(reader.read_u16::<LittleEndian>()?)
}

/// Read a little-endian u32
pub fn read_u32<R: Read>(reader: &mut R) -> ParserResult<u32> {
    Ok(reader.read_u32::<LittleEndian>()?)
}

/// Read a little-endian i16
pub fn read_i16<R: Read>(reader: &mut R) -> ParserResult<i16> {
    Ok(reader.read_i16::<LittleEndian>()?)
}

/// Read a little-endian i32
pub fn read_i32<R: Read>(reader: &mut R) -> ParserResult<i32> {
    Ok(reader.read_i32::<LittleEndian>()?)
}

/// Read a fixed-length byte array
pub fn read_bytes<R: Read>(reader: &mut R, len: usize) -> ParserResult<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Read a UTF-8 string (WORD length + bytes)
pub fn read_string<R: Read>(reader: &mut R) -> ParserResult<String> {
    let len = read_u16(reader)? as usize;
    let bytes = read_bytes(reader, len)?;
    String::from_utf8(bytes).map_err(|e| crate::ParserError::InvalidData(e.to_string()))
}

/// Write a little-endian u16
pub fn write_u16<W: Write>(writer: &mut W, value: u16) -> ParserResult<()> {
    Ok(writer.write_u16::<LittleEndian>(value)?)
}

/// Write a little-endian u32
pub fn write_u32<W: Write>(writer: &mut W, value: u32) -> ParserResult<()> {
    Ok(writer.write_u32::<LittleEndian>(value)?)
}

/// Write a little-endian i16
pub fn write_i16<W: Write>(writer: &mut W, value: i16) -> ParserResult<()> {
    Ok(writer.write_i16::<LittleEndian>(value)?)
}

/// Write a little-endian i32
pub fn write_i32<W: Write>(writer: &mut W, value: i32) -> ParserResult<()> {
    Ok(writer.write_i32::<LittleEndian>(value)?)
}

/// Write a UTF-8 string (WORD length + bytes)
pub fn write_string<W: Write>(writer: &mut W, value: &str) -> ParserResult<()> {
    let bytes = value.as_bytes();
    if bytes.len() > u16::MAX as usize {
        return Err(crate::ParserError::InvalidData("String too long".into()));
    }
    write_u16(writer, bytes.len() as u16)?;
    writer.write_all(bytes)?;
    Ok(())
}

/// Decompress ZLIB data
pub fn decompress_zlib(data: &[u8]) -> ParserResult<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

/// Compress data with ZLIB
pub fn compress_zlib(data: &[u8]) -> ParserResult<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Decompress LZ4 frame data
pub fn decompress_lz4(data: &[u8]) -> ParserResult<Vec<u8>> {
    let mut decoder = FrameDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

/// Compress data with LZ4 frame
pub fn compress_lz4(data: &[u8]) -> ParserResult<Vec<u8>> {
    let mut encoder = FrameEncoder::new(Vec::new());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Read exactly n bytes or return error
pub fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8]) -> ParserResult<()> {
    reader.read_exact(buf)?;
    Ok(())
}

/// Skip n bytes
pub fn skip<R: Read>(reader: &mut R, n: usize) -> ParserResult<()> {
    let mut buf = vec![0u8; n];
    reader.read_exact(&mut buf)?;
    Ok(())
}

/// Cursor helper for parsing from bytes
pub struct ByteCursor<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> ByteCursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { cursor: Cursor::new(data) }
    }

    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    pub fn remaining(&self) -> usize {
        self.cursor.get_ref().len() - self.position()
    }

    pub fn read_u16(&mut self) -> ParserResult<u16> {
        read_u16(&mut self.cursor)
    }

    pub fn read_u32(&mut self) -> ParserResult<u32> {
        read_u32(&mut self.cursor)
    }

    pub fn read_i16(&mut self) -> ParserResult<i16> {
        read_i16(&mut self.cursor)
    }

    pub fn read_i32(&mut self) -> ParserResult<i32> {
        read_i32(&mut self.cursor)
    }

    pub fn read_bytes(&mut self, len: usize) -> ParserResult<Vec<u8>> {
        read_bytes(&mut self.cursor, len)
    }

    pub fn read_string(&mut self) -> ParserResult<String> {
        read_string(&mut self.cursor)
    }

    pub fn skip(&mut self, n: usize) -> ParserResult<()> {
        skip(&mut self.cursor, n)
    }
}