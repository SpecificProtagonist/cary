#version 450

layout(location = 0) in vec2 tex_coords;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler tex_sampler;
layout(set = 1, binding = 0) uniform texture2D lightmap;
layout(set = 1, binding = 1) uniform sampler lightmap_sampler;
layout(set = 1, binding = 2) uniform WindowSize {
    vec2 window_size;
};
layout(set = 2, binding = 0) uniform Transition {
    vec2 transition_center;
    float transition_distance;
    float aspect_ratio;
    float transition_victory; // actually bool, maybe change
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
    vec4 color = texture(sampler2D(tex, tex_sampler), tex_coords);
    vec4 transition_color = transition_victory > 0.5 ? vec4(0, 1, 1, 1) : vec4(1, 0.5, 0.5, 1);
    if(color.a < 0.5) {
        discard;
    } else if(color.rgb != vec3(0,0,0) && transition_distance > distance(gl_FragCoord.xy/window_size, transition_center)) {
        out_color = transition_color;
    } else {
        out_color = color;
    }
}
