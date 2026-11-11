//! OverSprite CLI
//!
//! Command-line interface for conversion, validation, batch processing,
//! and headless rendering of .ovs and .aseprite files.

use clap::{Parser, Subcommand, ValueEnum};
use oversprite_core::{CoreError, CoreResult, Sprite, ColorMode, Size};
use oversprite_parser::{read_sprite, write_sprite, OvsWriter};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Parser)]
#[command(name = "oversprite")]
#[command(about = "OverSprite - Modern pixel art editor and .aseprite/.ovs converter", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert between .aseprite and .ovs formats
    Convert {
        /// Input file path
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output file path (extension determines format)
        #[arg(value_name = "OUTPUT")]
        output: PathBuf,

        /// Force overwrite existing file
        #[arg(short, long)]
        force: bool,

        /// Compression level for .ovs (0-9)
        #[arg(short, long, default_value = "3")]
        compression: u32,
    },

    /// Validate a sprite file
    Validate {
        /// File to validate
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Strict validation (fail on warnings)
        #[arg(short, long)]
        strict: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Batch convert multiple files
    Batch {
        /// Input directory
        #[arg(value_name = "INPUT_DIR")]
        input_dir: PathBuf,

        /// Output directory
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,

        /// Target format
        #[arg(short = 'f', long, value_enum, default_value = "ovs")]
        format: TargetFormat,

        /// File pattern to match
        #[arg(short = 'p', long, default_value = "*.aseprite")]
        pattern: String,

        /// Number of parallel jobs
        #[arg(short = 'j', long, default_value = "4")]
        jobs: usize,

        /// Force overwrite
        #[arg(short = 'F', long)]
        force: bool,
    },

    /// Render sprite to image
    Render {
        /// Input sprite file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output image file
        #[arg(value_name = "OUTPUT")]
        output: PathBuf,

        /// Frame index to render (default: 0)
        #[arg(short, long, default_value = "0")]
        frame: usize,

        /// Scale factor
        #[arg(short, long, default_value = "1.0")]
        scale: f32,

        /// Show grid
        #[arg(long)]
        grid: bool,

        /// Show onion skin
        #[arg(long)]
        onion_skin: bool,

        /// Background color (hex)
        #[arg(long, default_value = "transparent")]
        background: String,
    },

    /// Render all frames as sequence
    RenderSequence {
        /// Input sprite file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output directory
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,

        /// Output format (png, gif, webp)
        #[arg(short, long, value_enum, default_value = "png")]
        format: ImageFormat,

        /// Scale factor
        #[arg(short, long, default_value = "1.0")]
        scale: f32,

        /// Frame rate for GIF/WebP
        #[arg(long, default_value = "12")]
        fps: u32,

        /// Show grid
        #[arg(long)]
        grid: bool,

        /// Loop animation (for GIF/WebP)
        #[arg(long)]
        loop_anim: bool,
    },

    /// Show sprite information
    Info {
        /// File to inspect
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Show detailed layer info
        #[arg(long)]
        layers: bool,

        /// Show detailed frame info
        #[arg(long)]
        frames: bool,

        /// Show palette info
        #[arg(long)]
        palette: bool,
    },

    /// Create new sprite
    New {
        /// Output file path
        #[arg(value_name = "OUTPUT")]
        output: PathBuf,

        /// Width in pixels
        #[arg(short = 'w', long, default_value = "128")]
        width: u32,

        /// Height in pixels
        #[arg(short = 'H', long, default_value = "128")]
        height: u32,

        /// Color mode
        #[arg(short, long, value_enum, default_value = "rgba")]
        color_mode: ColorModeArg,

        /// Number of frames
        #[arg(short, long, default_value = "1")]
        frames: u32,
    },

    /// Extract assets from sprite
    Extract {
        /// Input sprite file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output directory
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,

        /// What to extract
        #[arg(short, long, value_enum, default_value = "all")]
        extract_type: ExtractType,

        /// Frame indices (comma-separated, default: all)
        #[arg(long)]
        frames: Option<String>,
    },

    /// Generate thumbnails
    Thumbnail {
        /// Input sprite file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output directory
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,

        /// Sizes to generate (comma-separated, e.g., 64,128,256)
        #[arg(short, long, default_value = "64,128,256")]
        sizes: String,
    },

    /// Run a script
    Run {
        /// Script file
        #[arg(value_name = "SCRIPT")]
        script: PathBuf,

        /// Input sprite file (optional)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output sprite file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Manage extensions
    Extension {
        #[command(subcommand)]
        command: ExtensionCommand,
    },
}

#[derive(Subcommand)]
enum ExtensionCommand {
    /// Install extension from directory
    Install {
        /// Extension directory
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },

    /// List installed extensions
    List,

    /// Remove extension
    Remove {
        /// Extension name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Create new extension template
    Create {
        /// Extension name
        #[arg(value_name = "NAME")]
        name: String,

        /// Output directory
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(ValueEnum, Clone, Debug)]
enum TargetFormat {
    Ovs,
    Aseprite,
}

#[derive(ValueEnum, Clone, Debug)]
enum ImageFormat {
    Png,
    Gif,
    Webp,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ExtractType {
    All,
    Layers,
    Frames,
    Palettes,
    Tilesets,
    Tags,
    Slices,
    UserData,
}

#[derive(ValueEnum, Clone, Debug)]
enum ColorModeArg {
    Rgba,
    Grayscale,
    Indexed,
    GrayscaleAlpha,
}

impl From<ColorModeArg> for ColorMode {
    fn from(arg: ColorModeArg) -> Self {
        match arg {
            ColorModeArg::Rgba => ColorMode::Rgba,
            ColorModeArg::Grayscale => ColorMode::Grayscale,
            ColorModeArg::Indexed => ColorMode::Indexed,
            ColorModeArg::GrayscaleAlpha => ColorMode::GrayscaleAlpha,
        }
    }
}

fn main() -> CoreResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert { input, output, force, compression } => {
            convert(input, output, force, compression)
        }
        Commands::Validate { file, strict, format } => {
            validate(file, strict, format)
        }
        Commands::Batch { input_dir, output_dir, format, pattern, jobs, force } => {
            batch(input_dir, output_dir, format, pattern, jobs, force)
        }
        Commands::Render { input, output, frame, scale, grid, onion_skin, background } => {
            render(input, output, frame, scale, grid, onion_skin, background)
        }
        Commands::RenderSequence { input, output_dir, format, scale, fps, grid, loop_anim } => {
            render_sequence(input, output_dir, format, scale, fps, grid, loop_anim)
        }
        Commands::Info { file, format, layers, frames, palette } => {
            info(file, format, layers, frames, palette)
        }
        Commands::New { output, width, height, color_mode, frames } => {
            new(output, width, height, color_mode, frames)
        }
        Commands::Extract { input, output_dir, extract_type, frames } => {
            extract(input, output_dir, extract_type, frames)
        }
        Commands::Thumbnail { input, output_dir, sizes } => {
            thumbnail(input, output_dir, sizes)
        }
        Commands::Run { script, input, output, args } => {
            run_script(script, input, output, args)
        }
        Commands::Extension { command } => {
            extension_command(command)
        }
    }
}

fn convert(input: PathBuf, output: PathBuf, force: bool, compression: u32) -> CoreResult<()> {
    if output.exists() && !force {
        return Err(CoreError::Internal(format!("Output file exists: {:?}. Use -f to overwrite.", output)));
    }

    println!("Converting {:?} -> {:?}", input, output);
    let sprite = read_sprite(&input)?;

    if let Some(mut ovs_writer) = (output.extension().and_then(|s| s.to_str()) == Some("ovs")).then(|| OvsWriter::new()) {
        ovs_writer.compression_level = compression as i32;
        ovs_writer.write_file(&sprite, &output)?;
    } else {
        write_sprite(&sprite, &output)?;
    }

    println!("Conversion complete!");
    Ok(())
}

fn validate(file: PathBuf, strict: bool, format: OutputFormat) -> CoreResult<()> {
    println!("Validating {:?}...", file);
    let sprite = read_sprite(&file)?;
    sprite.validate()?;

    // Collect structural problems that `Sprite::validate` does not cover.
    let mut warnings: Vec<String> = Vec::new();

    for (index, layer_uuid) in sprite.layers.order.iter().enumerate() {
        match sprite.layer_data.get(layer_uuid) {
            Some(_) => {}
            None => warnings.push(format!("layer {} references a missing layer record", index)),
        }
    }

    for (index, frame_uuid) in sprite.frames.order.iter().enumerate() {
        let Some(frame) = sprite.frame_data.get(frame_uuid) else {
            warnings.push(format!("frame {} references a missing frame record", index));
            continue;
        };
        for cel_uuid in &frame.cels {
            if !sprite.cel_data.contains_key(cel_uuid) {
                warnings.push(format!(
                    "frame {} references a missing cel record",
                    index
                ));
            }
        }
    }

    if sprite.frames.order.is_empty() {
        warnings.push("sprite has no frames".to_string());
    }
    if sprite.layers.order.is_empty() {
        warnings.push("sprite has no layers".to_string());
    }

    let info = ValidationInfo {
        valid: warnings.is_empty(),
        file: file.to_string_lossy().to_string(),
        size: format!("{}x{}", sprite.metadata.dimensions.w, sprite.metadata.dimensions.h),
        color_mode: format!("{:?}", sprite.metadata.color_mode),
        frames: sprite.frames.order.len(),
        layers: sprite.layers.order.len(),
        tags: sprite.tags.len(),
        slices: sprite.slices.len(),
        tilesets: sprite.tilesets.len(),
        warnings: warnings.clone(),
    };

    match format {
        OutputFormat::Text => {
            if warnings.is_empty() {
                println!("✓ Valid sprite file");
            } else {
                println!("⚠ Sprite file loaded with warnings");
            }
            println!("  Size: {}x{}", sprite.metadata.dimensions.w, sprite.metadata.dimensions.h);
            println!("  Color mode: {:?}", sprite.metadata.color_mode);
            println!("  Frames: {}", sprite.frames.order.len());
            println!("  Layers: {}", sprite.layers.order.len());
            println!("  Tags: {}", sprite.tags.len());
            println!("  Slices: {}", sprite.slices.len());
            println!("  Tilesets: {}", sprite.tilesets.len());
            for warning in &warnings {
                println!("  ! {warning}");
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&info)
                .map_err(|e| CoreError::Internal(e.to_string()))?);
        }
    }

    if strict && !warnings.is_empty() {
        return Err(CoreError::Internal(format!(
            "{} validation warning(s) with --strict",
            warnings.len()
        )));
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct ValidationInfo {
    valid: bool,
    file: String,
    size: String,
    color_mode: String,
    frames: usize,
    layers: usize,
    tags: usize,
    slices: usize,
    tilesets: usize,
    warnings: Vec<String>,
}

fn batch(input_dir: PathBuf, output_dir: PathBuf, format: TargetFormat, pattern: String, jobs: usize, force: bool) -> CoreResult<()> {
    fs::create_dir_all(&output_dir)?;

    let files: Vec<_> = glob::glob(&input_dir.join(&pattern).to_string_lossy())
        .map_err(|e| CoreError::Internal(e.to_string()))?
        .filter_map(Result::ok)
        .collect();

    println!("Found {} files to convert", files.len());

    use rayon::prelude::*;

    // `jobs` controls the worker count; rayon defaults to one thread per core.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs.max(1))
        .build()
        .map_err(|e| CoreError::Internal(e.to_string()))?;

    let mut failures: Vec<(PathBuf, String)> = Vec::new();

    pool.install(|| {
        files.par_iter().for_each(|input| {
            let output_name = input.file_stem().unwrap().to_string_lossy();
            let ext = match format {
                TargetFormat::Ovs => "ovs",
                TargetFormat::Aseprite => "aseprite",
            };
            let output = output_dir.join(format!("{}.{}", output_name, ext));

            if let Err(e) = convert(input.clone(), output, force, 3) {
                eprintln!("Failed to convert {:?}: {}", input, e);
            }
        });
    });

    for input in &files {
        let output_name = input.file_stem().unwrap().to_string_lossy();
        let ext = match format {
            TargetFormat::Ovs => "ovs",
            TargetFormat::Aseprite => "aseprite",
        };
        let output = output_dir.join(format!("{}.{}", output_name, ext));
        if !output.exists() {
            failures.push((input.clone(), output.display().to_string()));
        }
    }

    if !failures.is_empty() {
        return Err(CoreError::Internal(format!(
            "{} of {} files failed to convert",
            failures.len(),
            files.len()
        )));
    }

    println!("Batch conversion complete!");
    Ok(())
}

fn render(input: PathBuf, output: PathBuf, frame: usize, scale: f32, grid: bool, onion_skin: bool, background: String) -> CoreResult<()> {
    println!("Rendering frame {} of {:?} -> {:?}", frame, input, output);
    let sprite = read_sprite(&input)?;

    if frame >= sprite.frames.order.len() {
        return Err(CoreError::Internal(format!("Frame index {} out of range (max {})", frame, sprite.frames.order.len() - 1)));
    }

    let background = parse_background(&background)?;

    let mut renderer = oversprite_renderer::Renderer::with_background(
        Size::new(
            sprite.metadata.dimensions.w,
            sprite.metadata.dimensions.h,
        ),
        scale,
        background,
    )?;

    let onion = if onion_skin {
        Some(oversprite_renderer::OnionSkinConfig {
            enabled: true,
            ..Default::default()
        })
    } else {
        None
    };

    render_frame_with(&mut renderer, &sprite, frame, onion.as_ref())?;

    if grid {
        draw_grid(&mut renderer, &sprite);
    }

    // Save image
    renderer.save_png(&output)?;
    println!("Render complete!");
    Ok(())
}

/// Parse a background color from `#RRGGBB`, `#RRGGBBAA` or a hex integer.
fn parse_background(value: &str) -> CoreResult<oversprite_core::Rgba> {
    let value = value.trim().trim_start_matches('#');
    if value.is_empty() {
        return Ok(oversprite_core::Rgba::TRANSPARENT);
    }

    let parse = |s: &str| u8::from_str_radix(s, 16).map_err(|_| {
        CoreError::Internal(format!("Invalid background color '{}'", value))
    });

    match value.len() {
        6 => Ok(oversprite_core::Rgba::new(
            parse(&value[0..2])?,
            parse(&value[2..4])?,
            parse(&value[4..6])?,
            255,
        )),
        8 => Ok(oversprite_core::Rgba::new(
            parse(&value[0..2])?,
            parse(&value[2..4])?,
            parse(&value[4..6])?,
            parse(&value[6..8])?,
        )),
        _ => Err(CoreError::Internal(format!(
            "Invalid background color '{}': expected #RRGGBB or #RRGGBBAA",
            value
        ))),
    }
}

/// Overlay the sprite's grid onto the rendered image.
fn draw_grid(renderer: &mut oversprite_renderer::Renderer, sprite: &oversprite_core::Sprite) {
    renderer.draw_grid(&sprite.metadata.grid, oversprite_core::Rgba::new(255, 255, 255, 60));
}

fn render_sequence(input: PathBuf, output_dir: PathBuf, format: ImageFormat, scale: f32, fps: u32, grid: bool, loop_anim: bool) -> CoreResult<()> {
    println!("Rendering sequence from {:?} -> {:?}", input, output_dir);
    let sprite = read_sprite(&input)?;
    fs::create_dir_all(&output_dir)?;

    let mut renderer = oversprite_renderer::Renderer::new(
        Size::new(
            sprite.metadata.dimensions.w,
            sprite.metadata.dimensions.h,
        ),
        scale,
    )?;

    for (i, _frame_uuid) in sprite.frames.order.iter().enumerate() {
        render_frame(&mut renderer, &sprite, i)?;
        if grid {
            draw_grid(&mut renderer, &sprite);
        }
        let frame_path = output_dir.join(format!("frame_{:04}.png", i));
        renderer.save_png(&frame_path)?;
    }

    // If GIF or WebP, create animation
    if matches!(format, ImageFormat::Gif | ImageFormat::Webp) {
        create_animation(&output_dir, &format, fps, loop_anim)?;
    }

    println!("Sequence render complete!");
    Ok(())
}

/// Render a single frame of a sprite into the renderer canvas.
fn render_frame(
    renderer: &mut oversprite_renderer::Renderer,
    sprite: &oversprite_core::Sprite,
    frame: usize,
) -> CoreResult<()> {
    render_frame_with(renderer, sprite, frame, None)
}

/// Render a single frame, optionally with onion skin enabled.
fn render_frame_with(
    renderer: &mut oversprite_renderer::Renderer,
    sprite: &oversprite_core::Sprite,
    frame: usize,
    onion_skin: Option<&oversprite_renderer::OnionSkinConfig>,
) -> CoreResult<()> {
    renderer.render_sprite(
        sprite,
        frame,
        &sprite.layer_data,
        &sprite.cel_data,
        &sprite.tilesets,
        &sprite.global_palette,
        onion_skin,
    )
}

/// Combine the rendered PNG frames into an animated GIF.
///
/// The `loop_anim` flag selects whether the animation repeats forever.
fn create_animation(output_dir: &Path, format: &ImageFormat, fps: u32, loop_anim: bool) -> CoreResult<()> {
    let mut entries: Vec<_> = fs::read_dir(output_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("png"))
        })
        .collect();
    entries.sort();

    let frames: Vec<image::RgbaImage> = entries
        .iter()
        .map(|path| {
            image::open(path)
                .map_err(|e| CoreError::Internal(format!("Failed to read {}: {}", path.display(), e)))
                .map(|img| img.to_rgba8())
        })
        .collect::<Result<Vec<_>, _>>()?;

    if frames.is_empty() {
        return Err(CoreError::Internal("No frames found to build an animation".into()));
    }

    let gif_path = output_dir.join("animation.gif");
    let file = fs::File::create(&gif_path)?;
    let mut encoder = image::codecs::gif::GifEncoder::new(file);
    encoder
        .set_repeat(if loop_anim {
            image::codecs::gif::Repeat::Infinite
        } else {
            image::codecs::gif::Repeat::Finite(1)
        })
        .map_err(|e| CoreError::Internal(e.to_string()))?;

    let delay = image::Delay::from_numer_denom_ms(100 / fps.max(1), 1);
    for frame in &frames {
        let animation_frame = image::Frame::from_parts(frame.clone(), 0, 0, delay);
        encoder
            .encode_frame(animation_frame)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
    }

    println!(
        "Created {:?} animation at {} fps ({} frames, loop: {}) -> {}",
        format,
        fps,
        frames.len(),
        loop_anim,
        gif_path.display()
    );
    Ok(())
}

fn info(file: PathBuf, format: OutputFormat, show_layers: bool, show_frames: bool, show_palette: bool) -> CoreResult<()> {
    let sprite = read_sprite(&file)?;

    match format {
        OutputFormat::Text => {
            println!("File: {}", file.display());
            println!("  Dimensions: {}x{}", sprite.metadata.dimensions.w, sprite.metadata.dimensions.h);
            println!("  Color mode: {:?}", sprite.metadata.color_mode);
            println!("  Frames: {}", sprite.frames.order.len());
            println!("  Layers: {}", sprite.layers.order.len());
            println!("  Tags: {}", sprite.tags.len());
            println!("  Slices: {}", sprite.slices.len());
            println!("  Tilesets: {}", sprite.tilesets.len());
            println!("  Palette colors: {}", sprite.global_palette.colors.len());

            if show_layers {
                println!("\nLayers:");
                for (i, uuid) in sprite.layers.order.iter().enumerate() {
                    if let Some(layer) = sprite.layer_data.get(uuid) {
                        println!("  [{}] {} ({:?}) - visible: {}, opacity: {}, blend: {:?}",
                            i, layer.name, layer.layer_type, layer.visible, layer.opacity, layer.blend_mode);
                    }
                }
            }

            if show_frames {
                println!("\nFrames:");
                for (i, uuid) in sprite.frames.order.iter().enumerate() {
                    if let Some(frame) = sprite.frame_data.get(uuid) {
                        println!("  [{}] duration: {}ms, cels: {}", i, frame.duration_ms, frame.cels.len());
                    }
                }
            }

            if show_palette {
                println!("\nPalette: {} colors", sprite.global_palette.colors.len());
            }
        }
        OutputFormat::Json => {
            let info = serde_json::json!({
                "file": file.to_string_lossy(),
                "dimensions": [sprite.metadata.dimensions.w, sprite.metadata.dimensions.h],
                "color_mode": format!("{:?}", sprite.metadata.color_mode),
                "frames": sprite.frames.order.len(),
                "layers": sprite.layers.order.len(),
                "tags": sprite.tags.len(),
                "slices": sprite.slices.len(),
                "tilesets": sprite.tilesets.len(),
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&sprite.metadata)
                .map_err(|e| CoreError::Internal(e.to_string()))?);
        }
    }
    Ok(())
}

fn new(output: PathBuf, width: u32, height: u32, color_mode: ColorModeArg, frames: u32) -> CoreResult<()> {
    println!("Creating new sprite: {}x{} {:?} {} frames", width, height, color_mode, frames);
    let mut sprite = Sprite::new_with_size(width, height, color_mode.into());

    for _ in 1..frames {
        sprite.add_frame(sprite.metadata.default_frame_duration_ms);
    }

    write_sprite(&sprite, &output)?;
    println!("Created {:?}", output);
    Ok(())
}

fn extract(input: PathBuf, output_dir: PathBuf, extract_type: ExtractType, frames_arg: Option<String>) -> CoreResult<()> {
    println!("Extracting {:?} from {:?} -> {:?}", extract_type, input, output_dir);
    let sprite = read_sprite(&input)?;
    fs::create_dir_all(&output_dir)?;

    // `frames_arg` selects a subset of frames; "all" (or None) keeps every frame.
    let selected_frames: Vec<usize> = match frames_arg.as_deref() {
        None => (0..sprite.frames.order.len()).collect(),
        Some(spec) => {
            let mut frames = Vec::new();
            for part in spec.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let index: usize = part.parse().map_err(|_| {
                    CoreError::Internal(format!("Invalid frame index '{}'", part))
                })?;
                if index >= sprite.frames.order.len() {
                    return Err(CoreError::Internal(format!(
                        "Frame index {} out of range (max {})",
                        index,
                        sprite.frames.order.len() - 1
                    )));
                }
                frames.push(index);
            }
            frames
        }
    };

    let want = |kind: ExtractType| matches!(extract_type, ExtractType::All) || extract_type == kind;

    if want(ExtractType::Layers) {
        let layers_dir = output_dir.join("layers");
        fs::create_dir_all(&layers_dir)?;
        for (index, layer_uuid) in sprite.layers.order.iter().enumerate() {
            let Some(layer) = sprite.layer_data.get(layer_uuid) else {
                continue;
            };
            let path = layers_dir.join(format!("layer_{:03}_{}.png", index, sanitize(&layer.name)));
            println!("  layer {} -> {}", layer.name, path.display());
        }
    }

    if want(ExtractType::Frames) {
        let frames_dir = output_dir.join("frames");
        fs::create_dir_all(&frames_dir)?;

        let mut renderer = oversprite_renderer::Renderer::new(
            Size::new(sprite.metadata.dimensions.w, sprite.metadata.dimensions.h),
            1.0,
        )?;

        for index in selected_frames {
            render_frame(&mut renderer, &sprite, index)?;
            let path = frames_dir.join(format!("frame_{:04}.png", index));
            renderer.save_png(&path)?;
            println!("  frame {} -> {}", index, path.display());
        }
    }

    if want(ExtractType::Palettes) {
        let palettes_dir = output_dir.join("palettes");
        fs::create_dir_all(&palettes_dir)?;
        let path = palettes_dir.join("global.gpl");
        std::fs::write(&path, serialize_gpl(&sprite.global_palette))?;
        println!("  global palette -> {}", path.display());
    }

    if want(ExtractType::Tilesets) {
        let tilesets_dir = output_dir.join("tilesets");
        fs::create_dir_all(&tilesets_dir)?;
        for (index, tileset) in sprite.tilesets.values().enumerate() {
            let path = tilesets_dir.join(format!("tileset_{:03}.json", index));
            let json = serde_json::to_string_pretty(tileset)
                .map_err(|e| CoreError::Internal(e.to_string()))?;
            std::fs::write(&path, json)?;
            println!("  tileset {} -> {}", index, path.display());
        }
    }

    if want(ExtractType::Tags) {
        let path = output_dir.join("tags.json");
        let json = serde_json::to_string_pretty(&sprite.tags)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        std::fs::write(&path, json)?;
        println!("  tags -> {}", path.display());
    }

    if want(ExtractType::Slices) {
        let path = output_dir.join("slices.json");
        let json = serde_json::to_string_pretty(&sprite.slices)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        std::fs::write(&path, json)?;
        println!("  slices -> {}", path.display());
    }

    if want(ExtractType::UserData) {
        let path = output_dir.join("user-data.json");
        let json = serde_json::to_string_pretty(&sprite.user_data)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        std::fs::write(&path, json)?;
        println!("  user data -> {}", path.display());
    }

    println!("Extraction complete!");
    Ok(())
}

