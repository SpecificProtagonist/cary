#version 450

layout(location = 0) in vec2 vertex;

layout(location = 1) in vec2 position;
layout(location = 2) in vec2 size;
layout(location = 3) in vec2 uv_center;
layout(location = 4) in vec2 uv_size;
layout(location = 5) in float layer;
layout(location = 6) in float rotation;

layout(location = 0) out vec2 tex_coords_frag;

vec2 rotate(vec2 vert) {
    return rotation == 1 ? vec2(vert.y, -vert.x)
         : rotation == 2 ? vec2(-vert.x, -vert.y)
         : rotation == 3 ? vec2(-vert.y, vert.x)
                         : vert;
}

void main() {
    gl_Position = vec4(position + rotate(vertex * size), layer, 1.0);
    tex_coords_frag = uv_center + vec2(vertex.x, -vertex.y) * uv_size;
}
