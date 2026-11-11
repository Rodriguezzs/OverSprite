//! Aseprite file writer

use crate::aseprite::*;
use crate::common::*;
use crate::ParserResult;
use oversprite_core::{
    Cel, CelExtra, CelType, ColorMode, ColorProfile, ColorProfileType, Layer, LayerType, Palette, Slice, Sprite,
    Tag, AnimationDirection, Tileset, TilesetFlags, Uuid,
};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Aseprite writer
pub struct AsepriteWriter {
    pub write_legacy_palette: bool,
}

impl Default for AsepriteWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AsepriteWriter {
    pub fn new() -> Self {
        Self { write_legacy_palette: true }
    }

    pub fn write<W: Write + std::io::Seek>(&self, sprite: &Sprite, writer: &mut W) -> CoreResult<()> {
        // Build layer index mapping (our UUID -> aseprite index)
        let layer_uuids: Vec<Uuid> = sprite.layers.order.clone();
        let layer_to_index: HashMap<Uuid, u16> = layer_uuids.iter()
            .enumerate()
            .map(|(i, uuid)| (*uuid, i as u16))
            .collect();

        // Build frame index mapping
        let frame_uuids: Vec<Uuid> = sprite.frames.order.clone();
        let frame_to_index: HashMap<Uuid, u16> = frame_uuids.iter()
            .enumerate()
            .map(|(i, uuid)| (*uuid, i as u16))
            .collect();

        // Calculate file size (we'll write header first, then update)
        let header_pos = writer.stream_position()?;

        // Write placeholder header
        let header = self.build_header(sprite);
        header.write(writer)?;

        // Write frames
        for (frame_idx, frame_uuid) in frame_uuids.iter().enumerate() {
            let frame = sprite.frame_data.get(frame_uuid).unwrap();
            let frame_start = writer.stream_position()?;

            // Write frame header placeholder
            let frame_header = AseFrameHeader {
                bytes_in_frame: 0, // Will update
                magic: FRAME_MAGIC,
                old_chunks: 0,
                duration_ms: frame.duration_ms as u16,
                _reserved: [0, 0],
                new_chunks: 0, // Will update
            };
            let frame_header_pos = writer.stream_position()?;
            frame_header.write(writer)?;

            let mut chunk_count: u32 = 0;

            // Write layer chunks (only in first frame)
            if frame_idx == 0 {
                for layer_uuid in &layer_uuids {
                    let layer = sprite.layer_data.get(layer_uuid).unwrap();
                    self.write_layer_chunk(writer, layer, sprite.metadata.color_mode)?;
                    chunk_count += 1;
                }
            }

            // Write cels for this frame
            for cel_uuid in &frame.cels {
                if let Some(cel) = sprite.cel_data.get(cel_uuid) {
                    self.write_cel_chunk(writer, cel, &layer_to_index, &frame_to_index, sprite.metadata.color_mode)?;
                    chunk_count += 1;
                }
            }

            // Write palette chunk (only in first frame, or if per-frame palette)
            if frame_idx == 0 {
                chunk_count += self.write_palette_chunk(writer, &sprite.global_palette)?;
            } else if let Some(palette) = sprite.frame_palettes.get(frame_uuid) {
                chunk_count += self.write_palette_chunk(writer, palette)?;
            }

            // Write tags chunk (only in first frame)
            if frame_idx == 0 && !sprite.tags.is_empty() {
                self.write_tags_chunk(writer, &sprite.tags)?;
                chunk_count += 1;
            }

            // Write slices chunk (only in first frame)
            if frame_idx == 0 && !sprite.slices.is_empty() {
                for slice in &sprite.slices {
                    self.write_slice_chunk(writer, slice)?;
                    chunk_count += 1;
                }
            }

            // Write tileset chunks (only in first frame)
            if frame_idx == 0 {
                for tileset in sprite.tilesets.values() {
                    self.write_tileset_chunk(writer, tileset)?;
                    chunk_count += 1;
                }
            }

            // Write color profile chunk (only in first frame)
            if frame_idx == 0 && sprite.metadata.color_profile.profile_type != ColorProfileType::None {
                self.write_color_profile_chunk(writer, &sprite.metadata.color_profile)?;
                chunk_count += 1;
            }

            // Update frame header with correct values
            let frame_end = writer.stream_position()?;
            let frame_size = (frame_end - frame_start) as u32;

            writer.seek(std::io::SeekFrom::Start(frame_header_pos))?;
            let updated_frame_header = AseFrameHeader {
                bytes_in_frame: frame_size,
                magic: FRAME_MAGIC,
                old_chunks: if chunk_count > 0xFFFF { 0xFFFF } else { chunk_count as u16 },
                duration_ms: frame.duration_ms as u16,
                _reserved: [0, 0],
                new_chunks: chunk_count,
            };
            updated_frame_header.write(writer)?;
            writer.seek(std::io::SeekFrom::Start(frame_end))?;
        }

        // Update main header with file size
        let file_end = writer.stream_position()?;
        let file_size = file_end as u32;

        writer.seek(std::io::SeekFrom::Start(header_pos))?;
        let mut updated_header = header;
        updated_header.file_size = file_size;
        updated_header.frames = frame_uuids.len() as u16;
        updated_header.width = sprite.metadata.dimensions.w as u16;
        updated_header.height = sprite.metadata.dimensions.h as u16;
        updated_header.color_depth = sprite.metadata.color_mode.to_aseprite_depth();
        updated_header.flags = sprite.metadata.flags.bits();
        updated_header.speed = sprite.metadata.default_frame_duration_ms as u16;
        updated_header.transparent_index = sprite.metadata.transparent_index;
        updated_header.num_colors = if sprite.metadata.color_mode == ColorMode::Indexed {
            sprite.global_palette.colors.len() as u16
        } else {
            0
        };
        updated_header.pixel_width = sprite.metadata.pixel_aspect.width as u8;
        updated_header.pixel_height = sprite.metadata.pixel_aspect.height as u8;
        updated_header.grid_x = sprite.metadata.grid.origin.x as i16;
        updated_header.grid_y = sprite.metadata.grid.origin.y as i16;
        updated_header.grid_width = sprite.metadata.grid.cell_size.w as u16;
        updated_header.grid_height = sprite.metadata.grid.cell_size.h as u16;
        updated_header.write(writer)?;

        Ok(())
    }

