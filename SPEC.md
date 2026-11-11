# OverSprite File Format Specification (.ovs)

**Version**: 1.0  
**Status**: Draft  
**Author**: OverSprite Team  
**Date**: 2026-10-24

---

## 1. Overview

The **`.ovs`** (OverSprite) format is a modern, high-performance file format for pixel art and animation projects. It is designed as a successor to `.aseprite` with the following goals:

- **Performance**: Fast load/save via memory-mapped I/O and columnar storage
- **Compression**: ZIP-based container with per-chunk compression strategies
- **Extensibility**: Schema-driven with versioned namespaces
- **Interoperability**: Lossless round-trip with `.aseprite` (import/export)
- **Modern Tooling**: MessagePack for metadata, raw buffers for pixel data

---

## 2. Container Structure

The `.ovs` file is a **ZIP archive** (compatible with standard tools) containing:

```
project.ovs/
├── manifest.msgpack          # Required: File manifest & schema version
├── metadata.msgpack          # Required: Sprite metadata (dimensions, color mode, etc.)
├── layers/
│   ├── index.msgpack         # Required: Layer hierarchy & properties
│   ├── <layer-uuid>.msgpack  # Per-layer: blend mode, opacity, visibility, etc.
│   └── <layer-uuid>.tileset  # Optional: Tileset data (if layer is tilemap)
├── frames/
│   ├── index.msgpack         # Required: Frame list with durations
│   └── <frame-uuid>/
│       ├── cels.msgpack      # Required: Cel list for this frame
│       ├── <cel-uuid>.pixels # Raw pixel buffer (compressed per strategy)
│       └── <cel-uuid>.tilemap # Tilemap buffer (if cel is tilemap)
├── palette/
│   ├── global.msgpack        # Global palette (indexed mode)
│   └── <frame-uuid>.msgpack  # Per-frame palette overrides
├── tags/
│   └── index.msgpack         # Animation tags (name, range, direction, color)
├── slices/
│   └── index.msgpack         # 9-slice / export slices
├── user-data/
│   └── *.msgpack             # Extension-specific data (namespaced)
├── thumbnails/
│   └── <size>.png            # Pre-rendered thumbnails (64x64, 128x128, 256x256)
└── assets/
    └── *                     # Embedded assets (fonts, brushes, reference images)
```

### Compression Strategy per Entry

| Entry Type | Compression | Rationale |
|------------|-------------|-----------|
| `*.msgpack` | Deflate (level 3) | Small structured data, fast decompress |
| `*.pixels` | **LZ4** (or Zstd level 1) | Large raw buffers, prioritize speed |
| `*.tilemap` | LZ4 | Tile indices compress well |
| `thumbnails/*.png` | None (already compressed) | PNG is optimal |
| `assets/*` | Store (no compression) | Already compressed formats |

---

## 3. Manifest (`manifest.msgpack`)

```msgpack
{
  "magic": 0x53564F01,             // Format identifier (see below)
  "format": "ovs",
  "version": 1,                    // Major version (breaking changes)
  "minor_version": 0,              // Minor version (additive)
  "created_by": "OverSprite 0.1.0", // Generator identification
  "created_at": "2026-10-24T00:00:00Z", // ISO 8601 UTC
  "modified_at": "2026-10-24T00:00:00Z",
  "uuid": "550e8400-e29b-41d4-a716-446655440000", // Project UUID (RFC 4122)
  "aseprite_compatible": true,     // Can export to .aseprite losslessly
  "schema": {
    "namespace": "oversprite.core.v1",
    "url": "https://oversprite.dev/schema/v1"
  },
  "features": [                    // Optional features used
    "tilesets",
    "animation_tags",
    "slices",
    "user_data",
    "thumbnails"
  ],
  "compression": {
    "metadata": "deflate",
    "pixels": "lz4",
    "tilemaps": "lz4"
  },
  "integrity": {
    "algorithm": "blake3",
    "manifest_hash": "hex-encoded-blake3-of-manifest"
  }
}
```

### 3.1 Magic Number

The manifest carries a `magic` field holding the value `0x53564F01`
(`OVS_MAGIC` in `oversprite_parser`). It identifies the file as an
`.ovs` container and is verified on every read. A file whose manifest
carries any other value is rejected with an "Invalid .ovs magic number"
error, so an `.ovs` file can never be confused with a renamed
`.aseprite` file or an unrelated ZIP container.

The value lives in the manifest rather than in a file prefix so the
container keeps the standard ZIP signature (`PK\x03\x04`) and stays
readable with ordinary ZIP tooling.

---

## 4. Metadata (`metadata.msgpack`)

