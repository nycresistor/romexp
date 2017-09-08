#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform vec4 in_color;
uniform usampler2D tex;

void main() {
    float rv = texture(tex,v_tex_coords).r;
    if (rv > 0.0) {
        color = vec4(in_color.rgb, 1.0);
    } else {
        color = vec4(0.0,0.0,0.0,in_color.a);
    }
}
