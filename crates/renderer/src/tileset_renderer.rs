//! OverSprite Renderer - Tileset Renderer Module
//!
//! Handles rendering of tilemap layers using tilesets.

use oversprite_core::{
    BlendMode, CoreError, CoreResult, Point, Rect, Tileset,
};
use tiny_skia::{Pixmap, PixmapPaint, Transform, PremultipliedColorU8};
use crate::blend::to_tiny_skia_blend_mode;

// Tile flags for flipping (matching Aseprite)
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TileFlags: u32 {
        const FLIP_X = 0x80000000;
        const FLIP_Y = 0x40000000;
        const FLIP_DIAGONAL = 0x20000000; // Diagonal flip (rotate 90 + flip X)
        const ID_MASK = 0x1FFFFFFF;
    }
}

/// Render a tilemap to the target pixmap
pub fn render_tilemap(
    target: &mut Pixmap,
    tileset_pixmap: &Pixmap,
    tilemap_data: &[u8],
    tileset: &Tileset,
    position: Point,
    blend_mode: BlendMode,
    opacity: u8,
) -> CoreResult<()> {
    // Parse tilemap data
    // Format: each tile is 4 bytes (tile ID with flip flags)
    // We need to know the tilemap dimensions from the cel bounds
    let tile_width = tileset.tile_width as f32;
    let tile_height = tileset.tile_height as f32;
    let tiles_per_row = tileset.tile_count;

    if tiles_per_row == 0 {
        return Ok(());
    }

    let blend_mode_ts = to_tiny_skia_blend_mode(blend_mode);
    let opacity_f = opacity as f32 / 255.0;

    let paint = PixmapPaint {
        blend_mode: blend_mode_ts,
        opacity: opacity_f,
        ..Default::default()
    };

    // Tilemap data is stored as u32 per tile (tile ID + flip flags)
    // Each tile entry is 4 bytes
    if tilemap_data.len() % 4 != 0 {
        return Err(CoreError::ValidationError("Invalid tilemap data length".into()));
    }

    let _num_tiles = tilemap_data.len() / 4;

    // Parse each tile
    for (i, chunk) in tilemap_data.chunks_exact(4).enumerate() {
        let tile_id_bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
        let raw_id = u32::from_le_bytes(tile_id_bytes);

        let flags = TileFlags::from_bits_truncate(raw_id);
        let tile_id = (raw_id & TileFlags::ID_MASK.bits()) as usize;

        if tile_id >= tileset.tile_count as usize {
            continue; // Empty tile or invalid
        }

        // Calculate tile position in tilemap grid
        // We need the tilemap grid dimensions - for now use a simple row layout
        // In reality, this should come from the cel's tilemap_width/tilemap_height properties
        let tiles_per_row = tileset.tile_count.max(1);
        let tx = (i % tiles_per_row as usize) as u32;
        let ty = (i / tiles_per_row as usize) as u32;

        // Source rect in tileset
        let src_x = (tile_id % tiles_per_row as usize) as u32 * tileset.tile_width as u32;
        let src_y = (tile_id / tiles_per_row as usize) as u32 * tileset.tile_height as u32;

        let src_rect = tiny_skia::Rect::from_xywh(
            src_x as f32, src_y as f32,
            tile_width, tile_height
        );

        if src_rect.is_none() {
            continue;
        }
        let src_rect = src_rect.unwrap();

        // Destination position
        let dst_x = position.x as f32 + tx as f32 * tile_width;
        let dst_y = position.y as f32 + ty as f32 * tile_height;

        let mut transform = Transform::from_translate(dst_x, dst_y);

        // Apply flip transforms
        if flags.contains(TileFlags::FLIP_DIAGONAL) {
            // Diagonal flip: transpose (swap x and y)
            transform = Transform::from_row(
                0.0, -1.0,  // sx=0, ky=-1
                1.0, 0.0,   // kx=1, sy=0
                dst_x + tile_width, dst_y  // tx, ty
            );
        } else {
            if flags.contains(TileFlags::FLIP_X) {
                transform = transform.post_scale(-1.0, 1.0);
                transform = transform.post_translate(-tile_width, 0.0);
            }
            if flags.contains(TileFlags::FLIP_Y) {
                transform = transform.post_scale(1.0, -1.0);
                transform = transform.post_translate(0.0, -tile_height);
            }
        }

        // Apply source rect cropping to transform (pre_translate by negative source position)
        let transform = transform.pre_translate(-src_rect.x(), -src_rect.y());
        target.draw_pixmap(0, 0, tileset_pixmap.as_ref(), &paint, transform, None);
    }

    Ok(())
}

