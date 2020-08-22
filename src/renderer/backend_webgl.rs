use winit::platform::web::WindowExtWebSys;
use wasm_bindgen::JsCast;
use web_sys::{
    WebGlProgram, 
    WebGl2RenderingContext, 
    HtmlCanvasElement, 
    WebGlShader, 
    WebGlVertexArrayObject, 
    WebGlBuffer, 
    WebGlFramebuffer,
    WebGlTexture
};
use image::GenericImageView;
use crate::textures::{self, TexCoords, TexAnchor};
use crate::Vec2;
use super::{Rgb, Layer};

const ATTRIB_VERTEX: u32 = 0;
const ATTRIB_POSITION: u32 = 1;
const ATTRIB_SIZE: u32 = 2;
const ATTRIB_UV_CENTER: u32 = 3;
const ATTRIB_UV_SIZE: u32 = 4;
const ATTRIB_LAYER: u32 = 5;
const ATTRIB_COLOR: u32 = 5;

#[repr(C)]
#[derive(Copy, Clone)]
struct SpriteInstance {
    vertex: Vec2,
    pos: Vec2,
    size: Vec2,
    uv_center: Vec2,
    uv_size: Vec2,
    layer: f32
} 
unsafe impl bytemuck::Pod for SpriteInstance {}
unsafe impl bytemuck::Zeroable for SpriteInstance {}

const vertices: &[f32; 12] = &[
    -0.5_f32,  0.5_f32,
    -0.5_f32, -0.5_f32,
     0.5_f32,  0.5_f32,
    -0.5_f32, -0.5_f32,
     0.5_f32, -0.5_f32,
     0.5_f32,  0.5_f32
];

pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: WebGl2RenderingContext,
    program_world: WebGlProgram,
    vao_world: WebGlVertexArrayObject,
    buffer: WebGlBuffer,
    sprite_instances: Vec<SpriteInstance>,
}

impl Renderer {
    pub fn create(window: &winit::window::Window) -> Self {
        let canvas = window.canvas();
        
        let context: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();

        context.enable(WebGl2RenderingContext::BLEND);
        context.enable(WebGl2RenderingContext::DEPTH_TEST);
        context.depth_func(WebGl2RenderingContext::LEQUAL);
        context.clear_depth(1.0);
        context.clear_color(0.0, 0.0, 0.0, 1.0);

        let program_world = link_program(
            &context, 
            &compile_shader(
                &context,
                WebGl2RenderingContext::VERTEX_SHADER,
                include_str!("shaders_webgl/vertex_world.glsl")
            ), 
            &compile_shader(
                &context,
                WebGl2RenderingContext::FRAGMENT_SHADER,
                include_str!("shaders_webgl/fragment_world.glsl")
            )
        );

        

        /*** TEXTURE ***/
        let texture_image = image::load_from_memory(include_bytes!(concat!(env!("OUT_DIR"), "/textures.png"))).unwrap();
        let texture = context.create_texture().expect("Failed to create texture");
        context.active_texture(WebGl2RenderingContext::TEXTURE0);
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        // We are never zoomed out, so don't generate mipmaps
        context.tex_storage_2d(WebGl2RenderingContext::TEXTURE_2D, 1, WebGl2RenderingContext::RGBA8, 
            texture_image.dimensions().0 as i32, texture_image.dimensions().1 as i32);

        let bytes: &[u8] = &texture_image.as_rgba8().unwrap();
        context.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            0, 0,
            texture_image.dimensions().0 as i32, texture_image.dimensions().1 as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(bytes),
        ).expect("Failed to upload texture data");

