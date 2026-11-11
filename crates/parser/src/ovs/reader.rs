//! Ovs file reader

use crate::ovs::*;
use crate::common::decompress_lz4;
use crate::ParserResult;
use oversprite_core::{
    Cel, CelType, CoreError, CoreResult, FileFlags, Frame, FrameIndex, Layer, LayerIndex,
    Metadata, Palette, Point, Rect, Rgba, Slice, SliceKey, Sprite,
    SpriteHeader, Tag, AnimationDirection, Tileset, Uuid, UserData,
    CompressionConfig,
};
use std::io::{BufReader, Read, Seek};
use zip::ZipArchive;

/// Ovs reader
pub struct OvsReader {
    pub verify_checksums: bool,
}

impl Default for OvsReader {
    fn default() -> Self {
        Self::new()
    }
}

impl OvsReader {
    pub fn new() -> Self {
        Self { verify_checksums: true }
    }

    pub fn read<R: Read + Seek>(&self, reader: &mut R) -> CoreResult<Sprite> {
        let mut archive = ZipArchive::new(reader)?;

        // Read manifest
        let manifest = self.read_manifest(&mut archive)?;

        // Read metadata
        let metadata = self.read_metadata(&mut archive)?;

        // Create sprite
        let mut sprite = Sprite::new_with_size(
            metadata.dimensions.w,
            metadata.dimensions.h,
            metadata.color_mode
        );
        sprite.metadata = metadata;
        sprite.header = self.build_header(&manifest);

        // Clear the default layer and frame added by new_with_size
        // The saved data will provide the actual layers and frames
        sprite.layer_data.clear();
        sprite.frame_data.clear();
        sprite.layers = oversprite_core::LayerIndex::new();
        sprite.frames = oversprite_core::FrameIndex::new();

        // Read layers
        self.read_layers(&mut archive, &mut sprite)?;

        // Read frames
        self.read_frames(&mut archive, &mut sprite)?;

        // Read palette
        self.read_palette(&mut archive, &mut sprite)?;

        // Read tags
        self.read_tags(&mut archive, &mut sprite)?;

        // Read slices
        self.read_slices(&mut archive, &mut sprite)?;

        // Read tilesets
        self.read_tilesets(&mut archive, &mut sprite)?;

        // Read user data
        self.read_user_data(&mut archive, &mut sprite)?;

        // Read thumbnails
        self.read_thumbnails(&mut archive, &mut sprite)?;

        // Validate
        sprite.validate()?;

        Ok(sprite)
    }

    fn read_manifest(&self, archive: &mut ZipArchive<impl Read + Seek>) -> ParserResult<serde_json::Value> {
        let mut file = archive.by_name(OVS_MANIFEST)?;
        let data: serde_json::Value = rmp_serde::from_read(&mut file)?;
        self.verify_magic(&data)?;
        Ok(data)
    }

    /// Verify the header magic number identifies this as an `.ovs` file.
    fn verify_magic(&self, manifest: &serde_json::Value) -> ParserResult<()> {
        let magic = manifest.get("magic").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if magic != OVS_MAGIC {
            return Err(crate::ParserError::InvalidOvsMagic { expected: OVS_MAGIC, actual: magic });
        }
        Ok(())
    }

    fn read_metadata(&self, archive: &mut ZipArchive<impl Read + Seek>) -> CoreResult<Metadata> {
        let mut file = archive.by_name(OVS_METADATA)?;
        let metadata: Metadata = rmp_serde::from_read(&mut file)?;
        Ok(metadata)
    }