/// Render a single tile from a tileset
pub fn render_tile(
    target: &mut Pixmap,
    tileset_pixmap: &Pixmap,
    tileset: &Tileset,
    tile_id: usize,
    position: Point,
    flip_x: bool,
    flip_y: bool,
    flip_diagonal: bool,
    blend_mode: BlendMode,
    opacity: u8,
) -> CoreResult<()> {
    if tile_id >= tileset.tile_count as usize {
        return Ok(());
    }

    let tile_width = tileset.tile_width as f32;
    let tile_height = tileset.tile_height as f32;
    let tiles_per_row = tileset.tile_count.max(1);

    let src_x = (tile_id % tiles_per_row as usize) as u32 * tileset.tile_width as u32;
    let src_y = (tile_id / tiles_per_row as usize) as u32 * tileset.tile_height as u32;

    let src_rect = tiny_skia::Rect::from_xywh(
        src_x as f32, src_y as f32,
        tile_width, tile_height
    );

    if src_rect.is_none() {
        return Ok(());
    }
    let src_rect = src_rect.unwrap();

    let dst_x = position.x as f32;
    let dst_y = position.y as f32;

    let mut transform = Transform::from_translate(dst_x, dst_y);

    if flip_diagonal {
        // Diagonal flip (transpose)
        transform = Transform::from_row(
            0.0, -1.0,  // sx=0, ky=-1
            1.0, 0.0,   // kx=1, sy=0
            dst_x + tile_width, dst_y  // tx, ty
        );
    } else {
        if flip_x {
            transform = transform.post_scale(-1.0, 1.0);
            transform = transform.post_translate(-tile_width, 0.0);
        }
        if flip_y {
            transform = transform.post_scale(1.0, -1.0);
            transform = transform.post_translate(0.0, -tile_height);
        }
    }

    let paint = PixmapPaint {
        blend_mode: to_tiny_skia_blend_mode(blend_mode),
        opacity: opacity as f32 / 255.0,
        ..Default::default()
    };

    // Apply source rect cropping to transform (pre_translate by negative source position)
    let transform = transform.pre_translate(-src_rect.x(), -src_rect.y());
    target.draw_pixmap(0, 0, tileset_pixmap.as_ref(), &paint, transform, None);

    Ok(())
}

/// Create a tileset pixmap from raw pixel data
pub fn create_tileset_pixmap(
    pixels: &[u8],
    width: u32,
    height: u32,
    color_mode: oversprite_core::ColorMode,
    palette: &oversprite_core::Palette,
) -> CoreResult<Pixmap> {
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| CoreError::Internal("Failed to create tileset pixmap".into()))?;

    let bytes_per_pixel = match color_mode {
        oversprite_core::ColorMode::Indexed => 1,
        oversprite_core::ColorMode::Grayscale => 2,
        oversprite_core::ColorMode::Rgba => 4,
        oversprite_core::ColorMode::GrayscaleAlpha => 2,
    };

    let expected_len = (width * height * bytes_per_pixel) as usize;
    if pixels.len() < expected_len {
        return Err(CoreError::PixelDataSizeMismatch { expected: expected_len, actual: pixels.len() });
    }

    match color_mode {
        oversprite_core::ColorMode::Indexed => {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let palette_idx = pixels[idx] as usize;
                    let color = palette.colors.get(palette_idx)
                        .cloned()
                        .map(|c| c.to_rgba())
                        .unwrap_or(oversprite_core::Rgba::TRANSPARENT);
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(color.r, color.g, color.b, color.a)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
        oversprite_core::ColorMode::Grayscale => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 2) as usize;
                    let gray = pixels[idx];
                    let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
        oversprite_core::ColorMode::Rgba => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    let r = pixels[idx];
                    let g = pixels.get(idx + 1).copied().unwrap_or(0);
                    let b = pixels.get(idx + 2).copied().unwrap_or(0);
                    let a = pixels.get(idx + 3).copied().unwrap_or(255);
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(r, g, b, a)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
        oversprite_core::ColorMode::GrayscaleAlpha => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 2) as usize;
                    let gray = pixels[idx];
                    let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
    }

    Ok(pixmap)
}

/// Calculate tilemap bounds from tile data
pub fn calculate_tilemap_bounds(
    tilemap_data: &[u8],
    tileset: &Tileset,
    position: Point,
) -> Rect {
    if tilemap_data.is_empty() {
        return Rect::new(position.x, position.y, 0, 0);
    }

    let num_tiles = tilemap_data.len() / 4;
    let tiles_per_row = tileset.tile_count.max(1);

    let _max_tx = 0;
    let _max_ty = 0;

    // Find max tile coordinates
    for (i, _) in tilemap_data.chunks_exact(4).enumerate() {
        let _tx = (i % tiles_per_row as usize) as u32;
        let _ty = (i / tiles_per_row as usize) as u32;
    }

    let width = ((num_tiles as u32 + tiles_per_row - 1) / tiles_per_row) * tileset.tile_width as u32;
    let height = tileset.tile_height as u32 * ((num_tiles as u32 + tiles_per_row - 1) / tiles_per_row).max(1);

    Rect::new(position.x, position.y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_flags() {
        let flags = TileFlags::FLIP_X | TileFlags::FLIP_Y;
        assert!(flags.contains(TileFlags::FLIP_X));
        assert!(flags.contains(TileFlags::FLIP_Y));
        assert!(!flags.contains(TileFlags::FLIP_DIAGONAL));
    }

    #[test]
    fn test_tile_id_mask() {
        let raw_id = 0x80000001u32; // FLIP_X | tile_id 1
        let flags = TileFlags::from_bits_truncate(raw_id);
        let tile_id = raw_id & TileFlags::ID_MASK.bits();
        assert_eq!(tile_id, 1);
        assert!(flags.contains(TileFlags::FLIP_X));
    }
}