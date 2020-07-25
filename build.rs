use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use shaderc::{Compiler, ShaderKind};
use texture_packer::{
    TexturePacker,
    TexturePackerConfig,
    importer::ImageImporter,
    exporter::ImageExporter
};
use image;



/// Builds the resources to be embedded. 
/// There won't be any error handling here because it will never be executed by the user
fn main() {
    compile_shaders();
    texture_atlas();
}

fn compile_shaders() {
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("shaders");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut compiler = Compiler::new().unwrap();

    // Vertex
    println!("cargo:rerun-if-changed=src/shaders/vertex.glsl");
    let vertex_spirv = compiler.compile_into_spirv(
        include_str!("src/shaders/vertex.glsl"), ShaderKind::Vertex, "vertex.glsl", "main", None
    ).unwrap();
    File::create(out_dir.join("vertex.spv")).unwrap()
        .write_all(vertex_spirv.as_binary_u8()).unwrap();

    // Fragment
    println!("cargo:rerun-if-changed=src/shaders/fragment.glsl");
    let fragment_spirv = compiler.compile_into_spirv(
        include_str!("src/shaders/fragment.glsl"), ShaderKind::Fragment, "fragment.glsl", "main", None
    ).unwrap();
    File::create(out_dir.join("fragment.spv")).unwrap()
        .write_all(fragment_spirv.as_binary_u8()).unwrap();
}

fn texture_atlas() {
    // Cargo doesn't support rerun on directory content change, so we have to manually trigger this
    println!("cargo:rerun-if-changed=textures/touch-to-rebuild-texture-atlas");

    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_owned();
    
    let mut packer = TexturePacker::new_skyline(TexturePackerConfig {
        max_width: 2048, // Why can't the texture packer choose these on its own? Libgdx was better.
        max_height: 4096,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 2,
        texture_padding: 2,
        trim: false
    });

    for entry in walkdir::WalkDir::new("textures") {
        let entry = entry.unwrap();
        if entry.path().extension().map_or(false, |ext|ext=="png") {
            let id = entry.path().to_string_lossy();
            let id = id["textures/".len() .. id.len() - ".png".len()]
                .replace(std::path::MAIN_SEPARATOR, "_")
                .replace('-', "_")
                .to_uppercase();
            let texture = ImageImporter::import_from_file(&entry.path()).unwrap();
            packer.pack_own(id, texture).unwrap();
        }
    }

    // Export texture
    ImageExporter::export(&packer).unwrap().write_to(
        &mut File::create(out_dir.join("textures.png")).unwrap(), 
        image::ImageFormat::Png)
    .unwrap();

    // Export uv-coordinates in a form where it can be included in a rust file
    let mut out_file = File::create(out_dir.join("uv-coords")).unwrap();
    for (id, sprite) in packer.get_frames() {
        writeln!(out_file, "pub const {}: Sprite = Sprite {{center_x: {}f32, center_y: {}f32, width: {}f32, height: {}f32}};", 
            id, sprite.frame.x, sprite.frame.y, sprite.frame.w, sprite.frame.h).unwrap()
    }
}