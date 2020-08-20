#version 300 es
precision mediump float;

in vec2 tex_coords;
in vec3 light_color;

out vec4 outColor;

uniform sampler2D tex;

void main() {
    outColor = texture(tex, tex_coords) * vec4(light_color, 1);
}
