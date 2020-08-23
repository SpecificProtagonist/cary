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
        // Only replace white and exact other color
        // so keycap image doesn't change
        // Usually one shouldn't compare floats for equality,
        // but these were read straight from the texture
        vec4 white = vec4(1, 1, 1, 1);
        vec4 black = vec4(0, 0, 0, 1);
        // we can't just create a vec, colors don't match - maybe because of gamme?
        vec4 cyan = texture(tex, $cyan_coords);
        vec4 red = texture(tex, $red_coords);
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