    fn build_header(&self, manifest: &serde_json::Value) -> SpriteHeader {
        SpriteHeader {
            uuid: Uuid::parse_str(manifest.get("uuid").and_then(|v| v.as_str()).unwrap_or("")).unwrap_or_else(|_| Uuid::new_v4()),
            name: manifest.get("created_by").and_then(|v| v.as_str()).unwrap_or("OverSprite").to_string(),
            version: manifest.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
            minor_version: manifest.get("minor_version").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            created_by: manifest.get("created_by").and_then(|v| v.as_str()).unwrap_or("OverSprite").to_string(),
            created_at: manifest.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            modified_at: manifest.get("modified_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            aseprite_compatible: manifest.get("aseprite_compatible").and_then(|v| v.as_bool()).unwrap_or(true),
            features: manifest.get("features").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            compression: CompressionConfig::default(),
            flags: FileFlags::LAYER_OPACITY_VALID | FileFlags::LAYER_BLEND_MODE_VALID | FileFlags::LAYERS_HAVE_UUIDS,
        }
    }

    fn read_layers(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        // Read layer index first
        let layer_index: LayerIndex = {
            let mut file = archive.by_name(OVS_LAYERS_INDEX)?;
            rmp_serde::from_read(&mut file)?
        };

        // Read each layer
        for layer_uuid in &layer_index.order {
            let layer_name = format!("layers/{}.msgpack", layer_uuid);
            let mut file = archive.by_name(&layer_name)?;
            let layer: Layer = rmp_serde::from_read(&mut file)?;

            // Validate
            layer.validate(sprite.metadata.color_mode)?;

            sprite.layer_data.insert(*layer_uuid, layer);
        }

        sprite.layers = layer_index;
        Ok(())
    }

    fn read_frames(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        // Read frame index first, limiting scope to drop the borrow before the loop
        let frame_index: FrameIndex = {
            let mut file = archive.by_name(OVS_FRAMES_INDEX)?;
            rmp_serde::from_read(&mut file)?
        };

        for frame_uuid in &frame_index.order {
            let frame_dir = format!("frames/{}/", frame_uuid);

            // Read cels index - limit scope to drop the borrow before reading pixel data
            let cels_data: serde_json::Value = {
                let mut cels_file = archive.by_name(&format!("{}cels.msgpack", frame_dir))?;
                rmp_serde::from_read(&mut cels_file)?
            };

            // Create frame - read duration from cels_data if available
            let frame_duration = cels_data.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(frame_index.default_duration_ms as u64) as u32;
            let mut frame = Frame::new(frame_duration);
            frame.uuid = *frame_uuid;

            // Read cels
            if let Some(cels_array) = cels_data.get("cels").and_then(|v| v.as_array()) {
                for cel_data in cels_array {
                    let cel_uuid = Uuid::parse_str(cel_data.get("uuid").and_then(|v| v.as_str()).unwrap_or(""))
                        .unwrap_or_else(|_| Uuid::new_v4());

                    // Read cel pixel data
                    let cel_type = CelType::from_u16(cel_data.get("cel_type").and_then(|v| v.as_u64()).unwrap_or(2) as u16)
                        .unwrap_or(CelType::Compressed);

                    let mut cel = Cel {
                        uuid: cel_uuid,
                        layer_uuid: Uuid::parse_str(cel_data.get("layer_uuid").and_then(|v| v.as_str()).unwrap_or("")).unwrap_or_else(|_| Uuid::nil()),
                        position: Point::new(
                            cel_data.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            cel_data.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        ),
                        opacity: cel_data.get("opacity").and_then(|v| v.as_u64()).unwrap_or(255) as u8,
                        cel_type,
                        z_index: cel_data.get("z_index").and_then(|v| v.as_i64()).unwrap_or(0) as i16,
                        bounds: Rect::new(
                            cel_data.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            cel_data.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            cel_data.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            cel_data.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        ),
                        linked_frame_uuid: cel_data.get("linked_frame_uuid")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok()),
                        tileset_uuid: cel_data.get("tileset_uuid")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok()),
                        has_extra: cel_data.get("has_extra").and_then(|v| v.as_bool()).unwrap_or(false),
                        extra: Default::default(),
                        user_data: UserData::default(),
                    };

                    // Read pixel data based on type
                    match cel_type {
                        CelType::Compressed | CelType::Raw => {
                            let pixel_file_name = format!("{}{}.pixels.lz4", frame_dir, cel_uuid);
                            if let Ok(mut pixel_file) = archive.by_name(&pixel_file_name) {
                                let mut compressed = Vec::new();
                                pixel_file.read_to_end(&mut compressed)?;
                                let decompressed = decompress_lz4(&compressed)
                                    .map_err(|e| CoreError::Lz4Error(e.to_string()))?;
                                cel.user_data.set_property("pixels", decompressed.into());
                            }
                        }
                        CelType::Tilemap => {
                            let tilemap_file_name = format!("{}{}.tilemap.lz4", frame_dir, cel_uuid);
                            if let Ok(mut tilemap_file) = archive.by_name(&tilemap_file_name) {
                                let mut compressed = Vec::new();
                                tilemap_file.read_to_end(&mut compressed)?;
                                let decompressed = decompress_lz4(&compressed)
                                    .map_err(|e| CoreError::Lz4Error(e.to_string()))?;
                                cel.user_data.set_property("tilemap", decompressed.into());
                            }
                        }
                        CelType::Linked => {
                            // No pixel data for linked cels
                        }
                    }

                    frame.add_cel(cel.uuid);
                    sprite.cel_data.insert(cel.uuid, cel);
                }
            }

            sprite.frame_data.insert(*frame_uuid, frame);
            sprite.frames.add_frame(*frame_uuid);
        }