/// Replace characters that are unsafe to use in file names.
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if cleaned.is_empty() {
        "unnamed".to_string()
    } else {
        cleaned
    }
}

/// Serialize a palette in the GIMP `.gpl` text format.
fn serialize_gpl(palette: &oversprite_core::Palette) -> String {
    let mut out = String::from("GIMP Palette\nName: OverSprite\nColumns: 16\n#\n");
    for color in &palette.colors {
        out.push_str(&format!("{} {} {}\n", color.r, color.g, color.b));
    }
    out
}

fn thumbnail(input: PathBuf, output_dir: PathBuf, sizes: String) -> CoreResult<()> {
    println!("Generating thumbnails for {:?} -> {:?}", input, output_dir);
    let sprite = read_sprite(&input)?;
    fs::create_dir_all(&output_dir)?;

    let mut requested: Vec<u32> = Vec::new();
    for size_str in sizes.split(',') {
        let size_str = size_str.trim();
        if size_str.is_empty() {
            continue;
        }
        let size: u32 = size_str.parse().map_err(|_| {
            CoreError::Internal(format!("Invalid thumbnail size '{}'", size_str))
        })?;
        if size == 0 {
            return Err(CoreError::Internal("Thumbnail size must be greater than 0".into()));
        }
        requested.push(size);
    }

    if requested.is_empty() {
        return Err(CoreError::Internal("No thumbnail sizes requested".into()));
    }

    let width = sprite.metadata.dimensions.w;
    let height = sprite.metadata.dimensions.h;

    for size in requested {
        // Scale the sprite so its longest side matches the requested size.
        let scale = size as f32 / width.max(height) as f32;

        let mut renderer = oversprite_renderer::Renderer::new(Size::new(width, height), scale)?;
        render_frame(&mut renderer, &sprite, 0)?;

        let path = output_dir.join(format!("thumbnail_{}.png", size));
        renderer.save_png(&path)?;
        println!("  {}x{} -> {}", size, size, path.display());
    }

    println!("Thumbnail generation complete!");
    Ok(())
}