```msgpack
{
  "dimensions": {
    "width": 320,      // u32, max 65535
    "height": 180      // u32, max 65535
  },
  "color_mode": "rgba",          // "indexed" | "grayscale" | "rgba" | "grayscale_alpha"
  "color_depth": 32,             // 8 | 16 | 32 | 64 (bits per pixel)
  "transparent_index": 0,        // u8, only for indexed mode
  "background_color": [0,0,0,0], // RGBA u8[4]
  "pixel_aspect": {              // Pixel aspect ratio
    "width": 1,                  // u16
    "height": 1                  // u16
  },
  "grid": {                      // Optional grid
    "enabled": true,
    "origin": {"x": 0, "y": 0},  // i16
    "cell_size": {"w": 16, "h": 16} // u16
  },
  "color_profile": {             // Optional ICC/sRGB
    "type": "srgb",              // "none" | "srgb" | "icc"
    "icc_data": null             // bytes if type="icc"
  },
  "flags": {
    "layer_opacity_valid": true,
    "layer_blend_mode_valid": true,
    "layers_have_uuids": true
  }
}
```

### Color Mode Details

| Mode | Depth | Pixel Format | Palette |
|------|-------|--------------|---------|
| `indexed` | 8 | 1 byte index | Required (global + per-frame) |
| `grayscale` | 16 | 2 bytes (value, alpha) | Optional |
| `rgba` | 32 | 4 bytes (R,G,B,A) | Optional |
| `grayscale_alpha` | 16 | 2 bytes (value, alpha) | N/A |

---

## 5. Layers (`layers/index.msgpack` + `layers/<uuid>.msgpack`)

### Layer Index
```msgpack
{
  "order": ["uuid-1", "uuid-2", "uuid-3"], // Back-to-front render order
  "groups": {                              // Group hierarchy
    "uuid-group-1": ["uuid-2", "uuid-3"]   // Parent -> children
  }
}
```

### Per-Layer (`layers/<uuid>.msgpack`)
```msgpack
{
  "uuid": "550e8400-e29b-41d4-a716-446655440001",
  "name": "Background",
  "type": "image",              // "image" | "group" | "tilemap"
  "parent": null,               // Parent group UUID or null
  "child_level": 0,             // Nesting depth (0 = root)
  "visible": true,
  "editable": true,
  "lock_movement": false,
  "is_background": true,        // Only one per sprite
  "prefer_linked_cels": false,
  "collapsed": false,           // UI: group collapsed
  "is_reference": false,        // Reference layer (external file)
  "reference_path": null,       // Path if is_reference
  "blend_mode": 0,              // 0-19 (see Blend Mode Table)
  "opacity": 255,               // u8 (0-255)
  "z_index": 0,                 // i16 (per-cel override in frame)
  "tileset_uuid": null,         // UUID if type="tilemap"
  "user_data": {}               // Free-form properties
}
```

### Blend Mode Table (matches Aseprite)

| Value | Mode | Formula |
|-------|------|---------|
| 0 | Normal | `src` |
| 1 | Multiply | `src * dst` |
| 2 | Screen | `1 - (1-src)*(1-dst)` |
| 3 | Overlay | `src<0.5 ? 2*src*dst : 1-2*(1-src)*(1-dst)` |
| 4 | Darken | `min(src, dst)` |
| 5 | Lighten | `max(src, dst)` |
| 6 | Color Dodge | `dst / (1-src)` |
| 7 | Color Burn | `1 - (1-dst)/src` |
| 8 | Hard Light | `overlay(dst, src)` |
| 9 | Soft Light | `complex` |
| 10 | Difference | `abs(src-dst)` |
| 11 | Exclusion | `src+dst-2*src*dst` |
| 12 | Hue | `hsl_hue(src) + hsl_sat(dst) + hsl_lum(dst)` |
| 13 | Saturation | `hsl_hue(dst) + hsl_sat(src) + hsl_lum(dst)` |
| 14 | Color | `hsl_hue(src) + hsl_sat(src) + hsl_lum(dst)` |
| 15 | Luminosity | `hsl_hue(dst) + hsl_sat(dst) + hsl_lum(src)` |
| 16 | Addition | `min(src+dst, 1)` |
| 17 | Subtract | `max(src-dst, 0)` |
| 18 | Divide | `dst / src` |
| 19 | Source Atop | `src*dst_alpha + dst*(1-src_alpha)` |

---

## 6. Frames (`frames/index.msgpack` + `frames/<uuid>/`)

### Frame Index
```msgpack
{
  "order": ["frame-uuid-1", "frame-uuid-2", ...],
  "default_duration_ms": 100  // Fallback if frame has no duration
}
```

### Per-Frame Directory (`frames/<frame-uuid>/`)

