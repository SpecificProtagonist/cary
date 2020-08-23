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
    if(color.a < 0.5) {
        discard;
    } else if(color.rgb != vec3(0,0,0) && transition_distance > distance(gl_FragCoord.xy/window_size, transition_center)) {
        // Only replace white and exact other color
        // so keycap image doesn't change
        // Usually one shouldn't compare floats for equality,
        // but these were read straight from the texture
        vec4 white = vec4(1, 1, 1, 1);
        vec4 black = vec4(0, 0, 0, 1);
        // we can't just create a vec, colors don't match - maybe because of gamme?
        vec4 cyan = texture(sampler2D(tex, tex_sampler), $cyan_coords);
        vec4 red = texture(sampler2D(tex, tex_sampler), $red_coords);
        if(transition_victory == 1.0) {
            out_color = color == white ? cyan
                      : color == red   ? cyan
                                       : color;
        } else {
            out_color = color == white ? red
                      : color == cyan  ? red
                      : color == red   ? black
                                       : color;
        }
    } else {
        out_color = color;
    }
}