        sprite.frames = frame_index;
        Ok(())
    }

    fn read_palette(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        // Global palette
        if let Ok(mut file) = archive.by_name(OVS_PALETTE_GLOBAL) {
            let palette: Palette = rmp_serde::from_read(&mut file)?;
            sprite.global_palette = palette;
        }

        // Per-frame palettes - collect names first to avoid borrow issues
        let palette_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .filter(|name| name.starts_with(OVS_PALETTE_DIR) && name.ends_with(".msgpack") && name != OVS_PALETTE_GLOBAL)
            .collect();

        for name in palette_names {
            let frame_uuid_str = name.trim_start_matches(OVS_PALETTE_DIR).trim_end_matches(".msgpack");
            if let Ok(frame_uuid) = Uuid::parse_str(frame_uuid_str) {
                let mut file = archive.by_name(&name)?;
                let palette: Palette = rmp_serde::from_read(&mut file)?;
                sprite.frame_palettes.insert(frame_uuid, palette);
            }
        }
        Ok(())
    }

    fn read_tags(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        if let Ok(mut file) = archive.by_name(OVS_TAGS_INDEX) {
            let tags_data: serde_json::Value = rmp_serde::from_read(&mut file)?;
            if let Some(tags_array) = tags_data.get("tags").and_then(|v| v.as_array()) {
                for tag_data in tags_array {
                    let tag = Tag {
                        uuid: Uuid::parse_str(tag_data.get("uuid").and_then(|v| v.as_str()).unwrap_or("")).unwrap_or_else(|_| Uuid::new_v4()),
                        name: tag_data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        from_frame: tag_data.get("from_frame").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        to_frame: tag_data.get("to_frame").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        direction: match tag_data.get("direction").and_then(|v| v.as_str()).unwrap_or("forward") {
                            "reverse" => AnimationDirection::Reverse,
                            "ping_pong" => AnimationDirection::PingPong,
                            "ping_pong_reverse" => AnimationDirection::PingPongReverse,
                            _ => AnimationDirection::Forward,
                        },
                        repeat: tag_data.get("repeat").and_then(|v| v.as_u64()).unwrap_or(0) as u16,
                        color: {
                            let color_array = tag_data.get("color").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                            Rgba::new(
                                color_array.get(0).and_then(|v| v.as_u64()).unwrap_or(255) as u8,
                                color_array.get(1).and_then(|v| v.as_u64()).unwrap_or(100) as u8,
                                color_array.get(2).and_then(|v| v.as_u64()).unwrap_or(100) as u8,
                                255,
                            )
                        },
                        user_data: UserData::default(),
                    };
                    sprite.tags.push(tag);
                }
            }
        }
        Ok(())
    }

    fn read_slices(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        if let Ok(mut file) = archive.by_name(OVS_SLICES_INDEX) {
            let slices_data: serde_json::Value = rmp_serde::from_read(&mut file)?;
            if let Some(slices_array) = slices_data.get("slices").and_then(|v| v.as_array()) {
                for slice_data in slices_array {
                    let mut slice = Slice {
                        uuid: Uuid::parse_str(slice_data.get("uuid").and_then(|v| v.as_str()).unwrap_or("")).unwrap_or_else(|_| Uuid::new_v4()),
                        name: slice_data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        keys: Vec::new(),
                        is_9patch: slice_data.get("is_9patch").and_then(|v| v.as_bool()).unwrap_or(false),
                        has_pivot: slice_data.get("has_pivot").and_then(|v| v.as_bool()).unwrap_or(false),
                        user_data: UserData::default(),
                    };

                    if let Some(keys_array) = slice_data.get("keys").and_then(|v| v.as_array()) {
                        for key_data in keys_array {
                            let key = SliceKey {
                                frame: key_data.get("frame").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                bounds: Rect::new(
                                    key_data.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    key_data.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    key_data.get("w").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    key_data.get("h").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                ),
                                is_9patch: slice.is_9patch,
                                center: key_data.get("center").and_then(|v| v.as_array()).map(|c| Rect::new(
                                    c.get(0).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    c.get(1).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    c.get(2).and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    c.get(3).and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                )),
                                pivot: key_data.get("pivot").and_then(|v| v.as_array()).map(|c| Point::new(
                                    c.get(0).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    c.get(1).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                )),
                                hidden: false,
                            };
                            slice.keys.push(key);
                        }
                    }

                    sprite.slices.push(slice);
                }
            }
        }
        Ok(())
    }

    fn read_tilesets(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        // Tilesets are stored in layer files or as separate files
        // For now, read from layer user_data or separate files
        let tileset_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .filter(|name| name.starts_with(OVS_LAYERS_DIR) && name.ends_with(".tileset"))
            .collect();

        for name in tileset_names {
            let mut file = archive.by_name(&name)?;
            let tileset: Tileset = rmp_serde::from_read(&mut file)?;
            sprite.tilesets.insert(tileset.uuid, tileset);
        }
        Ok(())
    }

    fn read_user_data(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        let user_data_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .filter(|name| name.starts_with(OVS_USER_DATA_DIR) && name.ends_with(".msgpack"))
            .collect();

        for name in user_data_names {
            let mut file = archive.by_name(&name)?;
            let user_data: UserData = rmp_serde::from_read(&mut file)?;
            // Merge with sprite user_data
            sprite.user_data.merge(user_data);
        }
        Ok(())
    }

    fn read_thumbnails(&self, archive: &mut ZipArchive<impl Read + Seek>, sprite: &mut Sprite) -> CoreResult<()> {
        let thumbnail_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .filter(|name| name.starts_with(OVS_THUMBNAILS_DIR) && name.ends_with(".png"))
            .collect();

        for name in thumbnail_names {
            let size_name = name.trim_start_matches(OVS_THUMBNAILS_DIR).trim_end_matches(".png");
            let mut file = archive.by_name(&name)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            sprite.thumbnails.insert(size_name.to_string(), data);
        }
        Ok(())
    }

    /// Read from file path
    pub fn read_file(&self, path: &std::path::Path) -> CoreResult<Sprite> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        self.read(&mut reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_creation() {
        let reader = OvsReader::new();
        assert!(reader.verify_checksums);
    }
}