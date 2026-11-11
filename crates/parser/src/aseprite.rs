//! Aseprite file format parser
//!
//! Parses .aseprite/.ase files according to the specification in ase-file-specs.md

pub mod chunks;
pub mod reader;
pub mod writer;

pub use self::reader::AsepriteReader;
pub use self::writer::AsepriteWriter;
// read_aseprite and write_aseprite are defined below

use crate::common::*;
use crate::ParserResult;
use oversprite_core::{
    BlendMode, CelType, ColorMode, CoreResult, LayerType, Sprite,
};
use std::path::Path;

/// Aseprite file magic number
pub const ASE_MAGIC: u16 = 0xA5E0;
/// Frame magic number
pub const FRAME_MAGIC: u16 = 0xF1FA;

/// Chunk types
pub mod chunk_type {
    pub const OLD_PALETTE_1: u16 = 0x0004;
    pub const OLD_PALETTE_2: u16 = 0x0011;
    pub const LAYER: u16 = 0x2004;
    pub const CEL: u16 = 0x2005;
    pub const CEL_EXTRA: u16 = 0x2006;
    pub const COLOR_PROFILE: u16 = 0x2007;
    pub const EXTERNAL_FILES: u16 = 0x2008;
    pub const MASK: u16 = 0x2016;
    pub const PATH: u16 = 0x2017;
    pub const TAGS: u16 = 0x2018;
    pub const PALETTE: u16 = 0x2019;
    pub const USER_DATA: u16 = 0x2020;
    pub const SLICE: u16 = 0x2022;
    pub const TILESET: u16 = 0x2023;
}

/// Aseprite header (128 bytes)
#[derive(Debug, Clone)]
pub struct AseHeader {
    pub file_size: u32,
    pub magic: u16,
    pub frames: u16,
    pub width: u16,
    pub height: u16,
    pub color_depth: u16,
    pub flags: u32,
    pub speed: u16,
    pub _reserved1: u8,
    pub _reserved2: u8,
    pub transparent_index: u8,
    pub _reserved3: [u8; 3],
    pub num_colors: u16,
    pub pixel_width: u8,
    pub pixel_height: u8,
    pub grid_x: i16,
    pub grid_y: i16,
    pub grid_width: u16,
    pub grid_height: u16,
    pub _reserved4: [u8; 90],
}

impl AseHeader {
    pub fn read<R: std::io::Read>(reader: &mut R) -> ParserResult<Self> {
        let file_size = read_u32(reader)?;
        let magic = read_u16(reader)?;
        if magic != ASE_MAGIC {
            return Err(crate::ParserError::InvalidMagic { expected: ASE_MAGIC, actual: magic });
        }
        let frames = read_u16(reader)?;
        let width = read_u16(reader)?;
        let height = read_u16(reader)?;
        let color_depth = read_u16(reader)?;
        let flags = read_u32(reader)?;
        let speed = read_u16(reader)?;
        let _reserved1 = read_bytes(reader, 1)?[0];
        let _reserved2 = read_bytes(reader, 1)?[0];
        let transparent_index = read_bytes(reader, 1)?[0];
        let _reserved3 = read_bytes(reader, 3)?.try_into().unwrap();
        let num_colors = read_u16(reader)?;
        let pixel_width = read_bytes(reader, 1)?[0];
        let pixel_height = read_bytes(reader, 1)?[0];
        let grid_x = read_i16(reader)?;
        let grid_y = read_i16(reader)?;
        let grid_width = read_u16(reader)?;
        let grid_height = read_u16(reader)?;
        let _reserved4 = read_bytes(reader, 90)?.try_into().unwrap();

        Ok(Self {
            file_size,
            magic,
            frames,
            width,
            height,
            color_depth,
            flags,
            speed,
            _reserved1,
            _reserved2,
            transparent_index,
            _reserved3,
            num_colors,
            pixel_width,
            pixel_height,
            grid_x,
            grid_y,
            grid_width,
            grid_height,
            _reserved4,
        })
    }

    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> ParserResult<()> {
        write_u32(writer, self.file_size)?;
        write_u16(writer, self.magic)?;
        write_u16(writer, self.frames)?;
        write_u16(writer, self.width)?;
        write_u16(writer, self.height)?;
        write_u16(writer, self.color_depth)?;
        write_u32(writer, self.flags)?;
        write_u16(writer, self.speed)?;
        writer.write_all(&[self._reserved1])?;
        writer.write_all(&[self._reserved2])?;
        writer.write_all(&[self.transparent_index])?;
        writer.write_all(&self._reserved3)?;
        write_u16(writer, self.num_colors)?;
        writer.write_all(&[self.pixel_width])?;
        writer.write_all(&[self.pixel_height])?;
        write_i16(writer, self.grid_x)?;
        write_i16(writer, self.grid_y)?;
        write_u16(writer, self.grid_width)?;
        write_u16(writer, self.grid_height)?;
        writer.write_all(&self._reserved4)?;
        Ok(())
    }
}