#### `cels.msgpack`
```msgpack
{
  "cels": [
    {
      "uuid": "cel-uuid-1",
      "layer_uuid": "layer-uuid-1",
      "x": 0,                    // i32 (sub-pixel: FIXED 16.16 in extra)
      "y": 0,
      "opacity": 255,            // u8 (0-255), multiplies layer opacity
      "cel_type": "compressed",  // "raw" | "linked" | "compressed" | "tilemap"
      "z_index": 0,              // i16 (render order offset)
      "width": 320,              // u16 (cel bounds)
      "height": 180,
      "linked_frame_uuid": null, // If cel_type="linked"
      "tilemap_uuid": null,      // If cel_type="tilemap"
      "has_extra": false,        // True if extra.msgpack exists
      "user_data": {}
    }
  ]
}
```

#### `<cel-uuid>.pixels` (Raw Pixel Buffer)
**Format**: Raw bytes, row-major, top-to-bottom, left-to-right
- **RGBA**: `width * height * 4` bytes
- **Grayscale**: `width * height * 2` bytes (value, alpha)
- **Indexed**: `width * height * 1` byte (palette index)

**Compressed**: LZ4 frame format (with 4-byte uncompressed size prefix)

#### `<cel-uuid>.tilemap` (Tilemap Cel)
```msgpack
{
  "width_tiles": 20,           // u16
  "height_tiles": 15,          // u16
  "bpp": 32,                   // 8 | 16 | 32 (bits per tile)
  "tile_id_mask": 0x1FFFFFFF,  // u32
  "flip_x_mask": 0x20000000,   // u32
  "flip_y_mask": 0x40000000,   // u32
  "flip_d_mask": 0x80000000,   // u32 (diagonal/X-Y swap)
  "tiles": [ ... ]             // Compressed: LZ4(array of u32)
}
```

#### `<cel-uuid>.extra.msgpack` (Optional)
```msgpack
{
  "precise_bounds": true,
  "precise_x": 0.0,       // f32 (FIXED 16.16)
  "precise_y": 0.0,
  "precise_width": 320.0,
  "precise_height": 180.0
}
```

---

## 7. Palettes (`palette/global.msgpack` + `palette/<frame-uuid>.msgpack`)

### Global Palette
```msgpack
{
  "colors": [
    {"r": 0, "g": 0, "b": 0, "a": 255, "name": "black"},
    {"r": 255, "g": 255, "b": 255, "a": 255, "name": "white"}
    // ... up to 65536 entries
  ],
  "color_names": { "0": "black", "1": "white" } // Optional index->name
}
```

### Per-Frame Palette Override
```msgpack
{
  "base_palette": "global",   // Always "global" for v1
  "changes": {
    "5": {"r": 255, "g": 0, "b": 0, "a": 255},  // Index 5 changed
    "10": {"r": 0, "g": 255, "b": 0, "a": 255}
  }
}
```

---

## 8. Animation Tags (`tags/index.msgpack`)

```msgpack
{
  "tags": [
    {
      "uuid": "tag-uuid-1",
      "name": "walk",
      "from_frame": 0,           // Frame index in frames/order
      "to_frame": 7,             // Inclusive
      "direction": "forward",    // "forward" | "reverse" | "ping_pong" | "ping_pong_reverse"
      "repeat": 0,               // u16 (0 = infinite)
      "color": [255, 100, 100],  // RGB u8[3] for UI
      "user_data": {}
    }
  ]
}
```

---

## 9. Slices (`slices/index.msgpack`)

```msgpack
{
  "slices": [
    {
      "uuid": "slice-uuid-1",
      "name": "button",
      "keys": [
        {
          "frame": 0,           // Frame index where this key starts
          "bounds": {"x": 10, "y": 10, "w": 50, "h": 20}, // i32
          "is_9patch": true,
          "center": {"x": 15, "y": 5, "w": 20, "h": 10},  // 9-patch center
          "pivot": {"x": 25, "y": 10},                     // Pivot point
          "hidden": false
        }
      ],
      "user_data": {}
    }
  ]
}
```

---

## 10. Tilesets (Embedded in Layer or External)

### In `layers/<uuid>.tileset` (if layer type = tilemap)
```msgpack
{
  "uuid": "tileset-uuid-1",
  "name": "Overworld",
  "tile_width": 16,           // u16
  "tile_height": 16,          // u16
  "tile_count": 256,          // u32
  "base_index": 1,            // i16 (display offset)
  "flags": {
    "external_file": false,
    "tiles_in_file": true,
    "empty_tile_zero": true,   // Tile ID 0 = empty
    "auto_flip_x": true,
    "auto_flip_y": true,
    "auto_flip_d": true
  },
  "external_ref": null,       // If external_file=true: {file_id, tileset_id}
  "image_uuid": "cel-uuid-of-tileset-image" // Ref to frame/cel with tileset image
}
```

