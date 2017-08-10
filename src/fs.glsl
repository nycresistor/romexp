#version 130

out vec4 out_color;
uniform uint ww;
uniform uint wh;
uniform uint stride;
uniform uint romh;

uniform usampler2D romtex;

void main() {
    uint texw = 16384u;
     uint x = uint(gl_FragCoord[0] - 0.5);
     uint y = (wh - 1u) - uint(gl_FragCoord[1] - 0.5);
     uint col = x / stride;
     uint bitidx = ((y  + (col*wh)) * stride) + (x % stride);
     
     uint tex_off = bitidx / 8u;
     uint tex_bit_off = bitidx % 8u;

     if (tex_off >= {}u) {
     	out_color = vec4(0.0,0.0,0.4,1.0);
     	return;
	}	
    uint tex_off_x = tex_off % texw;
    uint tex_off_y = tex_off / texw;
    vec2 coord = vec2( (float(tex_off_x)+0.5) / float(texw), (float(tex_off_y)+0.5) / float(romh) );
     uint rv = (texture(romtex, coord).r >> (7u-tex_bit_off)) & 1u;
    uint rv2 = texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r;
     out_color = vec4(float(rv),float(rv),float(rv), 1.0);
}
