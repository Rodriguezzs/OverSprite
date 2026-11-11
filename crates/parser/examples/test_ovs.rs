//! OverSprite OVS Round-trip Test
//!
//! This example creates a sprite programmatically, saves it as .ovs,
//! loads it back, and validates that all data matches.

use oversprite_core::{
    BlendMode, Cel, CelType, ColorMode, Frame, Layer, LayerType, Metadata, Palette, PaletteColor,
    Point, Rect, Rgba, Size, Sprite, Uuid, UserData, PropertyValue, AnimationDirection,
};
use oversprite_parser::ovs::{read_ovs, write_ovs};
use std::path::Path;

/// Compare two PropertyValues by their numeric value for integer types,
/// or exact match for other types. This handles the MessagePack limitation
/// where integer enum variants (Int8, UInt8, Int16, etc.) are not preserved.
fn property_values_equal(a: Option<&PropertyValue>, b: Option<&PropertyValue>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => match (a, b) {
            // For integer types, compare the numeric value regardless of variant
            (PropertyValue::Int8(a), PropertyValue::Int8(b)) => a == b,
            (PropertyValue::Int8(a), PropertyValue::UInt8(b)) => *a as u8 == *b,
            (PropertyValue::Int8(a), PropertyValue::Int16(b)) => *a as i16 == *b,
            (PropertyValue::Int8(a), PropertyValue::UInt16(b)) => *a as u16 == *b,
            (PropertyValue::Int8(a), PropertyValue::Int32(b)) => *a as i32 == *b,
            (PropertyValue::Int8(a), PropertyValue::UInt32(b)) => *a as u32 == *b,
            (PropertyValue::Int8(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::Int8(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::UInt8(a), PropertyValue::Int8(b)) => *a == *b as u8,
            (PropertyValue::UInt8(a), PropertyValue::UInt8(b)) => a == b,
            (PropertyValue::UInt8(a), PropertyValue::Int16(b)) => *a as i16 == *b,
            (PropertyValue::UInt8(a), PropertyValue::UInt16(b)) => *a as u16 == *b,
            (PropertyValue::UInt8(a), PropertyValue::Int32(b)) => *a as i32 == *b,
            (PropertyValue::UInt8(a), PropertyValue::UInt32(b)) => *a as u32 == *b,
            (PropertyValue::UInt8(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::UInt8(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::Int16(a), PropertyValue::Int8(b)) => *a == *b as i16,
            (PropertyValue::Int16(a), PropertyValue::UInt8(b)) => *a as u16 == *b as u16,
            (PropertyValue::Int16(a), PropertyValue::Int16(b)) => a == b,
            (PropertyValue::Int16(a), PropertyValue::UInt16(b)) => *a as u16 == *b,
            (PropertyValue::Int16(a), PropertyValue::Int32(b)) => *a as i32 == *b,
            (PropertyValue::Int16(a), PropertyValue::UInt32(b)) => *a as u32 == *b,
            (PropertyValue::Int16(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::Int16(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::UInt16(a), PropertyValue::Int8(b)) => *a as i16 == *b as i16,
            (PropertyValue::UInt16(a), PropertyValue::UInt8(b)) => *a == *b as u16,
            (PropertyValue::UInt16(a), PropertyValue::Int16(b)) => *a as i16 == *b,
            (PropertyValue::UInt16(a), PropertyValue::UInt16(b)) => a == b,
            (PropertyValue::UInt16(a), PropertyValue::Int32(b)) => *a as i32 == *b,
            (PropertyValue::UInt16(a), PropertyValue::UInt32(b)) => *a as u32 == *b,
            (PropertyValue::UInt16(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::UInt16(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::Int32(a), PropertyValue::Int8(b)) => *a == *b as i32,
            (PropertyValue::Int32(a), PropertyValue::UInt8(b)) => *a as u32 == *b as u32,
            (PropertyValue::Int32(a), PropertyValue::Int16(b)) => *a == *b as i32,
            (PropertyValue::Int32(a), PropertyValue::UInt16(b)) => *a as u32 == *b as u32,
            (PropertyValue::Int32(a), PropertyValue::Int32(b)) => a == b,
            (PropertyValue::Int32(a), PropertyValue::UInt32(b)) => *a as u32 == *b,
            (PropertyValue::Int32(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::Int32(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::UInt32(a), PropertyValue::Int8(b)) => *a as i32 == *b as i32,
            (PropertyValue::UInt32(a), PropertyValue::UInt8(b)) => *a == *b as u32,
            (PropertyValue::UInt32(a), PropertyValue::Int16(b)) => *a as i32 == *b as i32,
            (PropertyValue::UInt32(a), PropertyValue::UInt16(b)) => *a == *b as u32,
            (PropertyValue::UInt32(a), PropertyValue::Int32(b)) => *a as i32 == *b,
            (PropertyValue::UInt32(a), PropertyValue::UInt32(b)) => a == b,
            (PropertyValue::UInt32(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::UInt32(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::Int64(a), PropertyValue::Int8(b)) => *a == *b as i64,
            (PropertyValue::Int64(a), PropertyValue::UInt8(b)) => *a as u64 == *b as u64,
            (PropertyValue::Int64(a), PropertyValue::Int16(b)) => *a == *b as i64,
            (PropertyValue::Int64(a), PropertyValue::UInt16(b)) => *a as u64 == *b as u64,
            (PropertyValue::Int64(a), PropertyValue::Int32(b)) => *a == *b as i64,
            (PropertyValue::Int64(a), PropertyValue::UInt32(b)) => *a as u64 == *b as u64,
            (PropertyValue::Int64(a), PropertyValue::Int64(b)) => a == b,
            (PropertyValue::Int64(a), PropertyValue::UInt64(b)) => *a as u64 == *b,

            (PropertyValue::UInt64(a), PropertyValue::Int8(b)) => *a as i64 == *b as i64,
            (PropertyValue::UInt64(a), PropertyValue::UInt8(b)) => *a == *b as u64,
            (PropertyValue::UInt64(a), PropertyValue::Int16(b)) => *a as i64 == *b as i64,
            (PropertyValue::UInt64(a), PropertyValue::UInt16(b)) => *a == *b as u64,
            (PropertyValue::UInt64(a), PropertyValue::Int32(b)) => *a as i64 == *b as i64,
            (PropertyValue::UInt64(a), PropertyValue::UInt32(b)) => *a == *b as u64,
            (PropertyValue::UInt64(a), PropertyValue::Int64(b)) => *a as i64 == *b,
            (PropertyValue::UInt64(a), PropertyValue::UInt64(b)) => a == b,

            // For other types, use exact equality
            (a, b) => a == b,
        },
        (None, None) => true,
        _ => false,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OverSprite OVS Round-trip Test ===\n");

    // Create a test sprite
    let mut sprite = create_test_sprite();

    // Validate before saving
    sprite.validate()?;
    println!("✓ Original sprite validated successfully");
    println!("  Dimensions: {}x{}", sprite.metadata.dimensions.w, sprite.metadata.dimensions.h);
    println!("  Color mode: {:?}", sprite.metadata.color_mode);
    println!("  Layers: {}", sprite.layer_data.len());
    println!("  Frames: {}", sprite.frame_data.len());
    println!("  Cels: {}", sprite.cel_data.len());
    println!("  Tags: {}", sprite.tags.len());

    // Save to .ovs file
    let test_file = Path::new("/tmp/oversprite_roundtrip_test.ovs");
    println!("\n→ Saving sprite to {:?}...", test_file);
    write_ovs(&sprite, test_file)?;
    println!("✓ Sprite saved successfully");

    // Load back from .ovs file
    println!("\n→ Loading sprite from {:?}...", test_file);
    let loaded_sprite = read_ovs(test_file)?;
    println!("✓ Sprite loaded successfully");

    // Validate loaded sprite
    loaded_sprite.validate()?;
    println!("✓ Loaded sprite validated successfully");

    // Compare original and loaded sprites
    println!("\n=== Round-trip Validation ===");
    let all_ok = validate_roundtrip(&sprite, &loaded_sprite)?;

    if all_ok {
        println!("\n✅ ===== ROUND-TRIP TEST PASSED =====");
        println!("All metadata and pixel data match perfectly!");
    } else {
        println!("\n❌ ===== ROUND-TRIP TEST FAILED =====");
        return Err("Round-trip validation failed".into());
    }

    // Cleanup
    std::fs::remove_file(test_file).ok();

    Ok(())
}

/// Creates a test sprite with multiple layers, frames, and pixel data
fn create_test_sprite() -> Sprite {
    let mut sprite = Sprite::new_with_size(64, 64, ColorMode::Rgba);

    // Set some metadata
    sprite.metadata.dimensions = Size::new(64, 64);
    sprite.metadata.color_mode = ColorMode::Rgba;
    sprite.metadata.default_frame_duration_ms = 100;

    // Create a global palette (for RGBA mode it's less critical, but still useful)
    let mut palette = Palette::default();
    palette.colors = vec![
        PaletteColor { r: 0, g: 0, b: 0, a: 255, name: Some("Black".into()) },
        PaletteColor { r: 255, g: 255, b: 255, a: 255, name: Some("White".into()) },
        PaletteColor { r: 255, g: 0, b: 0, a: 255, name: Some("Red".into()) },
        PaletteColor { r: 0, g: 255, b: 0, a: 255, name: Some("Green".into()) },
        PaletteColor { r: 0, g: 0, b: 255, a: 255, name: Some("Blue".into()) },
    ];
    sprite.global_palette = palette;

    // --- Layer 1: Background (Red) ---
    let bg_layer = Layer::new_image("Background");
    let bg_layer_uuid = bg_layer.uuid;
    sprite.layer_data.insert(bg_layer_uuid, bg_layer);
    sprite.layers.add_layer(bg_layer_uuid, None);

    // --- Layer 2: Foreground (Blue square) ---
    let fg_layer = Layer::new_image("Foreground");
    let fg_layer_uuid = fg_layer.uuid;
    sprite.layer_data.insert(fg_layer_uuid, fg_layer);
    sprite.layers.add_layer(fg_layer_uuid, None);

    // --- Layer 3: Detail layer (Green dots) ---
    let detail_layer = Layer::new_image("Detail");
    let detail_layer_uuid = detail_layer.uuid;
    sprite.layer_data.insert(detail_layer_uuid, detail_layer);
    sprite.layers.add_layer(detail_layer_uuid, None);

    // Create 3 frames
    for frame_idx in 0..3 {
        let frame = Frame::new(100);
        let frame_uuid = frame.uuid;
        sprite.frame_data.insert(frame_uuid, frame);
        sprite.frames.add_frame(frame_uuid);

        // Frame 0: Full red background
        // Frame 1: Red background with blue square moving
        // Frame 2: Red background with blue square + green dots

        // Background cel (same in all frames - solid red)
        let bg_cel = create_solid_cel(
            bg_layer_uuid,
            Point::new(0, 0),
            Rect::new(0, 0, 64, 64),
            Rgba::new(255, 0, 0, 255), // Red
            sprite.metadata.color_mode,
        );
        let bg_cel_uuid = bg_cel.uuid;
        sprite.cel_data.insert(bg_cel_uuid, bg_cel);
        sprite.frame_data.get_mut(&frame_uuid).unwrap().add_cel(bg_cel_uuid);

        // Foreground cel (blue square at different positions per frame)
        let fg_x = (frame_idx as i32 * 16) + 16; // 16, 32, 48
        let fg_y = 16;
        let fg_cel = create_solid_cel(
            fg_layer_uuid,
            Point::new(fg_x, fg_y),
            Rect::new(fg_x, fg_y, 16, 16),
            Rgba::new(0, 0, 255, 255), // Blue
            sprite.metadata.color_mode,
        );
        let fg_cel_uuid = fg_cel.uuid;
        sprite.cel_data.insert(fg_cel_uuid, fg_cel);
        sprite.frame_data.get_mut(&frame_uuid).unwrap().add_cel(fg_cel_uuid);

        // Detail cel (green dots in frame 2 only)
        if frame_idx == 2 {
            let detail_cel = create_pattern_cel(
                detail_layer_uuid,
                Point::new(0, 0),
                Rect::new(0, 0, 64, 64),
                |x, y| {
                    if (x % 8 == 0 && y % 8 == 0) {
                        Rgba::new(0, 255, 0, 255) // Green dots
                    } else {
                        Rgba::TRANSPARENT
                    }
                },
                sprite.metadata.color_mode,
            );
            let detail_cel_uuid = detail_cel.uuid;
            sprite.cel_data.insert(detail_cel_uuid, detail_cel);
            sprite.frame_data.get_mut(&frame_uuid).unwrap().add_cel(detail_cel_uuid);
        }
    }

    // Add an animation tag
    let tag = oversprite_core::Tag {
        uuid: Uuid::new_v4(),
        name: "Test Animation".into(),
        from_frame: 0,
        to_frame: 2,
        direction: AnimationDirection::Forward,
        repeat: 0,
        color: Rgba::new(255, 255, 0, 255), // Yellow tag color
        user_data: UserData::default(),
    };
    sprite.tags.push(tag);

    // Remove the default layer and frame that Sprite::new() adds
    // (We'll add our own layers and frames)
    if let Some(default_layer_uuid) = sprite.layers.order.first().copied() {
        sprite.layer_data.remove(&default_layer_uuid);
        sprite.layers.order.remove(0);
    }
    if let Some(default_frame_uuid) = sprite.frames.order.first().copied() {
        sprite.frame_data.remove(&default_frame_uuid);
        sprite.frames.order.remove(0);
    }

    // Add user data to sprite
    sprite.user_data.set_property("test_property", PropertyValue::String("roundtrip_test".into()));
    sprite.user_data.set_property("test_number", PropertyValue::UInt32(300)); // Use 300 to avoid i8 ambiguity

    sprite
}

/// Creates a cel filled with a solid color
fn create_solid_cel(
    layer_uuid: Uuid,
    position: Point,
    bounds: Rect,
    color: Rgba,
    color_mode: ColorMode,
) -> Cel {
    let width = bounds.w;
    let height = bounds.h;
    let bytes_per_pixel = match color_mode {
        ColorMode::Indexed => 1,
        ColorMode::Grayscale => 2,
        ColorMode::Rgba => 4,
        ColorMode::GrayscaleAlpha => 2,
    };

    // Create pixel data
    let mut pixels = Vec::with_capacity((width * height * bytes_per_pixel) as usize);

    for _y in 0..height {
        for _x in 0..width {
            match color_mode {
                ColorMode::Indexed => {
                    // For indexed, we'd need a palette index - use 1 (first color after transparent)
                    pixels.push(1);
                }
                ColorMode::Grayscale => {
                    // Convert to grayscale
                    let gray = ((color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000) as u8;
                    pixels.push(gray);
                    pixels.push(color.a);
                }
                ColorMode::Rgba => {
                    pixels.push(color.r);
                    pixels.push(color.g);
                    pixels.push(color.b);
                    pixels.push(color.a);
                }
                ColorMode::GrayscaleAlpha => {
                    let gray = ((color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000) as u8;
                    pixels.push(gray);
                    pixels.push(color.a);
                }
            }
        }
    }

    let mut cel = Cel::new(layer_uuid, position, bounds);
    cel.cel_type = CelType::Raw; // Raw pixel data
    cel.user_data.set_property("pixels", pixels_to_property_value(&pixels));

    cel
}

/// Creates a cel with a pattern function
fn create_pattern_cel<F>(
    layer_uuid: Uuid,
    position: Point,
    bounds: Rect,
    pattern_fn: F,
    color_mode: ColorMode,
) -> Cel
where
    F: Fn(u32, u32) -> Rgba,
{
    let width = bounds.w;
    let height = bounds.h;
    let bytes_per_pixel = match color_mode {
        ColorMode::Indexed => 1,
        ColorMode::Grayscale => 2,
        ColorMode::Rgba => 4,
        ColorMode::GrayscaleAlpha => 2,
    };

    let mut pixels = Vec::with_capacity((width * height * bytes_per_pixel) as usize);

    for y in 0..height {
        for x in 0..width {
            let color = pattern_fn(x, y);
            match color_mode {
                ColorMode::Indexed => {
                    // Use 0 for transparent, 1+ for palette indices
                    if color.a == 0 {
                        pixels.push(0);
                    } else {
                        pixels.push(2); // Green index in our test palette
                    }
                }
                ColorMode::Grayscale => {
                    let gray = ((color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000) as u8;
                    pixels.push(gray);
                    pixels.push(color.a);
                }
                ColorMode::Rgba => {
                    pixels.push(color.r);
                    pixels.push(color.g);
                    pixels.push(color.b);
                    pixels.push(color.a);
                }
                ColorMode::GrayscaleAlpha => {
                    let gray = ((color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000) as u8;
                    pixels.push(gray);
                    pixels.push(color.a);
                }
            }
        }
    }

    let mut cel = Cel::new(layer_uuid, position, bounds);
    cel.cel_type = CelType::Raw;
    cel.user_data.set_property("pixels", pixels_to_property_value(&pixels));

    cel
}

/// Converts a Vec<u8> to PropertyValue::Array of UInt8
fn pixels_to_property_value(pixels: &[u8]) -> PropertyValue {
    PropertyValue::Array(pixels.iter().map(|&b| PropertyValue::UInt8(b)).collect())
}

/// Validates that the round-trip preserved all data
fn validate_roundtrip(original: &Sprite, loaded: &Sprite) -> Result<bool, Box<dyn std::error::Error>> {
    let mut all_ok = true;

    // 1. Check metadata
    println!("\n1. Checking metadata...");
    if original.metadata.dimensions != loaded.metadata.dimensions {
        println!("  ❌ Dimensions mismatch: {:?} vs {:?}",
            original.metadata.dimensions, loaded.metadata.dimensions);
        all_ok = false;
    } else {
        println!("  ✓ Dimensions match: {}x{}",
            original.metadata.dimensions.w, original.metadata.dimensions.h);
    }

    if original.metadata.color_mode != loaded.metadata.color_mode {
        println!("  ❌ Color mode mismatch: {:?} vs {:?}",
            original.metadata.color_mode, loaded.metadata.color_mode);
        all_ok = false;
    } else {
        println!("  ✓ Color mode matches: {:?}", original.metadata.color_mode);
    }

    if original.metadata.default_frame_duration_ms != loaded.metadata.default_frame_duration_ms {
        println!("  ❌ Frame duration mismatch");
        all_ok = false;
    } else {
        println!("  ✓ Frame duration matches");
    }

    // 2. Check layers
    println!("\n2. Checking layers...");
    if original.layer_data.len() != loaded.layer_data.len() {
        println!("  ❌ Layer count mismatch: {} vs {}",
            original.layer_data.len(), loaded.layer_data.len());
        all_ok = false;
    } else {
        println!("  ✓ Layer count matches: {}", original.layer_data.len());

        // Check each layer
        for (uuid, orig_layer) in &original.layer_data {
            if let Some(loaded_layer) = loaded.layer_data.get(uuid) {
                if orig_layer.name != loaded_layer.name {
                    println!("  ❌ Layer name mismatch for {}: {} vs {}", uuid, orig_layer.name, loaded_layer.name);
                    all_ok = false;
                } else if orig_layer.layer_type != loaded_layer.layer_type {
                    println!("  ❌ Layer type mismatch for {}: {:?} vs {:?}", uuid, orig_layer.layer_type, loaded_layer.layer_type);
                    all_ok = false;
                } else if orig_layer.blend_mode != loaded_layer.blend_mode {
                    println!("  ❌ Layer blend mode mismatch for {}", uuid);
                    all_ok = false;
                } else if orig_layer.opacity != loaded_layer.opacity {
                    println!("  ❌ Layer opacity mismatch for {}", uuid);
                    all_ok = false;
                } else {
                    println!("  ✓ Layer '{}' matches", orig_layer.name);
                }
            } else {
                println!("  ❌ Layer {} not found in loaded sprite", uuid);
                all_ok = false;
            }
        }
    }

    // 3. Check frames
    println!("\n3. Checking frames...");
    if original.frame_data.len() != loaded.frame_data.len() {
        println!("  ❌ Frame count mismatch: {} vs {}",
            original.frame_data.len(), loaded.frame_data.len());
        all_ok = false;
    } else {
        println!("  ✓ Frame count matches: {}", original.frame_data.len());

        for (uuid, orig_frame) in &original.frame_data {
            if let Some(loaded_frame) = loaded.frame_data.get(uuid) {
                if orig_frame.duration_ms != loaded_frame.duration_ms {
                    println!("  ❌ Frame duration mismatch for {}", uuid);
                    all_ok = false;
                } else if orig_frame.cels.len() != loaded_frame.cels.len() {
                    println!("  ❌ Frame cel count mismatch for {}: {} vs {}", uuid, orig_frame.cels.len(), loaded_frame.cels.len());
                    all_ok = false;
                } else {
                    println!("  ✓ Frame {} matches ({} cels)", uuid, orig_frame.cels.len());
                }
            } else {
                println!("  ❌ Frame {} not found in loaded sprite", uuid);
                all_ok = false;
            }
        }
    }

    // 4. Check cels and pixel data
    println!("\n4. Checking cels and pixel data...");
    for (uuid, orig_cel) in &original.cel_data {
        if let Some(loaded_cel) = loaded.cel_data.get(uuid) {
            // Check cel metadata
            if orig_cel.layer_uuid != loaded_cel.layer_uuid {
                println!("  ❌ Cel {} layer_uuid mismatch", uuid);
                all_ok = false;
            } else if orig_cel.position != loaded_cel.position {
                println!("  ❌ Cel {} position mismatch: {:?} vs {:?}", uuid, orig_cel.position, loaded_cel.position);
                all_ok = false;
            } else if orig_cel.bounds != loaded_cel.bounds {
                println!("  ❌ Cel {} bounds mismatch: {:?} vs {:?}", uuid, orig_cel.bounds, loaded_cel.bounds);
                all_ok = false;
            } else if orig_cel.opacity != loaded_cel.opacity {
                println!("  ❌ Cel {} opacity mismatch", uuid);
                all_ok = false;
            } else if orig_cel.cel_type != loaded_cel.cel_type {
                println!("  ❌ Cel {} type mismatch: {:?} vs {:?}", uuid, orig_cel.cel_type, loaded_cel.cel_type);
                all_ok = false;
            } else {
                // Check pixel data
                let orig_pixels = orig_cel.user_data.get_property("pixels")
                    .and_then(|p| p.as_bytes());
                let loaded_pixels = loaded_cel.user_data.get_property("pixels")
                    .and_then(|p| p.as_bytes());

                match (orig_pixels, loaded_pixels) {
                    (Some(orig), Some(loaded)) => {
                        if orig == loaded {
                            println!("  ✓ Cel {} pixel data matches ({} bytes)", uuid, orig.len());
                        } else {
                            println!("  ❌ Cel {} pixel data mismatch! {} vs {} bytes", uuid, orig.len(), loaded.len());
                            // Show first few bytes for debugging
                            println!("    Original:  {:?}", &orig[..orig.len().min(16)]);
                            println!("    Loaded:    {:?}", &loaded[..loaded.len().min(16)]);
                            all_ok = false;
                        }
                    }
                    (None, None) => {
                        println!("  ⚠ Cel {} has no pixel data (expected for linked cels)", uuid);
                    }
                    _ => {
                        println!("  ❌ Cel {} pixel data presence mismatch", uuid);
                        all_ok = false;
                    }
                }
            }
        } else {
            println!("  ❌ Cel {} not found in loaded sprite", uuid);
            all_ok = false;
        }
    }

    // 5. Check palette
    println!("\n5. Checking palette...");
    if original.global_palette.colors.len() != loaded.global_palette.colors.len() {
        println!("  ❌ Palette color count mismatch: {} vs {}",
            original.global_palette.colors.len(), loaded.global_palette.colors.len());
        all_ok = false;
    } else {
        let palette_match = original.global_palette.colors.iter()
            .zip(loaded.global_palette.colors.iter())
            .all(|(o, l)| o.r == l.r && o.g == l.g && o.b == l.b && o.a == l.a);
        if palette_match {
            println!("  ✓ Palette matches ({} colors)", original.global_palette.colors.len());
        } else {
            println!("  ❌ Palette colors mismatch");
            all_ok = false;
        }
    }

    // 6. Check tags
    println!("\n6. Checking animation tags...");
    if original.tags.len() != loaded.tags.len() {
        println!("  ❌ Tag count mismatch: {} vs {}", original.tags.len(), loaded.tags.len());
        all_ok = false;
    } else {
        println!("  ✓ Tag count matches: {}", original.tags.len());
        for (i, (orig, loaded)) in original.tags.iter().zip(loaded.tags.iter()).enumerate() {
            if orig.name == loaded.name
                && orig.from_frame == loaded.from_frame
                && orig.to_frame == loaded.to_frame
                && orig.direction == loaded.direction {
                println!("  ✓ Tag {} '{}' matches", i, orig.name);
            } else {
                println!("  ❌ Tag {} mismatch", i);
                all_ok = false;
            }
        }
    }

    // 7. Check user data
    println!("\n7. Checking user data...");
    let orig_test_prop = original.user_data.get_property("test_property");
    let loaded_test_prop = loaded.user_data.get_property("test_property");
    if property_values_equal(orig_test_prop, loaded_test_prop) {
        println!("  ✓ User data 'test_property' matches");
    } else {
        println!("  ❌ User data 'test_property' mismatch: {:?} vs {:?}", orig_test_prop, loaded_test_prop);
        all_ok = false;
    }

    let orig_test_num = original.user_data.get_property("test_number");
    let loaded_test_num = loaded.user_data.get_property("test_number");
    if property_values_equal(orig_test_num, loaded_test_num) {
        println!("  ✓ User data 'test_number' matches");
    } else {
        println!("  ❌ User data 'test_number' mismatch: {:?} vs {:?}", orig_test_num, loaded_test_num);
        all_ok = false;
    }

    Ok(all_ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ovs_roundtrip() {
        let mut sprite = create_test_sprite();
        sprite.validate().unwrap();

        let test_file = Path::new("/tmp/oversprite_test_roundtrip.ovs");
        write_ovs(&sprite, test_file).unwrap();
        let loaded = read_ovs(test_file).unwrap();
        loaded.validate().unwrap();

        assert!(validate_roundtrip(&sprite, &loaded).unwrap());

        std::fs::remove_file(test_file).ok();
    }
}