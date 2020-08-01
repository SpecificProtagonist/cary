use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;
use shaderc::{Compiler, ShaderKind};
use guillotiere::*;
use image::{DynamicImage, GenericImage, GenericImageView};



/// Builds the resources to be embedded. 
/// There won't be any error handling here because this will never be executed by the user
fn main() {
    compile_shaders();
    texture_atlas();
}

fn compile_shaders() {
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("shaders");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut compiler = Compiler::new().unwrap();

    let mut compile_shader = |name, shader_kind| {
        let src_file = format!("src/shaders/{}.glsl", name);
        println!("cargo:rerun-if-changed={}", src_file);
        let vertex_spirv = compiler.compile_into_spirv(
            &std::fs::read_to_string(src_file).unwrap(), 
            shader_kind, name, "main", None
        ).unwrap();
        File::create(out_dir.join(format!("{}.spv", name))).unwrap()
            .write_all(vertex_spirv.as_binary_u8()).unwrap();
    };

    compile_shader("fragment_light", ShaderKind::Fragment);
    compile_shader("vertex_light", ShaderKind::Vertex);
    compile_shader("fragment_world", ShaderKind::Fragment);
    compile_shader("vertex_world", ShaderKind::Vertex);
}

fn texture_atlas() {
    // TODO: normal maps

    // Cargo doesn't support rerun on directory content change, so we have to manually trigger this
    println!("cargo:rerun-if-changed=textures/touch-to-rebuild-texture-atlas");

    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_owned();
   
    let mut size = Size::new(1024, 1024);
    let mut atlas_alloc = AtlasAllocator::new(size);
    let mut map: HashMap<String, Vec<_>> = HashMap::new();

    for entry in walkdir::WalkDir::new("textures") {
        let entry = entry.unwrap();
        if entry.path().extension().map_or(false, |ext|ext=="png") {
            let id = entry.path().to_string_lossy();
            // Turn path into a valid id
            let mut id = id["textures/".len() .. id.len() - ".png".len()]
                .replace(std::path::MAIN_SEPARATOR, "_")
                .replace('-', "_")
                .to_uppercase();

            let index = if let Some((last_segment_start, _)) = id.rmatch_indices('_').next() {
                if let Ok(index) = id[(last_segment_start+1)..].parse() {
                    id.truncate(last_segment_start);
                    index
                } else { 0 }
            } else { 0 };

            let image = image::open(&entry.path()).unwrap();
            // Why does it use signed integers?
            let image_size = Size::new(image.width() as i32, image.height() as i32);

            let insert = |map: &mut HashMap<String, Vec<_>>, alloc| {
                if let Some(vec) = map.get_mut(&id) {
                    if vec.len() <= index {
                        vec.resize(index+1, None);
                    }
                    vec[index] = Some((alloc, image));
                } else {
                    let mut vec = vec![None; index+1];
                    vec[index] = Some((alloc, image));
                    map.insert(id, vec);
                }
            };

            if let Some(alloc) = atlas_alloc.allocate(image_size) {
                insert(&mut map, alloc);
            } else {
                // Grow the texture atlas dimensions
                if size.height < size.width {
                    size.height *= 2
                } else {
                    size.width *= 2
                }
                let changelist = atlas_alloc.resize_and_rearrange(size);
                if changelist.failures.len() != 0 {
                    panic!();
                }
                for change in changelist.changes {
                    for (_, sprites) in map.iter_mut() {
                        for sprite in sprites {
                            if let Some(sprite) = sprite {
                                // You know how Allocations have id's? Turns out change.old.id is wrong
                                if sprite.0.rectangle == change.old.rectangle {
                                    sprite.0 = change.new;
                                    break;
                                }
                            }
                        }
                    }
                }
                insert(&mut map, atlas_alloc.allocate(image_size).unwrap());
            }
        }
    }

    let mut texture = DynamicImage::new_rgba8(size.width as u32, size.height as u32);

    // Blit sprites
    for sprite in map.values().flat_map(|vec|vec.iter()) {
        if let Some((alloc, image)) = sprite {
            for x in 0..image.width() {
                for y in 0..image.height() {
                    texture.put_pixel(alloc.rectangle.min.x as u32 + x, alloc.rectangle.min.y as u32 + y, image.get_pixel(x, y));
                }
            }
        }
    }

    // Write texture. TODO: check if it can be compressed further
    texture.write_to(
        &mut File::create(out_dir.join("textures.png")).unwrap(), 
        image::ImageFormat::Png
    ).unwrap();

    // Export uv-coordinates in a form where it can be included in a rust file
    let mut out_file = File::create(out_dir.join("uv-coords.rs")).unwrap();
    writeln!(out_file, "pub const UV_COORDS_FACTOR: Vec2 = Vec2({}f32, {}f32);",
        1.0/(size.width as f32), 1.0/(size.height as f32)).unwrap();
    for (id, vec) in &map {
        if vec.len() == 1 {
            let rectangle = vec[0].as_ref().unwrap().0.rectangle;
            writeln!(out_file, "pub const {}: Sprite = Sprite {{center: Vec2({}f32, {}f32), size: Vec2({}f32, {}f32)}};", 
                id, rectangle.center().x, rectangle.center().y, rectangle.width(), rectangle.height()).unwrap()
        } else {
            writeln!(out_file, "pub const {}: &[Sprite] = &[", id).unwrap();
            for sprite in vec {
                if let Some(sprite) = sprite {
                    let rectangle = sprite.0.rectangle;
                    writeln!(out_file, "    Sprite {{center: Vec2({}f32, {}f32), size: Vec2({}f32, {}f32)}},",
                        rectangle.center().x, rectangle.center().y, rectangle.width(), rectangle.height()).unwrap()
                } else {
                    panic!("Missing animation frame for {}", id);
                }
            }
            writeln!(out_file, "];").unwrap();
        }
    }
}