use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use shaderc::{Compiler, ShaderKind};



fn main() {
    compile_shaders();
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