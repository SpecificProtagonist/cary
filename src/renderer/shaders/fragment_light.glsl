#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in vec3 light_color;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler tex_sampler;

void main() {
    outColor = texture(sampler2D(tex, tex_sampler), tex_coords) * vec4(light_color, 1);
}
