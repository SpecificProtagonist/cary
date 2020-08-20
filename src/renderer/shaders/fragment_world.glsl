#version 450

layout(location = 0) in vec2 tex_coords;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler tex_sampler;
layout(set = 1, binding = 0) uniform texture2D lightmap;
layout(set = 1, binding = 1) uniform sampler lightmap_sampler;
layout(set = 1, binding = 2) uniform Uniforms {
    vec2 window_size;
};


vec4 light_color() {
    vec4 light = texture(sampler2D(lightmap, lightmap_sampler), gl_FragCoord.xy/window_size);
    float avg = (light.r + light.g + light.b) / 3;
    float adjusted =
        step(0.03, avg) * 0.10
      + step(0.17, avg) * 0.15
      + step(0.28, avg) * 0.25
      + step(0.55, avg) * 0.25
      + step(0.85, avg) * 0.25;
    return light * adjusted/avg;
}

void main() {
    vec4 color = texture(sampler2D(tex, tex_sampler), tex_coords) * light_color();
    if(color.a < 0.5) {
        discard;
    } else {
        outColor = color;
    }
}
