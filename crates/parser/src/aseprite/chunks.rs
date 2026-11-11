//! Aseprite chunk parsers

use crate::common::*;
use crate::ParserResult;
use oversprite_core::{
    Cel, CelType, ColorMode, ColorProfile,
    Fixed16_16, Layer, LayerType, Palette, PaletteColor,
    Point, Rect, Rgba, Slice, SliceKey, Tag, AnimationDirection,
    Tileset, TilesetFlags, Uuid, UserData,
};
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Parse layer chunk (0x2004)
pub fn parse_layer_chunk<R: Read>(reader: &mut R, _color_mode: ColorMode) -> ParserResult<Layer> {
    let flags = read_u16(reader)?;
    let layer_type = layer_type_from_aseprite(read_u16(reader)?);
    let child_level = read_u16(reader)?;
    let _default_width = read_u16(reader)?;
    let _default_height = read_u16(reader)?;
    let blend_mode = blend_mode_from_aseprite(read_u16(reader)?);
    let opacity = read_bytes(reader, 1)?[0];
    let _reserved = read_bytes(reader, 3)?;
    let name = read_string(reader)?;

    let mut layer = Layer {
        uuid: Uuid::new_v4(),
        name,
        layer_type,
        parent: None,
        child_level,
        visible: (flags & 1) != 0,
        editable: (flags & 2) != 0,
        lock_movement: (flags & 4) != 0,
        is_background: (flags & 8) != 0,
        prefer_linked_cels: (flags & 16) != 0,
        collapsed: (flags & 32) != 0,
        is_reference: (flags & 64) != 0,
        reference_path: None,
        blend_mode,
        opacity,
        z_index: 0,
        tileset_uuid: None,
        default_width: 0,
        default_height: 0,
        user_data: UserData::default(),
    };

    // If tilemap layer, read tileset index
    if layer_type == LayerType::Tilemap {
        let tileset_index = read_u32(reader)?;
        // Store as user data for now, will resolve later
        layer.user_data.set_property("aseprite_tileset_index", tileset_index.into());
    }

    // If file has UUID flag, read UUID
    // This is determined by file header flags bit 4
    // For now we generate our own UUID

    Ok(layer)
}

/// Parse cel chunk (0x2005)
pub fn parse_cel_chunk<R: Read>(reader: &mut R, color_mode: ColorMode) -> ParserResult<Cel> {
    let layer_index = read_u16(reader)?;
    let x = read_i16(reader)?;
    let y = read_i16(reader)?;
    let opacity = read_bytes(reader, 1)?[0];
    let cel_type = cel_type_from_aseprite(read_u16(reader)?);
    let z_index = read_i16(reader)?;
    let _reserved = read_bytes(reader, 5)?;

    let mut cel = Cel {
        uuid: Uuid::new_v4(),
        layer_uuid: Uuid::nil(), // Will be resolved via layer_index
        position: Point::new(x as i32, y as i32),
        opacity,
        cel_type,
        z_index,
        bounds: Rect::empty(),
        linked_frame_uuid: None,
        tileset_uuid: None,
        has_extra: false,
        extra: Default::default(),
        user_data: UserData::default(),
    };

    // Store layer_index for later resolution
    cel.user_data.set_property("aseprite_layer_index", (layer_index as u32).into());

    match cel_type {
        CelType::Raw => {
            let width = read_u16(reader)?;
            let height = read_u16(reader)?;
            let bytes_per_pixel = color_mode.bytes_per_pixel() as u32;
            let pixel_data = read_bytes(reader, (width as u32 * height as u32 * bytes_per_pixel) as usize)?;
            cel.bounds = Rect::new(x as i32, y as i32, width as u32, height as u32);
            cel.user_data.set_property("raw_pixels", pixel_data.into());
        }
        CelType::Linked => {
            let linked_frame = read_u16(reader)?;
            cel.linked_frame_uuid = Some(Uuid::nil()); // Will resolve later
            cel.user_data.set_property("aseprite_linked_frame", (linked_frame as u32).into());
        }
        CelType::Compressed => {
            let width = read_u16(reader)?;
            let height = read_u16(reader)?;
            let _compressed_size = read_u32(reader)?; // Actually part of chunk size
            // Read remaining as compressed data
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;
            cel.bounds = Rect::new(x as i32, y as i32, width as u32, height as u32);
            cel.user_data.set_property("compressed_pixels", data.into());
        }
        CelType::Tilemap => {
            let width_tiles = read_u16(reader)?;
            let height_tiles = read_u16(reader)?;
            let bpp = read_u16(reader)?;
            let id_mask = read_u32(reader)?;
            let flip_x_mask = read_u32(reader)?;
            let flip_y_mask = read_u32(reader)?;
            let flip_d_mask = read_u32(reader)?;
            let _reserved = read_bytes(reader, 10)?;
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;

            cel.bounds = Rect::new(x as i32, y as i32, width_tiles as u32, height_tiles as u32);
            cel.user_data.set_property("tilemap_width", (width_tiles as u32).into());
            cel.user_data.set_property("tilemap_height", (height_tiles as u32).into());
            cel.user_data.set_property("tilemap_bpp", (bpp as u32).into());
            cel.user_data.set_property("tilemap_id_mask", id_mask.into());
            cel.user_data.set_property("tilemap_flip_x_mask", flip_x_mask.into());
            cel.user_data.set_property("tilemap_flip_y_mask", flip_y_mask.into());
            cel.user_data.set_property("tilemap_flip_d_mask", flip_d_mask.into());
            cel.user_data.set_property("tilemap_data", data.into());
        }
    }

    Ok(cel)
}

