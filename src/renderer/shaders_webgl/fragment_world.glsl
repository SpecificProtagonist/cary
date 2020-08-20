#version 300 es
precision mediump float;

in vec2 tex_coords;

out vec4 out_color;

uniform sampler2D tex;
uniform sampler2D lightmap;
uniform vec2 window_size;

vec4 light_color() {
    vec4 light = texture(lightmap, gl_FragCoord.xy/window_size);
    float avg = (light.r + light.g + light.b) / 3.0;
    float adjusted =
        step(0.03, avg) * 0.10
      + step(0.17, avg) * 0.15
      + step(0.28, avg) * 0.25
      + step(0.55, avg) * 0.25
      + step(0.85, avg) * 0.25;
    return light * adjusted/avg;
}

void main() {
    vec4 color = texture(tex, tex_coords) * light_color();
    if(color.a < 0.5) {
        discard;
    } else {
        out_color = color;
    }
}