        context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::NEAREST as i32);
        context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR as i32);
        context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);

        context.use_program(Some(&program_world));
        let uniform_world_tex = context.get_uniform_location(&program_world, "tex").unwrap();
        context.uniform1i(Some(&uniform_world_tex), 0);


        let buffer = context.create_buffer().unwrap();

        /*** WORLD VERTEX BUFFER ***/
        let vao_world = context.create_vertex_array().unwrap();
        context.bind_vertex_array(Some(&vao_world));
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        let instance_size = std::mem::size_of::<SpriteInstance>() as i32;
        context.vertex_attrib_pointer_with_i32(ATTRIB_VERTEX,    2, WebGl2RenderingContext::FLOAT, false, instance_size, 4 * 0);
        context.vertex_attrib_pointer_with_i32(ATTRIB_POSITION,  2, WebGl2RenderingContext::FLOAT, false, instance_size, 4 * 2);
        context.vertex_attrib_pointer_with_i32(ATTRIB_SIZE,      2, WebGl2RenderingContext::FLOAT, false, instance_size, 4 * 4);
        context.vertex_attrib_pointer_with_i32(ATTRIB_UV_CENTER, 2, WebGl2RenderingContext::FLOAT, false, instance_size, 4 * 6);
        context.vertex_attrib_pointer_with_i32(ATTRIB_UV_SIZE,   2, WebGl2RenderingContext::FLOAT, false, instance_size, 4 * 8);
        context.vertex_attrib_pointer_with_i32(ATTRIB_LAYER,     1, WebGl2RenderingContext::FLOAT, false, instance_size, 4 * 10);
        context.enable_vertex_attrib_array(ATTRIB_VERTEX);
        context.enable_vertex_attrib_array(ATTRIB_POSITION);
        context.enable_vertex_attrib_array(ATTRIB_SIZE);
        context.enable_vertex_attrib_array(ATTRIB_UV_CENTER);
        context.enable_vertex_attrib_array(ATTRIB_UV_SIZE);
        context.enable_vertex_attrib_array(ATTRIB_LAYER);


        // Remove the "Loading..." text
        web_sys::window().unwrap().document().unwrap().get_element_by_id("loading").unwrap().remove();

        Renderer {
            canvas,
            context,
            program_world,
            vao_world,
            buffer,
            sprite_instances: Vec::new(),
        }
    }

    // This doesn't seem to get called by winit?
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.context.viewport(0, 0, width as i32, height as i32);

        // resize depth?

        self.context.use_program(Some(&self.program_world));
        let window_size_uniform = self.context.get_uniform_location(&self.program_world, "window_size").unwrap();
        self.context.uniform2f(Some(&window_size_uniform), width as f32, height as f32);
    }

    pub fn render(&mut self) {
        // Check if we need to resize
        let window_width = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap() as u32;
        let window_height = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap() as u32;
        if (window_width != self.canvas.width()) | (window_height != self.canvas.height()) {
            self.resize(window_width, window_height);
        }

        /*** WORLD ***/
        self.context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        self.context.bind_vertex_array(Some(&self.vao_world));
        self.context.use_program(Some(&self.program_world));
        self.context.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);
        // Upload instance data
        self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));
        unsafe {
            self.context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &js_sys::Uint8Array::view(&bytemuck::cast_slice(&self.sprite_instances)),
                WebGl2RenderingContext::STREAM_DRAW,
            );
        }
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        self.context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, self.sprite_instances.len() as i32);
        
        self.sprite_instances.clear();

        self.context.flush();
    }

    pub fn draw(&mut self, camera: &crate::Camera, pos: Vec2, anchor: TexAnchor, tex: &TexCoords, layer: Layer, mirror: bool) {
        let size_real = tex.size / textures::PIXELS_PER_TILE;
        let pos = Vec2(pos.0, pos.1 + match anchor {
            TexAnchor::Top    => -size_real.1/2.0,
            TexAnchor::Center => 0.0,
            TexAnchor::Bottom => size_real.1/2.0
        });
        let resize = Vec2(self.canvas.height() as f32 / self.canvas.width() as f32, 1.0) / camera.size;
        let screen_pos = (pos-camera.pos) * resize;
        let screen_size = size_real * resize;
        if (screen_pos.0 + screen_size.0/2.0 > -1.0) &
           (screen_pos.1 + screen_size.1/2.0 > -1.0) &
           (screen_pos.0 - screen_size.0/2.0 <  1.0) &
           (screen_pos.1 - screen_size.1/2.0 <  1.0) 
        {
            for i in 0..6 {
                self.sprite_instances.push(SpriteInstance {
                    vertex: Vec2(vertices[(2*i) as usize], vertices[(2*i+1) as usize]),
                    pos: screen_pos,
                    size: screen_size,
                    uv_center: tex.center * textures::UV_COORDS_FACTOR,
                    uv_size: tex.size * if mirror {Vec2(-1.0, 1.0)} else {Vec2(1.0, 1.0)} * textures::UV_COORDS_FACTOR,
                    layer: layer.into()
                });
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log(s: &str);
}
fn log(s: &str) {
    unsafe { js_log(s) }
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> WebGlShader {
    let shader = context.create_shader(shader_type).expect("Unable to create shader object");

    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        shader
    } else {
        log(&context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")));
        panic!("Failed to compile shader");
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> WebGlProgram {
    let program = context.create_program().expect("Unable to create program object");

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        program
    } else {
        log(&context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")));
        panic!("Failed to link program");
    }
}