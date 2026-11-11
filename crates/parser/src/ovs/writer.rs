//! Ovs file writer

use crate::ovs::*;
use crate::common::compress_lz4;
use oversprite_core::{
    CelType, CoreResult, LayerType, Sprite, AnimationDirection,
};
use std::io::{BufWriter, Write, Seek};
use std::path::Path;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

/// Ovs writer
pub struct OvsWriter {
    pub compression: OvsCompression,
    pub compression_level: i32,
}

impl Default for OvsWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl OvsWriter {
    pub fn new() -> Self {
        Self {
            compression: OvsCompression::Lz4,
            compression_level: 3,
        }
    }

    pub fn write<W: Write + Seek>(&self, sprite: &Sprite, writer: &mut W) -> CoreResult<()> {
        let mut zip = ZipWriter::new(writer);
        let options = FileOptions::default()
            .compression_method(self.zip_compression_method())
            .compression_level(Some(self.compression_level));

        // Write manifest
        self.write_manifest(&mut zip, sprite, &options)?;

        // Write metadata
        self.write_metadata(&mut zip, sprite, &options)?;

        // Write layers index
        self.write_layers_index(&mut zip, sprite, &options)?;

        // Write individual layer files
        self.write_layer_files(&mut zip, sprite, &options)?;

        // Write frames index
        self.write_frames_index(&mut zip, sprite, &options)?;

        // Write frame directories with cels and pixel data
        self.write_frame_files(&mut zip, sprite, &options)?;

        // Write palette
        self.write_palette_files(&mut zip, sprite, &options)?;

        // Write tags
        self.write_tags_index(&mut zip, sprite, &options)?;

        // Write slices
        self.write_slices_index(&mut zip, sprite, &options)?;

        // Write tilesets
        self.write_tileset_files(&mut zip, sprite, &options)?;

        // Write user data
        self.write_user_data_files(&mut zip, sprite, &options)?;

        // Write thumbnails
        self.write_thumbnail_files(&mut zip, sprite, &options)?;

        // Finish zip
        zip.finish()?;
        Ok(())
    }

    fn zip_compression_method(&self) -> CompressionMethod {
        match self.compression {
            OvsCompression::None => CompressionMethod::Stored,
            OvsCompression::Deflate => CompressionMethod::Deflated,
            OvsCompression::Lz4 => CompressionMethod::Deflated, // We handle LZ4 manually
            OvsCompression::Zstd => CompressionMethod::Deflated,
        }
    }

    fn write_manifest<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        let manifest = serde_json::json!({
            "magic": OVS_MAGIC,
            "format": OVS_FORMAT_ID,
            "version": sprite.header.version,
            "minor_version": sprite.header.minor_version,
            "created_by": sprite.header.created_by,
            "created_at": sprite.header.created_at,
            "modified_at": chrono::Utc::now().to_rfc3339(),
            "uuid": sprite.header.uuid.to_string(),
            "aseprite_compatible": sprite.header.aseprite_compatible,
            "schema": {
                "namespace": "oversprite.core.v1",
                "url": "https://oversprite.dev/schema/v1"
            },
            "features": sprite.header.features,
            "compression": {
                "metadata": "deflate",
                "pixels": "lz4",
                "tilemaps": "lz4"
            },
            "integrity": {
                "algorithm": "blake3",
                "manifest_hash": "" // Would compute hash
            }
        });