/// Total size of the serialized Aseprite header.
///
/// The Aseprite format fixes this at 128 bytes; every field offset in this
/// module depends on it, so a change to the struct layout must update this
/// constant and the frame header offset that follows.
pub const ASE_HEADER_SIZE: usize = 128;

/// Frame header (16 bytes)
#[derive(Debug, Clone)]
pub struct AseFrameHeader {
    pub bytes_in_frame: u32,
    pub magic: u16,
    pub old_chunks: u16,
    pub duration_ms: u16,
    pub _reserved: [u8; 2],
    pub new_chunks: u32,
}

impl AseFrameHeader {
    pub fn read<R: std::io::Read>(reader: &mut R) -> ParserResult<Self> {
        let bytes_in_frame = read_u32(reader)?;
        let magic = read_u16(reader)?;
        if magic != FRAME_MAGIC {
            return Err(crate::ParserError::InvalidMagic { expected: FRAME_MAGIC, actual: magic });
        }
        let old_chunks = read_u16(reader)?;
        let duration_ms = read_u16(reader)?;
        let _reserved = read_bytes(reader, 2)?.try_into().unwrap();
        let new_chunks = read_u32(reader)?;

        Ok(Self {
            bytes_in_frame,
            magic,
            old_chunks,
            duration_ms,
            _reserved,
            new_chunks,
        })
    }

    pub fn chunk_count(&self) -> u32 {
        if self.old_chunks == 0xFFFF {
            self.new_chunks
        } else {
            self.old_chunks as u32
        }
    }

    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> ParserResult<()> {
        write_u32(writer, self.bytes_in_frame)?;
        write_u16(writer, self.magic)?;
        write_u16(writer, self.old_chunks)?;
        write_u16(writer, self.duration_ms)?;
        writer.write_all(&self._reserved)?;
        write_u32(writer, self.new_chunks)?;
        Ok(())
    }
}

/// Generic chunk header
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    pub size: u32,
    pub chunk_type: u16,
}

impl ChunkHeader {
    pub fn read<R: std::io::Read>(reader: &mut R) -> ParserResult<Self> {
        let size = read_u32(reader)?;
        let chunk_type = read_u16(reader)?;
        if size < 6 {
            return Err(crate::ParserError::InvalidData("Chunk size too small".into()));
        }
        Ok(Self { size, chunk_type })
    }
}

/// Read a complete .aseprite file
pub fn read_aseprite(path: &Path) -> CoreResult<Sprite> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    AsepriteReader::new().read(&mut reader)
}

/// Write a sprite to .aseprite format
pub fn write_aseprite(sprite: &Sprite, path: &Path) -> CoreResult<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    AsepriteWriter::new().write(sprite, &mut writer)
}

/// Blend mode mapping from Aseprite to OverSprite
pub fn blend_mode_from_aseprite(value: u16) -> BlendMode {
    BlendMode::from_u8(value as u8).unwrap_or(BlendMode::Normal)
}

/// Blend mode mapping to Aseprite
pub fn blend_mode_to_aseprite(mode: BlendMode) -> u16 {
    mode as u16
}

/// Layer type mapping
pub fn layer_type_from_aseprite(value: u16) -> LayerType {
    LayerType::from_u16(value).unwrap_or(LayerType::Image)
}

pub fn layer_type_to_aseprite(lt: LayerType) -> u16 {
    lt.to_u16()
}

/// Cel type mapping
pub fn cel_type_from_aseprite(value: u16) -> CelType {
    CelType::from_u16(value).unwrap_or(CelType::Compressed)
}

pub fn cel_type_to_aseprite(ct: CelType) -> u16 {
    ct.to_u16()
}

/// Color mode mapping
fn color_mode_from_aseprite(depth: u16) -> ColorMode {
    ColorMode::from_aseprite_depth(depth).unwrap_or(ColorMode::Rgba)
}