/// Parse cel extra chunk (0x2006)
pub fn parse_cel_extra_chunk<R: Read>(reader: &mut R) -> ParserResult<(bool, Fixed16_16, Fixed16_16, Fixed16_16, Fixed16_16)> {
    let flags = read_u32(reader)?;
    let has_bounds = (flags & 1) != 0;
    let precise_x = Fixed16_16::new(read_i32(reader)? as f32 / 65536.0);
    let precise_y = Fixed16_16::new(read_i32(reader)? as f32 / 65536.0);
    let precise_w = Fixed16_16::new(read_i32(reader)? as f32 / 65536.0);
    let precise_h = Fixed16_16::new(read_i32(reader)? as f32 / 65536.0);
    let _reserved = read_bytes(reader, 16)?;
    Ok((has_bounds, precise_x, precise_y, precise_w, precise_h))
}

/// Parse color profile chunk (0x2007)
pub fn parse_color_profile_chunk<R: Read>(reader: &mut R) -> ParserResult<ColorProfile> {
    let profile_type = read_u16(reader)?;
    let flags = read_u16(reader)?;
    let _fixed_gamma = read_i32(reader)? as f32 / 65536.0;
    let _reserved = read_bytes(reader, 8)?;

    let _use_fixed_gamma = (flags & 1) != 0;

    match profile_type {
        0 => Ok(ColorProfile::none()),
        1 => Ok(ColorProfile::srgb()),
        2 => {
            let icc_len = read_u32(reader)?;
            let icc_data = read_bytes(reader, icc_len as usize)?;
            Ok(ColorProfile::icc(icc_data))
        }
        _ => Ok(ColorProfile::none()),
    }
}

/// Parse external files chunk (0x2008)
pub fn parse_external_files_chunk<R: Read>(reader: &mut R) -> ParserResult<Vec<(Uuid, u8, String)>> {
    let count = read_u32(reader)?;
    let _reserved = read_bytes(reader, 8)?;
    let mut files = Vec::new();
    for _ in 0..count {
        let _entry_id = read_u32(reader)?;
        let file_type = read_bytes(reader, 1)?[0];
        let _reserved = read_bytes(reader, 7)?;
        let name = read_string(reader)?;
        files.push((Uuid::new_v4(), file_type, name));
    }
    Ok(files)
}

/// Parse tags chunk (0x2018)
pub fn parse_tags_chunk<R: Read>(reader: &mut R) -> ParserResult<Vec<Tag>> {
    let count = read_u16(reader)?;
    let _reserved = read_bytes(reader, 8)?;
    let mut tags = Vec::new();
    for _ in 0..count {
        let from = read_u16(reader)?;
        let to = read_u16(reader)?;
        let direction = match read_bytes(reader, 1)?[0] {
            0 => AnimationDirection::Forward,
            1 => AnimationDirection::Reverse,
            2 => AnimationDirection::PingPong,
            3 => AnimationDirection::PingPongReverse,
            _ => AnimationDirection::Forward,
        };
        let repeat = read_u16(reader)?;
        let _reserved = read_bytes(reader, 6)?;
        let r = read_bytes(reader, 1)?[0];
        let g = read_bytes(reader, 1)?[0];
        let b = read_bytes(reader, 1)?[0];
        let _extra = read_bytes(reader, 1)?;
        let name = read_string(reader)?;

        tags.push(Tag {
            uuid: Uuid::new_v4(),
            name,
            from_frame: from as u32,
            to_frame: to as u32,
            direction,
            repeat,
            color: Rgba::new(r, g, b, 255),
            user_data: UserData::default(),
        });
    }
    Ok(tags)
}

