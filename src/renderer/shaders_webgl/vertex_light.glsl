#version 300 es
precision mediump float;

layout(location = 0) in vec2 vertex;

layout(location = 1) in vec2 position;
layout(location = 2) in vec2 size;
layout(location = 3) in vec2 uv_center;
layout(location = 4) in vec2 uv_size;
layout(location = 5) in vec3 color;

out vec2 tex_coords;
out vec3 light_color;

void main() {
    gl_Position = vec4(position + vertex * size, 0.5, 1.0);
    tex_coords = uv_center + vec2(vertex.x, -vertex.y) * uv_size;
    light_color = color;
}