    fn build_header(&self, sprite: &Sprite) -> AseHeader {
        AseHeader {
            file_size: 0, // Will update
            magic: ASE_MAGIC,
            frames: sprite.frames.len() as u16,
            width: sprite.metadata.dimensions.w as u16,
            height: sprite.metadata.dimensions.h as u16,
            color_depth: sprite.metadata.color_mode.to_aseprite_depth(),
            flags: sprite.metadata.flags.bits(),
            speed: sprite.metadata.default_frame_duration_ms as u16,
            _reserved1: 0,
            _reserved2: 0,
            transparent_index: sprite.metadata.transparent_index,
            _reserved3: [0, 0, 0],
            num_colors: if sprite.metadata.color_mode == ColorMode::Indexed {
                sprite.global_palette.colors.len() as u16
            } else {
                0
            },
            pixel_width: sprite.metadata.pixel_aspect.width as u8,
            pixel_height: sprite.metadata.pixel_aspect.height as u8,
            grid_x: sprite.metadata.grid.origin.x as i16,
            grid_y: sprite.metadata.grid.origin.y as i16,
            grid_width: sprite.metadata.grid.cell_size.w as u16,
            grid_height: sprite.metadata.grid.cell_size.h as u16,
            _reserved4: [0; 90],
        }
    }