/// Parse palette chunk (0x2019)
pub fn parse_palette_chunk<R: Read>(reader: &mut R) -> ParserResult<Palette> {
    let _size = read_u32(reader)?;
    let first = read_u32(reader)?;
    let last = read_u32(reader)?;
    let _reserved = read_bytes(reader, 8)?;

    let mut colors = Vec::new();
    let count = (last - first + 1) as usize;
    for _i in 0..count {
        let entry_flags = read_u16(reader)?;
        let r = read_bytes(reader, 1)?[0];
        let g = read_bytes(reader, 1)?[0];
        let b = read_bytes(reader, 1)?[0];
        let a = read_bytes(reader, 1)?[0];
        let has_name = (entry_flags & 1) != 0;
        let name = if has_name { Some(read_string(reader)?) } else { None };

        colors.push(PaletteColor::new(r, g, b, a).with_name(name.unwrap_or_default()));
    }

    Ok(Palette {
        uuid: Uuid::new_v4(),
        name: "Imported".to_string(),
        colors,
        color_names: HashMap::new(),
        user_data: UserData::default(),
    })
}

/// Parse user data chunk (0x2020)
pub fn parse_user_data_chunk<R: Read>(reader: &mut R) -> ParserResult<UserData> {
    let flags = read_u32(reader)?;
    let mut user_data = UserData::default();

    if (flags & 1) != 0 {
        user_data.text = Some(read_string(reader)?);
    }
    if (flags & 2) != 0 {
        let r = read_bytes(reader, 1)?[0];
        let g = read_bytes(reader, 1)?[0];
        let b = read_bytes(reader, 1)?[0];
        let a = read_bytes(reader, 1)?[0];
        user_data.color = Some(Rgba::new(r, g, b, a));
    }
    if (flags & 4) != 0 {
        let _size = read_u32(reader)?;
        let num_maps = read_u32(reader)?;
        for _ in 0..num_maps {
            let _map_key = read_u32(reader)?;
            let num_props = read_u32(reader)?;
            for _ in 0..num_props {
                let name = read_string(reader)?;
                let prop_type = read_u16(reader)?;
                // Simplified: just store as string for now
                let _ = parse_property_value(reader, prop_type)?;
                user_data.set_property(name, "parsed".into());
            }
        }
    }

    Ok(user_data)
}

fn parse_property_value<R: Read>(reader: &mut R, prop_type: u16) -> ParserResult<()> {
    match prop_type {
        0x0001 => { read_bytes(reader, 1)?; } // bool
        0x0002 | 0x0003 => { read_bytes(reader, 1)?; } // int8/uint8
        0x0004 | 0x0005 => { read_bytes(reader, 2)?; } // int16/uint16
        0x0006 | 0x0007 => { read_bytes(reader, 4)?; } // int32/uint32
        0x0008 | 0x0009 => { read_bytes(reader, 8)?; } // int64/uint64
        0x000A => { read_bytes(reader, 4)?; } // FIXED
        0x000B | 0x000C => { read_bytes(reader, 4)?; } // FLOAT/DOUBLE
        0x000D => { read_string(reader)?; } // STRING
        0x000E => { read_bytes(reader, 8)?; } // POINT (2x i32)
        0x000F => { read_bytes(reader, 8)?; } // SIZE (2x u32)
        0x0010 => { read_bytes(reader, 16)?; } // RECT (4x i32)
        0x0011 => { // Vector
            let count = read_u32(reader)?;
            let elem_type = read_u16(reader)?;
            for _ in 0..count {
                parse_property_value(reader, elem_type)?;
            }
        }
        0x0012 => { // Nested map
            let count = read_u32(reader)?;
            for _ in 0..count {
                let _ = parse_property_value(reader, 0x0012)?;
            }
        }
        0x0013 => { read_bytes(reader, 16)?; } // UUID
        _ => {}
    }
    Ok(())
}

