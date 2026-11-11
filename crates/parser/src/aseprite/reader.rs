//! Aseprite file reader

use crate::aseprite::chunks::*;
use crate::aseprite::*;
use oversprite_core::{
    Cel, CelExtra, CoreResult, FileFlags, Frame, Grid, Point, Rgba, Size, Sprite, Uuid,
};
use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Aseprite reader
pub struct AsepriteReader {
    pub strict_mode: bool,
}

impl Default for AsepriteReader {
    fn default() -> Self {
        Self::new()
    }
}

impl AsepriteReader {
    pub fn new() -> Self {
        Self { strict_mode: true }
    }

    pub fn read<R: Read + Seek>(&self, reader: &mut R) -> CoreResult<Sprite> {
        // Read header
        let header = AseHeader::read(reader)?;

        // Determine color mode
        let color_mode = color_mode_from_aseprite(header.color_depth);

        // Create sprite
        let mut sprite = Sprite::new_with_size(header.width as u32, header.height as u32, color_mode);
        sprite.header.uuid = Uuid::new_v4();
        sprite.header.name = Path::new("imported").file_stem().and_then(|s| s.to_str()).unwrap_or("imported").to_string();
        sprite.header.aseprite_compatible = true;

        // Drop the placeholder frame and layer the constructor created; the
        // file's own frames and layers replace them.
        sprite.frame_data.clear();
        sprite.frames.order.clear();
        sprite.layer_data.clear();
        sprite.layers.order.clear();
        sprite.layers.groups.clear();

        // Set metadata from header
        sprite.metadata.dimensions = Size::new(header.width as u32, header.height as u32);
        sprite.metadata.color_mode = color_mode;
        sprite.metadata.color_depth = header.color_depth;
        sprite.metadata.transparent_index = header.transparent_index;
        sprite.metadata.background_color = Rgba::TRANSPARENT;
        sprite.metadata.pixel_aspect = oversprite_core::metadata::PixelAspect::new(header.pixel_width as u16, header.pixel_height as u16);

        // Grid
        if header.grid_width > 0 && header.grid_height > 0 {
            sprite.metadata.grid = Grid::new(
                Point::new(header.grid_x as i32, header.grid_y as i32),
                Size::new(header.grid_width as u32, header.grid_height as u32)
            );
        }

        // Flags
        sprite.metadata.flags = FileFlags::from_bits_truncate(header.flags);
        sprite.metadata.default_frame_duration_ms = header.speed as u32;

        // File flags for later use
        let file_flags = header.flags;
        let _has_layer_uuids = (file_flags & 4) != 0;

        // Layer index mapping (aseprite layer index -> our UUID)
        let mut layer_index_map: HashMap<u16, Uuid> = HashMap::new();
        let mut layer_order: Vec<Uuid> = Vec::new();

        // Frame index mapping
        let mut frame_uuids: Vec<Uuid> = Vec::new();

        // Pending cels to resolve
        struct PendingCel {
            cel: Cel,
            layer_index: u16,
            linked_frame_index: Option<u16>,
        }
        let mut pending_cels: Vec<PendingCel> = Vec::new();

        // Read frames
        for frame_idx in 0..header.frames {
            let frame_header = AseFrameHeader::read(reader)?;
            let chunk_count = frame_header.chunk_count();

            // Create frame
            let duration = if frame_header.duration_ms > 0 {
                frame_header.duration_ms as u32
            } else {
                header.speed as u32
            };

            let mut frame = Frame::new(duration);
            let frame_uuid = frame.uuid;
            frame_uuids.push(frame_uuid);

            // Read chunks in this frame
            for _ in 0..chunk_count {
                let chunk_header = ChunkHeader::read(reader)?;
                let chunk_start = reader.stream_position()?;
                let chunk_end = chunk_start + (chunk_header.size - 6) as u64;

                // Parse based on chunk type
                match chunk_header.chunk_type {
                    chunk_type::LAYER => {
                        let layer = parse_layer_chunk(reader, color_mode)?;
                        let uuid = layer.uuid;

                        sprite.layer_data.insert(uuid, layer);
                        sprite.layers.add_layer(uuid, None);
                        layer_order.push(uuid);

                        // Map aseprite layer index to our UUID
                        layer_index_map.insert((layer_order.len() - 1) as u16, uuid);
                    }
                    chunk_type::CEL => {
                        let cel = parse_cel_chunk(reader, color_mode)?;
                        let layer_index = cel.user_data.get_property("aseprite_layer_index")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u16;

                        let linked_frame = cel.user_data.get_property("aseprite_linked_frame")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u16);

                        pending_cels.push(PendingCel {
                            cel,
                            layer_index,
                            linked_frame_index: linked_frame,
                        });
                    }
                    chunk_type::CEL_EXTRA => {
                        let (has_bounds, px, py, pw, ph) = parse_cel_extra_chunk(reader)?;
                        if let Some(pending) = pending_cels.last_mut() {
                            pending.cel.has_extra = has_bounds;
                            pending.cel.extra = CelExtra {
                                precise_bounds: has_bounds,
                                precise_x: px,
                                precise_y: py,
                                precise_width: pw,
                                precise_height: ph,
                            };
                        }
                    }
                    chunk_type::COLOR_PROFILE => {
                        let profile = parse_color_profile_chunk(reader)?;
                        sprite.metadata.color_profile = profile;
                    }
                    chunk_type::EXTERNAL_FILES => {
                        let _files = parse_external_files_chunk(reader)?;
                        // Store in user_data
                    }
                    chunk_type::TAGS => {
                        let tags = parse_tags_chunk(reader)?;
                        sprite.tags = tags;
                    }
                    chunk_type::PALETTE => {
                        let palette = parse_palette_chunk(reader)?;
                        // This is the global palette
                        if frame_idx == 0 {
                            sprite.global_palette = palette;
                        } else {
                            // Per-frame palette override
                            sprite.frame_palettes.insert(frame_uuid, palette);
                        }
                    }
                    chunk_type::USER_DATA => {
                        let user_data = parse_user_data_chunk(reader)?;
                        // Associate with last read object (layer, frame, cel, tag)
                        // For simplicity, attach to sprite
                        sprite.user_data.merge(user_data);
                    }
                    chunk_type::SLICE => {
                        let slice = parse_slice_chunk(reader)?;
                        sprite.slices.push(slice);
                    }
                    chunk_type::TILESET => {
                        let tileset = parse_tileset_chunk(reader)?;
                        sprite.tilesets.insert(tileset.uuid, tileset);
                    }
                    chunk_type::OLD_PALETTE_1 | chunk_type::OLD_PALETTE_2 => {
                        // Legacy palette, only use if no new palette chunk found
                        let _palette = parse_old_palette_chunk(reader, chunk_header.chunk_type)?;
                    }
                    _ => {
                        // Skip unknown chunks
                        reader.seek(SeekFrom::Start(chunk_end))?;
                    }
                }

                // Ensure we're at the right position
                let current = reader.stream_position()?;
                if current < chunk_end {
                    reader.seek(SeekFrom::Start(chunk_end))?;
                }
            }

            // Resolve cels for this frame
            for pending in pending_cels.drain(..) {
                if let Some(&layer_uuid) = layer_index_map.get(&pending.layer_index) {
                    let mut cel = pending.cel;
                    cel.layer_uuid = layer_uuid;

                    // Resolve linked frame
                    if let Some(linked_idx) = pending.linked_frame_index {
                        if (linked_idx as usize) < frame_uuids.len() {
                            cel.linked_frame_uuid = Some(frame_uuids[linked_idx as usize]);
                        }
                    }

                    // Add to frame
                    frame.add_cel(cel.uuid);
                    sprite.cel_data.insert(cel.uuid, cel);
                }
            }

            sprite.frame_data.insert(frame_uuid, frame);
            sprite.frames.add_frame(frame_uuid);
        }

        // Validate
        sprite.validate()?;

        Ok(sprite)
    }

    /// Read from file path
    pub fn read_file(&self, path: &Path) -> CoreResult<Sprite> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        self.read(&mut reader)
    }
}

/// Helper trait for getting numeric values from PropertyValue
trait PropertyValueExt {
    fn as_u64(&self) -> Option<u64>;
}

impl PropertyValueExt for oversprite_core::PropertyValue {
    fn as_u64(&self) -> Option<u64> {
        match self {
            oversprite_core::PropertyValue::UInt8(v) => Some(*v as u64),
            oversprite_core::PropertyValue::UInt16(v) => Some(*v as u64),
            oversprite_core::PropertyValue::UInt32(v) => Some(*v as u64),
            oversprite_core::PropertyValue::UInt64(v) => Some(*v),
            oversprite_core::PropertyValue::Int8(v) => Some(*v as u64),
            oversprite_core::PropertyValue::Int16(v) => Some(*v as u64),
            oversprite_core::PropertyValue::Int32(v) => Some(*v as u64),
            oversprite_core::PropertyValue::Int64(v) => Some(*v as u64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_reader_creation() {
        let reader = AsepriteReader::new();
        assert!(reader.strict_mode);
    }
}