        let data = rmp_serde::to_vec(&manifest)?;
        zip.start_file(OVS_MANIFEST, *options)?;
        zip.write_all(&data)?;
        Ok(())
    }

    fn write_metadata<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        let data = rmp_serde::to_vec(&sprite.metadata)?;
        zip.start_file(OVS_METADATA, *options)?;
        zip.write_all(&data)?;
        Ok(())
    }

    fn write_layers_index<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        let data = rmp_serde::to_vec(&sprite.layers)?;
        zip.start_file(OVS_LAYERS_INDEX, *options)?;
        zip.write_all(&data)?;
        Ok(())
    }

    fn write_layer_files<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        for layer_uuid in &sprite.layers.order {
            if let Some(layer) = sprite.layer_data.get(layer_uuid) {
                let layer_name = format!("{}{}.msgpack", OVS_LAYERS_DIR, layer_uuid);
                let data = rmp_serde::to_vec(layer)?;
                zip.start_file(&layer_name, *options)?;
                zip.write_all(&data)?;

                // Write tileset file if this is a tilemap layer
                if layer.layer_type == LayerType::Tilemap {
                    if let Some(tileset_uuid) = layer.tileset_uuid {
                        if let Some(tileset) = sprite.tilesets.get(&tileset_uuid) {
                            let tileset_name = format!("{}{}.tileset", OVS_LAYERS_DIR, layer_uuid);
                            let data = rmp_serde::to_vec(tileset)?;
                            zip.start_file(&tileset_name, *options)?;
                            zip.write_all(&data)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn write_frames_index<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        let data = rmp_serde::to_vec(&sprite.frames)?;
        zip.start_file(OVS_FRAMES_INDEX, *options)?;
        zip.write_all(&data)?;
        Ok(())
    }

    fn write_frame_files<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        for frame_uuid in &sprite.frames.order {
            if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                let frame_dir = format!("{}{}/", OVS_FRAMES_DIR, frame_uuid);

                // Write cels index with frame duration
                let cels_data = serde_json::json!({
                    "duration_ms": frame.duration_ms,
                    "cels": frame.cels.iter().map(|cel_uuid| {
                        if let Some(cel) = sprite.cel_data.get(cel_uuid) {
                            serde_json::json!({
                                "uuid": cel.uuid.to_string(),
                                "layer_uuid": cel.layer_uuid.to_string(),
                                "x": cel.position.x,
                                "y": cel.position.y,
                                "opacity": cel.opacity,
                                "cel_type": cel.cel_type.to_u16(),
                                "z_index": cel.z_index,
                                "width": cel.bounds.w,
                                "height": cel.bounds.h,
                                "linked_frame_uuid": cel.linked_frame_uuid.map(|u| u.to_string()),
                                "tileset_uuid": cel.tileset_uuid.map(|u| u.to_string()),
                                "has_extra": cel.has_extra,
                            })
                        } else {
                            serde_json::json!({})
                        }
                    }).collect::<Vec<_>>()
                });

                let cels_name = format!("{}cels.msgpack", frame_dir);
                let data = rmp_serde::to_vec(&cels_data)?;
                zip.start_file(&cels_name, *options)?;
                zip.write_all(&data)?;

                // Write pixel data for each cel
                for cel_uuid in &frame.cels {
                    if let Some(cel) = sprite.cel_data.get(cel_uuid) {
                        match cel.cel_type {
                            CelType::Compressed | CelType::Raw => {
                                // Check both "pixels" and "raw_pixels"/"compressed_pixels" property names
                                let pixel_key = if cel.user_data.has_property("pixels") { "pixels" }
                                    else if cel.user_data.has_property("raw_pixels") { "raw_pixels" }
                                    else if cel.user_data.has_property("compressed_pixels") { "compressed_pixels" }
                                    else { "pixels" };

                                if let Some(pixels) = cel.user_data.get_property(pixel_key) {
                                    if let Some(bytes) = pixels.as_bytes() {
                                        let pixel_name = format!("{}{}.pixels.lz4", frame_dir, cel_uuid);
                                        let compressed = compress_lz4(&bytes)?;
                                        zip.start_file(&pixel_name, *options)?;
                                        zip.write_all(&compressed)?;
                                    }
                                }
                            }
                            CelType::Tilemap => {
                                // Check for "tilemap" or "tilemap_data" property
                                let tilemap_key = if cel.user_data.has_property("tilemap") { "tilemap" }
                                    else { "tilemap_data" };

                                if let Some(tilemap) = cel.user_data.get_property(tilemap_key) {
                                    if let Some(bytes) = tilemap.as_bytes() {
                                        let tilemap_name = format!("{}{}.tilemap.lz4", frame_dir, cel_uuid);
                                        let compressed = compress_lz4(&bytes)?;
                                        zip.start_file(&tilemap_name, *options)?;
                                        zip.write_all(&compressed)?;
                                    }
                                }
                            }
                            CelType::Linked => {
                                // No pixel data
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn write_palette_files<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        // Global palette
        let data = rmp_serde::to_vec(&sprite.global_palette)?;
        zip.start_file(OVS_PALETTE_GLOBAL, *options)?;
        zip.write_all(&data)?;

        // Per-frame palettes
        for (frame_uuid, palette) in &sprite.frame_palettes {
            let palette_name = format!("{}{}.msgpack", OVS_PALETTE_DIR, frame_uuid);
            let data = rmp_serde::to_vec(palette)?;
            zip.start_file(&palette_name, *options)?;
            zip.write_all(&data)?;
        }
        Ok(())
    }

    fn write_tags_index<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        let tags_data = serde_json::json!({
            "tags": sprite.tags.iter().map(|tag| serde_json::json!({
                "uuid": tag.uuid.to_string(),
                "name": tag.name,
                "from_frame": tag.from_frame,
                "to_frame": tag.to_frame,
                "direction": match tag.direction {
                    AnimationDirection::Forward => "forward",
                    AnimationDirection::Reverse => "reverse",
                    AnimationDirection::PingPong => "ping_pong",
                    AnimationDirection::PingPongReverse => "ping_pong_reverse",
                },
                "repeat": tag.repeat,
                "color": [tag.color.r, tag.color.g, tag.color.b],
            })).collect::<Vec<_>>()
        });

        let data = rmp_serde::to_vec(&tags_data)?;
        zip.start_file(OVS_TAGS_INDEX, *options)?;
        zip.write_all(&data)?;
        Ok(())
    }

    fn write_slices_index<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        let slices_data = serde_json::json!({
            "slices": sprite.slices.iter().map(|slice| serde_json::json!({
                "uuid": slice.uuid.to_string(),
                "name": slice.name,
                "is_9patch": slice.is_9patch,
                "has_pivot": slice.has_pivot,
                "keys": slice.keys.iter().map(|key| serde_json::json!({
                    "frame": key.frame,
                    "x": key.bounds.x,
                    "y": key.bounds.y,
                    "w": key.bounds.w,
                    "h": key.bounds.h,
                    "center": key.center.map(|c| [c.x, c.y, c.w as i32, c.h as i32]),
                    "pivot": key.pivot.map(|p| [p.x, p.y]),
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>()
        });

        let data = rmp_serde::to_vec(&slices_data)?;
        zip.start_file(OVS_SLICES_INDEX, *options)?;
        zip.write_all(&data)?;
        Ok(())
    }

    fn write_tileset_files<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        for tileset in sprite.tilesets.values() {
            let tileset_name = format!("{}{}.tileset", OVS_LAYERS_DIR, tileset.uuid);
            let data = rmp_serde::to_vec(tileset)?;
            zip.start_file(&tileset_name, *options)?;
            zip.write_all(&data)?;
        }
        Ok(())
    }

    fn write_user_data_files<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        // Write sprite-level user data
        if !sprite.user_data.properties.is_empty() || sprite.user_data.text.is_some() || sprite.user_data.color.is_some() {
            let data = rmp_serde::to_vec(&sprite.user_data)?;
            let name = format!("{}sprite.msgpack", OVS_USER_DATA_DIR);
            zip.start_file(&name, *options)?;
            zip.write_all(&data)?;
        }

        // Write layer user data
        for (layer_uuid, layer) in &sprite.layer_data {
            if !layer.user_data.properties.is_empty() || layer.user_data.text.is_some() || layer.user_data.color.is_some() {
                let data = rmp_serde::to_vec(&layer.user_data)?;
                let name = format!("{}layer_{}.msgpack", OVS_USER_DATA_DIR, layer_uuid);
                zip.start_file(&name, *options)?;
                zip.write_all(&data)?;
            }
        }

        Ok(())
    }

    fn write_thumbnail_files<W: Write + Seek>(&self, zip: &mut ZipWriter<W>, sprite: &Sprite, options: &FileOptions) -> CoreResult<()> {
        for (size, data) in &sprite.thumbnails {
            let name = format!("{}{}.png", OVS_THUMBNAILS_DIR, size);
            zip.start_file(&name, *options)?;
            zip.write_all(data)?;
        }
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
        let writer = OvsWriter::new();
        assert_eq!(writer.compression, OvsCompression::Lz4);
    }
}