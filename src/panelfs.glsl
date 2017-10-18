#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform vec4 in_color;
uniform usampler2D tex;
uniform usampler2D characters;
uniform vec4 bounds;

void main() {
	if (v_tex_coords.x < bounds.x || v_tex_coords.y < bounds.y ||
		v_tex_coords.x >= bounds.z || v_tex_coords.y >= bounds.a) {
		color = vec4(0.0,0.0,0.0,1.0);
		return;
	}
    float cx, cy;

    float vx = modf(v_tex_coords.x,cx);
    float vy = modf(v_tex_coords.y,cy);

    float ascii = texelFetch(characters,ivec2(int(cx),int(cy)),0).r;
    float c0 = ascii / 128.0; // 'a'

    vec2 voff = vec2(c0 + (vx/128.0),vy);
    float rv = texture(tex,voff).r;
    if (rv > 0.0) {
        color = vec4(in_color.rgb, 1.0);
    } else {
        color = vec4(0.0,0.0,0.0,in_color.a);
    }
}