    fn write_layer_chunk<W: Write>(&self, writer: &mut W, layer: &Layer, _color_mode: ColorMode) -> ParserResult<()> {
        let mut flags: u16 = 0;
        if layer.visible { flags |= 1; }
        if layer.editable { flags |= 2; }
        if layer.lock_movement { flags |= 4; }
        if layer.is_background { flags |= 8; }
        if layer.prefer_linked_cels { flags |= 16; }
        if layer.collapsed { flags |= 32; }
        if layer.is_reference { flags |= 64; }

        let chunk_data = {
            let mut buf = Vec::new();
            write_u16(&mut buf, flags)?;
            write_u16(&mut buf, layer_type_to_aseprite(layer.layer_type))?;
            write_u16(&mut buf, layer.child_level)?;
            write_u16(&mut buf, layer.default_width as u16)?;
            write_u16(&mut buf, layer.default_height as u16)?;
            write_u16(&mut buf, blend_mode_to_aseprite(layer.blend_mode))?;
            buf.push(layer.opacity);
            buf.extend([0, 0, 0]);
            write_string(&mut buf, &layer.name)?;

            if layer.layer_type == LayerType::Tilemap {
                // Write tileset index (0 for now)
                write_u32(&mut buf, 0)?;
            }

            buf
        };

        // Write chunk header
        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::LAYER)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    fn write_cel_chunk<W: Write>(
        &self,
        writer: &mut W,
        cel: &Cel,
        layer_to_index: &HashMap<Uuid, u16>,
        frame_to_index: &HashMap<Uuid, u16>,
        _color_mode: ColorMode
    ) -> ParserResult<()> {
        let layer_index = *layer_to_index.get(&cel.layer_uuid).unwrap_or(&0);

        let chunk_data = {
            let mut buf = Vec::new();
            write_u16(&mut buf, layer_index)?;
            write_i16(&mut buf, cel.position.x as i16)?;
            write_i16(&mut buf, cel.position.y as i16)?;
            buf.push(cel.opacity);
            write_u16(&mut buf, cel_type_to_aseprite(cel.cel_type))?;
            write_i16(&mut buf, cel.z_index)?;
            buf.extend([0, 0, 0, 0, 0]);

            match cel.cel_type {
                CelType::Raw => {
                    // Raw image data
                    let width = cel.bounds.w as u16;
                    let height = cel.bounds.h as u16;
                    write_u16(&mut buf, width)?;
                    write_u16(&mut buf, height)?;
                    // Pixel data would go here
                }
                CelType::Linked => {
                    let linked_idx = cel.linked_frame_uuid
                        .and_then(|uuid| frame_to_index.get(&uuid))
                        .copied()
                        .unwrap_or(0);
                    write_u16(&mut buf, linked_idx)?;
                }
                CelType::Compressed => {
                    let width = cel.bounds.w as u16;
                    let height = cel.bounds.h as u16;
                    write_u16(&mut buf, width)?;
                    write_u16(&mut buf, height)?;
                    // Compressed data would go here
                }
                CelType::Tilemap => {
                    write_u16(&mut buf, cel.bounds.w as u16)?; // width in tiles
                    write_u16(&mut buf, cel.bounds.h as u16)?; // height in tiles
                    write_u16(&mut buf, 32)?; // bpp
                    write_u32(&mut buf, 0x1FFFFFFF)?; // id mask
                    write_u32(&mut buf, 0x20000000)?; // flip x mask
                    write_u32(&mut buf, 0x40000000)?; // flip y mask
                    write_u32(&mut buf, 0x80000000)?; // flip d mask
                    buf.extend([0; 10]); // reserved
                    // Compressed tile data would go here
                }
            }

            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::CEL)?;
        writer.write_all(&chunk_data)?;

        // Write cel extra if needed
        if cel.has_extra {
            self.write_cel_extra_chunk(writer, &cel.extra)?;
        }

        Ok(())
    }

