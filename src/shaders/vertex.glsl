#version 450

layout(location = 0) in vec2 vertex;

layout(location = 1) in vec2 position;
layout(location = 2) in vec2 size;
layout(location = 3) in vec2 uv_center;
layout(location = 4) in float layer;

layout(location = 0) out vec2 tex_coords_frag;

void main() {
    gl_Position = vec4(position + vertex * size, layer, 1.0);
    tex_coords_frag = uv_center + vertex * size;
}
