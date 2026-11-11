# OverSprite

A modern pixel art editor with a native **`.ovs`** file format.

OverSprite reads and writes `.aseprite` files and stores its own work in
`.ovs`, a ZIP container that holds MessagePack metadata and LZ4-compressed
pixel data, with a Blake3 integrity hash.

## Workspace layout

| Crate | Package | Purpose |
| --- | --- | --- |
| `crates/core` | `oversprite_core` | Shared data structures (sprites, layers, cels, palettes) |
| `crates/parser` | `oversprite_parser` | `.aseprite` and `.ovs` readers and writers |
| `crates/renderer` | `oversprite_renderer` | Software rasterizer built on tiny-skia |
| `crates/ui` | `oversprite_ui` | Editor window built with egui/eframe |
| `crates/scripting` | `oversprite_scripting` | Lua scripting and extension API (mlua 0.9) |
| `crates/cli` | `oversprite_cli` | `oversprite` command-line interface |

## Building

```sh
cargo build --workspace
cargo test --workspace
```

The editor binary is `oversprite-editor` and the CLI binary is `oversprite`.

## Command-line usage

```sh
# Create a new sprite in the native format
oversprite new -w 64 -H 64 --frames 4 sprite.ovs

# Inspect a file
oversprite info sprite.ovs
oversprite validate sprite.ovs

# Convert between formats
oversprite convert sprite.ovs sprite.aseprite
oversprite convert sprite.aseprite sprite.ovs

# Render frames to PNG
oversprite render sprite.ovs frame0.png
oversprite render-sequence sprite.ovs frames/ --format gif --fps 12

# Extract assets
oversprite extract sprite.ovs out/ --extract-type frames
oversprite thumbnail sprite.ovs thumbs/ --sizes 64,128,256

# Batch conversion
oversprite batch ./input ./output --format ovs --pattern "*.aseprite" --jobs 4
```

## The `.ovs` format

An `.ovs` file is a standard ZIP archive:

```
manifest.msgpack      Format identification, schema, integrity hash
metadata.msgpack      Sprite dimensions, color mode, timing
layers/index.msgpack  Layer order
layers/<uuid>.msgpack Layer definitions
frames/index.msgpack  Frame order
frames/<uuid>/cels.msgpack  Cel definitions and pixel references
palette/global.msgpack  Global palette
tags/index.msgpack    Animation tags
slices/index.msgpack  Slices
```

Pixel buffers are stored separately and compressed with LZ4. The manifest
carries the magic number `0x53564F01`, which the reader verifies on every
load. See [SPEC.md](SPEC.md) for the full specification.

## License

MIT OR Apache-2.0