---

## 11. User Data (`user-data/*.msgpack`)

Namespaced by extension ID (`publisher/ExtensionName`):

```msgpack
{
  "aseprite/Attachment-System": {
    "attachments": [...]
  },
  "my-studio/CustomTool": {
    "tool_state": {...}
  }
}
```

---

## 12. Aseprite Compatibility Layer

### Import (.aseprite → .ovs)
1. Parse `.aseprite` using spec from `ase-file-specs.md`
2. Convert to internal `Sprite` model
3. Write as `.ovs` with `aseprite_compatible: true`
4. Preserve all chunk data (including unknown chunks in `user_data`)

### Export (.ovs → .aseprite)
1. Load `.ovs` into internal `Sprite` model
2. Serialize to `.aseprite` chunks:
   - Header → ASE header
   - Layers → Layer chunks (0x2004)
   - Frames + Cels → Frame chunks + Cel chunks (0x2005)
   - Palettes → Palette chunk (0x2019)
   - Tags → Tags chunk (0x2018)
   - Slices → Slice chunk (0x2022)
   - Tilesets → Tileset chunk (0x2023)
   - User data → User data chunks (0x2020)
3. **Lossless guarantee**: All core features map 1:1
4. **Extensions**: Store unknown data in User Data chunks

### Feature Parity Matrix

| Feature | .aseprite | .ovs | Notes |
|---------|-----------|------|-------|
| Layers (image/group/tilemap) | ✅ | ✅ | Full parity |
| Layer blend modes (20) | ✅ | ✅ | Identical values |
| Layer opacity | ✅ | ✅ | 0-255 |
| Layer UUIDs | ✅ | ✅ | RFC 4122 |
| Cels (raw/linked/compressed/tilemap) | ✅ | ✅ | Full parity |
| Cel Z-index | ✅ | ✅ | i16 |
| Precise bounds (sub-pixel) | ✅ | ✅ | FIXED 16.16 |
| Color profiles (sRGB/ICC) | ✅ | ✅ | Full parity |
| Palettes (RGBA + names) | ✅ | ✅ | Full parity |
| Per-frame palette | ✅ | ✅ | Override mechanism |
| Animation tags | ✅ | ✅ | Full parity |
| Slices (9-patch, pivot) | ✅ | ✅ | Full parity |
| Tilesets (with flip flags) | ✅ | ✅ | Full parity |
| External file refs | ✅ | ✅ | Via user_data |
| User data (text/color/props) | ✅ | ✅ | Full parity |
| Nested properties (maps/vectors) | ✅ | ✅ | MsgPack native |
| Custom extension data | ✅ | ✅ | Namespaced |

---

## 13. Versioning & Migration

### Version Scheme
- **Major**: Breaking changes (structural, UUID format, compression algorithm)
- **Minor**: Additive (new optional fields, new chunk types, new features)

### Migration Rules
1. **Reader**: Must support all minor versions ≤ current
2. **Writer**: Always writes current major.minor
3. **Unknown fields**: Preserve in `user_data["_unknown"]` for round-trip
4. **Deprecated fields**: Mark in schema, keep for 2 major versions

---

## 14. Performance Targets

| Operation | Target (1024x1024, 60 frames) |
|-----------|-------------------------------|
| Full load | < 200 ms |
| Full save | < 300 ms |
| Single frame load | < 10 ms |
| Thumbnail generation | < 50 ms |
| Memory usage (loaded) | < 200 MB |

---

## 15. Reference Implementation

The canonical implementation is in the `oversprite-core` crate:

```rust
// Core types
pub struct Sprite { ... }
pub struct Frame { ... }
pub struct Layer { ... }
pub struct Cel { ... }
pub struct Palette { ... }
pub struct Tag { ... }
pub struct Slice { ... }
pub struct Tileset { ... }

// I/O
pub fn read_ovs(path: &Path) -> Result<Sprite, OvsError>;
pub fn write_ovs(sprite: &Sprite, path: &Path) -> Result<(), OvsError>;
pub fn read_aseprite(path: &Path) -> Result<Sprite, AseError>;
pub fn write_aseprite(sprite: &Sprite, path: &Path) -> Result<(), AseError>;
```

---

## 16. Appendix: MessagePack Schema (Concise)

```msgpack
# All UUIDs are 16-byte arrays (RFC 4122)
# All timestamps are ISO 8601 strings
# All colors are [u8; 4] (R,G,B,A)
# All points/sizes/rects are {x,y} or {w,h} or {x,y,w,h} with i32
# FIXED 16.16 stored as f32
```

---

*End of Specification*