/// Parse slice chunk (0x2022)
pub fn parse_slice_chunk<R: Read>(reader: &mut R) -> ParserResult<Slice> {
    let num_keys = read_u32(reader)?;
    let flags = read_u32(reader)?;
    let _reserved = read_u32(reader)?;
    let name = read_string(reader)?;
    let is_9patch = (flags & 1) != 0;
    let has_pivot = (flags & 2) != 0;

    let mut slice = Slice {
        uuid: Uuid::new_v4(),
        name,
        keys: Vec::new(),
        is_9patch,
        has_pivot,
        user_data: UserData::default(),
    };

    for _ in 0..num_keys {
        let frame = read_u32(reader)?;
        let x = read_i32(reader)?;
        let y = read_i32(reader)?;
        let w = read_u32(reader)?;
        let h = read_u32(reader)?;

        let mut key = SliceKey::new(frame, Rect::new(x, y, w, h));

        if is_9patch {
            let cx = read_i32(reader)?;
            let cy = read_i32(reader)?;
            let cw = read_u32(reader)?;
            let ch = read_u32(reader)?;
            key.center = Some(Rect::new(cx, cy, cw, ch));
        }

        if has_pivot {
            let px = read_i32(reader)?;
            let py = read_i32(reader)?;
            key.pivot = Some(Point::new(px, py));
        }

        slice.keys.push(key);
    }

    Ok(slice)
}

/// Parse tileset chunk (0x2023)
pub fn parse_tileset_chunk<R: Read + Seek>(reader: &mut R) -> ParserResult<Tileset> {
    let _tileset_id = read_u32(reader)?;
    let flags = read_u32(reader)?;
    let tile_count = read_u32(reader)?;
    let tile_width = read_u16(reader)?;
    let tile_height = read_u16(reader)?;
    let base_index = read_i16(reader)?;
    let _reserved = read_bytes(reader, 14)?;
    let name = read_string(reader)?;

    let mut tileset = Tileset {
        uuid: Uuid::new_v4(),
        name,
        tile_width,
        tile_height,
        tile_count,
        base_index,
        flags: TilesetFlags::from_bits_truncate(flags),
        external_ref: None,
        image_uuid: None,
        tiles: Vec::new(),
        user_data: UserData::default(),
    };

    if (flags & 1) != 0 {
        let _file_id = read_u32(reader)?;
        let _ext_tileset_id = read_u32(reader)?;
        tileset.external_ref = Some(oversprite_core::tileset::ExternalTilesetRef {
            file_id: Uuid::new_v4(),
            tileset_id: Uuid::new_v4(),
        });
    }

    if (flags & 2) != 0 {
        let _data_len = read_u32(reader)?;
        // Image data follows, compressed with ZLIB
        // We'll store reference for later
        tileset.user_data.set_property("aseprite_tileset_image_data_offset", reader.stream_position().unwrap_or(0).into());
    }

    Ok(tileset)
}

/// Parse old palette chunks (0x0004, 0x0011)
pub fn parse_old_palette_chunk<R: Read>(reader: &mut R, chunk_type: u16) -> ParserResult<Palette> {
    let packets = read_u16(reader)?;
    let mut colors = Vec::new();

    for _ in 0..packets {
        let _skip = read_bytes(reader, 1)?[0];
        let count = read_bytes(reader, 1)?[0];
        let actual_count = if count == 0 { 256 } else { count as usize };

        for _ in 0..actual_count {
            let (r, g, b) = if chunk_type == 0x0004 {
                // 8-bit per channel
                (read_bytes(reader, 1)?[0], read_bytes(reader, 1)?[0], read_bytes(reader, 1)?[0])
            } else {
                // 6-bit per channel (0-63), scale to 8-bit
                let r6 = read_bytes(reader, 1)?[0];
                let g6 = read_bytes(reader, 1)?[0];
                let b6 = read_bytes(reader, 1)?[0];
                ((r6 * 255 / 63) as u8, (g6 * 255 / 63) as u8, (b6 * 255 / 63) as u8)
            };
            colors.push(PaletteColor::new(r, g, b, 255));
        }
    }

    // Pad to 256
    while colors.len() < 256 {
        colors.push(PaletteColor::default());
    }

    Ok(Palette {
        uuid: Uuid::new_v4(),
        name: "Legacy".to_string(),
        colors,
        color_names: HashMap::new(),
        user_data: UserData::default(),
    })
}