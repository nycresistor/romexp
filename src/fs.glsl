#version 130

out vec4 out_color;

uniform uvec4 win;
uniform uint bitstride;
uniform uint colstride;
uniform uint datalen;
uniform float zoom;

uniform uvec2 selection;
uniform uint texwidth;

uniform usampler2D romtex;

void main() {
     // corrected position in window
     vec2 fc = vec2( gl_FragCoord[0] - 0.5, float(win[3] - 1u) - (gl_FragCoord[1] - 0.5) );
     // absolute coordinates in bitmap
     uvec2 ac = uvec2( win.x + uint(fc.x * zoom), win.y + uint(fc.y * zoom) );
     uint col = ac.x / bitstride;
     uint row = ac.y;
     
     uint bitidx = col * colstride + row * bitstride + ac.x % bitstride;
     
     uint tex_off = bitidx / 8u;
     uint tex_bit_off = bitidx % 8u;

     if (tex_off >= datalen) {
     	out_color = vec4(0.0,0.0,0.4,1.0);
     	return;
	}	
    uint tex_off_x = tex_off % texwidth;
    uint tex_off_y = tex_off / texwidth;
    uint rv = (texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r >> (7u-tex_bit_off)) & 1u;
    vec4 c = vec4(float(rv),float(rv),float(rv), 1.0);
    if (bitidx >= selection[0] && bitidx < selection[1]) {
        c.b = 0.0; c.g = 0.0;
    }
     out_color = c; 
}
