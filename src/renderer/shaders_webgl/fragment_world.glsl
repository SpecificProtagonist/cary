#version 300 es
precision mediump float;

in vec2 tex_coords;

out vec4 out_color;

uniform sampler2D tex;
uniform sampler2D lightmap;
uniform vec2 window_size;
uniform vec2 transition_center;
uniform float transition_distance;
uniform float transition_victory; // should be a bool in


void main() {
    vec4 color = texture(tex, tex_coords);
    vec4 transition_color = transition_victory == 1.0 ? vec4(0, 1, 1, 1) : vec4(1, 0.5, 0.5, 1);
    if(color.a < 0.5) {
        discard;
    } else if(transition_distance > distance(gl_FragCoord.xy/window_size, transition_center)) {
        out_color = transition_color;
    } else {
        out_color = color;
    }
}