    fn write_cel_extra_chunk<W: Write>(&self, writer: &mut W, extra: &CelExtra) -> ParserResult<()> {
        let mut flags: u32 = 0;
        if extra.precise_bounds { flags |= 1; }

        let chunk_data = {
            let mut buf = Vec::new();
            write_u32(&mut buf, flags)?;
            write_i32(&mut buf, (extra.precise_x.0 * 65536.0) as i32)?;
            write_i32(&mut buf, (extra.precise_y.0 * 65536.0) as i32)?;
            write_i32(&mut buf, (extra.precise_width.0 * 65536.0) as i32)?;
            write_i32(&mut buf, (extra.precise_height.0 * 65536.0) as i32)?;
            buf.extend([0; 16]);
            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::CEL_EXTRA)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    fn write_color_profile_chunk<W: Write>(&self, writer: &mut W, profile: &ColorProfile) -> ParserResult<()> {
        let (profile_type, flags) = match profile.profile_type {
            ColorProfileType::None => (0u16, 0u16),
            ColorProfileType::Srgb => {
                let mut f = 0u16;
                if profile.use_fixed_gamma { f |= 1; }
                (1, f)
            }
            ColorProfileType::Icc => {
                let mut f = 0u16;
                if profile.use_fixed_gamma { f |= 1; }
                (2, f)
            }
        };

        let chunk_data = {
            let mut buf = Vec::new();
            write_u16(&mut buf, profile_type)?;
            write_u16(&mut buf, flags)?;
            write_i32(&mut buf, (profile.fixed_gamma * 65536.0) as i32)?;
            buf.extend([0; 8]);
            if profile_type == 2 {
                if let Some(icc) = &profile.icc_data {
                    write_u32(&mut buf, icc.len() as u32)?;
                    buf.extend(icc);
                }
            }
            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::COLOR_PROFILE)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    /// Writes the modern palette chunk and, when enabled, the legacy palette
    /// chunk that older Aseprite versions expect.
    ///
    /// Returns the number of chunks written so the caller can keep the frame
    /// header's chunk count in sync with the stream.
    fn write_palette_chunk<W: Write>(&self, writer: &mut W, palette: &Palette) -> ParserResult<u32> {
        let chunk_data = {
            let mut buf = Vec::new();
            write_u32(&mut buf, palette.colors.len() as u32)?;
            write_u32(&mut buf, 0)?; // first
            write_u32(&mut buf, (palette.colors.len() - 1) as u32)?; // last
            buf.extend([0; 8]);

            for color in &palette.colors {
                let mut entry_flags: u16 = 0;
                if color.name.is_some() { entry_flags |= 1; }
                write_u16(&mut buf, entry_flags)?;
                buf.push(color.r);
                buf.push(color.g);
                buf.push(color.b);
                buf.push(color.a);
                if let Some(name) = &color.name {
                    write_string(&mut buf, name)?;
                }
            }
            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::PALETTE)?;
        writer.write_all(&chunk_data)?;

        let mut written = 1;

        // Also write legacy palette if enabled
        if self.write_legacy_palette {
            self.write_legacy_palette_chunk(writer, palette, chunk_type::OLD_PALETTE_1)?;
            written += 1;
        }

        Ok(written)
    }

    fn write_legacy_palette_chunk<W: Write>(&self, writer: &mut W, palette: &Palette, chunk_type: u16) -> ParserResult<()> {
        // Single packet carrying up to 256 colors.
        //
        // The packet's count byte encodes 256 as 0, so the number of color
        // triples written has to be tracked separately from the byte stored
        // in the packet header.
        let chunk_data = {
            let mut buf = Vec::new();
            write_u16(&mut buf, 1)?; // packets
            buf.push(0); // skip

            let color_count = palette.colors.len().min(256);
            buf.push(if color_count == 256 { 0 } else { color_count as u8 });

            for color in palette.colors.iter().take(color_count) {
                if chunk_type == chunk_type::OLD_PALETTE_1 {
                    buf.push(color.r);
                    buf.push(color.g);
                    buf.push(color.b);
                } else {
                    // Scale to 6-bit
                    buf.push((color.r as u16 * 63 / 255) as u8);
                    buf.push((color.g as u16 * 63 / 255) as u8);
                    buf.push((color.b as u16 * 63 / 255) as u8);
                }
            }
            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    fn write_tags_chunk<W: Write>(&self, writer: &mut W, tags: &[Tag]) -> ParserResult<()> {
        let chunk_data = {
            let mut buf = Vec::new();
            write_u16(&mut buf, tags.len() as u16)?;
            buf.extend([0; 8]);

            for tag in tags {
                write_u16(&mut buf, tag.from_frame as u16)?;
                write_u16(&mut buf, tag.to_frame as u16)?;
                let direction = match tag.direction {
                    AnimationDirection::Forward => 0u8,
                    AnimationDirection::Reverse => 1,
                    AnimationDirection::PingPong => 2,
                    AnimationDirection::PingPongReverse => 3,
                };
                buf.push(direction);
                write_u16(&mut buf, tag.repeat)?;
                buf.extend([0; 6]);
                buf.push(tag.color.r);
                buf.push(tag.color.g);
                buf.push(tag.color.b);
                buf.push(0); // extra
                write_string(&mut buf, &tag.name)?;
            }
            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::TAGS)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    fn write_slice_chunk<W: Write>(&self, writer: &mut W, slice: &Slice) -> ParserResult<()> {
        let mut flags: u32 = 0;
        if slice.is_9patch { flags |= 1; }
        if slice.has_pivot { flags |= 2; }

        let chunk_data = {
            let mut buf = Vec::new();
            write_u32(&mut buf, slice.keys.len() as u32)?;
            write_u32(&mut buf, flags)?;
            write_u32(&mut buf, 0)?; // reserved
            write_string(&mut buf, &slice.name)?;

            for key in &slice.keys {
                write_u32(&mut buf, key.frame)?;
                write_i32(&mut buf, key.bounds.x)?;
                write_i32(&mut buf, key.bounds.y)?;
                write_u32(&mut buf, key.bounds.w)?;
                write_u32(&mut buf, key.bounds.h)?;

                if slice.is_9patch {
                    if let Some(center) = key.center {
                        write_i32(&mut buf, center.x)?;
                        write_i32(&mut buf, center.y)?;
                        write_u32(&mut buf, center.w)?;
                        write_u32(&mut buf, center.h)?;
                    }
                }

                if slice.has_pivot {
                    if let Some(pivot) = key.pivot {
                        write_i32(&mut buf, pivot.x)?;
                        write_i32(&mut buf, pivot.y)?;
                    }
                }
            }
            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::SLICE)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    fn write_tileset_chunk<W: Write>(&self, writer: &mut W, tileset: &Tileset) -> ParserResult<()> {
        let chunk_data = {
            let mut buf = Vec::new();
            write_u32(&mut buf, 0)?; // tileset ID (internal)
            write_u32(&mut buf, tileset.flags.bits())?;
            write_u32(&mut buf, tileset.tile_count)?;
            write_u16(&mut buf, tileset.tile_width)?;
            write_u16(&mut buf, tileset.tile_height)?;
            write_i16(&mut buf, tileset.base_index)?;
            buf.extend([0; 14]);
            write_string(&mut buf, &tileset.name)?;

            if tileset.flags.contains(TilesetFlags::EXTERNAL_FILE) {
                if let Some(ref _ext) = tileset.external_ref {
                    write_u32(&mut buf, 0)?; // file ID
                    write_u32(&mut buf, 0)?; // external tileset ID
                }
            }

            if tileset.flags.contains(TilesetFlags::TILES_IN_FILE) {
                // Would write compressed image data here
                write_u32(&mut buf, 0)?; // data length
            }

            buf
        };

        write_u32(writer, (chunk_data.len() + 6) as u32)?;
        write_u16(writer, chunk_type::TILESET)?;
        writer.write_all(&chunk_data)?;

        Ok(())
    }

    /// Write to file path
    pub fn write_file(&self, sprite: &Sprite, path: &Path) -> CoreResult<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write(sprite, &mut writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_creation() {
        let writer = AsepriteWriter::new();
        assert!(writer.write_legacy_palette);
    }

    #[test]
    fn test_header_is_exactly_128_bytes() {
        let sprite = Sprite::new_with_size(32, 32, ColorMode::Rgba);
        let header = AsepriteWriter::new().build_header(&sprite);

        let mut cursor = std::io::Cursor::new(Vec::new());
        header.write(&mut cursor).expect("header write");

        assert_eq!(
            cursor.get_ref().len(),
            ASE_HEADER_SIZE,
            "Aseprite header must be exactly 128 bytes"
        );
    }

    #[test]
    fn test_written_file_roundtrips() {
        let mut sprite = Sprite::new_with_size(16, 16, ColorMode::Rgba);
        sprite.add_frame(sprite.metadata.default_frame_duration_ms);

        let mut cursor = std::io::Cursor::new(Vec::new());
        AsepriteWriter::new()
            .write(&sprite, &mut cursor)
            .expect("aseprite write");

        cursor.set_position(0);
        let parsed = AsepriteReader::new()
            .read(&mut cursor)
            .expect("aseprite read back");

        assert_eq!(parsed.metadata.dimensions.w, 16);
        assert_eq!(parsed.metadata.dimensions.h, 16);
        assert_eq!(parsed.frames.len(), sprite.frames.len());
    }
}