fn run_script(script: PathBuf, input: Option<PathBuf>, output: Option<PathBuf>, args: Vec<String>) -> CoreResult<()> {
    println!("Running script {:?}", script);
    let mut context = oversprite_scripting::ScriptContext::new()?;

    if let Some(input_path) = input {
        let sprite = read_sprite(&input_path)?;
        context = oversprite_scripting::ScriptContext::with_sprite(sprite)?;
    }

    context.run_file(&script)
        .map_err(|e| CoreError::Internal(e.to_string()))?;

    if let Some(output_path) = output {
        if let Some(sprite) = context.sprite() {
            write_sprite(&sprite.lock().unwrap(), &output_path)?;
        }
    }

    let _ = args;
    println!("Script execution complete!");
    Ok(())
}

fn extension_command(command: ExtensionCommand) -> CoreResult<()> {
    match command {
        ExtensionCommand::Install { dir } => {
            println!("Installing extension from {:?}", dir);
            // Load and install extension
        }
        ExtensionCommand::List => {
            println!("Installed extensions:");
            // List extensions
        }
        ExtensionCommand::Remove { name } => {
            println!("Removing extension: {}", name);
        }
        ExtensionCommand::Create { name, dir } => {
            println!("Creating extension template: {} at {:?}", name, dir);
            fs::create_dir_all(&dir)?;
            // Create package.json and main.lua template
        }
    }
    Ok(())
}