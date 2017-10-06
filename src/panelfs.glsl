#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform vec4 in_color;
uniform usampler2D tex;

void main() {
	if (v_tex_coords.x < 0 || v_tex_coords.y < 0) {
		color = vec4(0.0,0.0,1.0,1.0);
		return;
	}
    float cx, cy;
    float c0 = 97.0 / 128.0; // 'a'

    float vx = modf(v_tex_coords.x,cx);
    float vy = modf(v_tex_coords.y,cy);

    vec2 voff = vec2(c0 + (vx/128.0),vy);
    //vec2 voff = vec2( (v_tex_coords.x/(1024.0*25.0)) + (97.0 / 128.0), v_tex_coords.y/20.0);
    float rv = texture(tex,voff).r;
    if (rv > 0.0) {
        color = vec4(in_color.rgb, 1.0);
    } else {
        color = vec4(0.0,0.1,0.0,in_color.a);
    }
}
