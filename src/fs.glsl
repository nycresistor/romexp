#version 130

in vec2 v_tex_coords;
out vec4 color;

uniform uvec4 win;
uniform uint bitstride;
uniform uint colstride;
uniform uint datalen;

uniform uvec2 selection;
uniform uint texwidth;

uniform usampler2D romtex;

uniform float zoom;
uniform vec2 ul_offset;

void main() {
     vec2 fc = vec2( v_tex_coords[0] * float(win[2]),
                     (1.0 - v_tex_coords[1]) * float(win[3]));
     fc = fc / zoom + ul_offset;
     // absolute coordinates in bitmap
     if (fc[0] < 0 || fc[1] < 0) {
     	color = vec4(0.0,0.0,0.4,1.0);
	return;
     }
     uvec2 ac = uvec2( win.x + uint(fc.x), win.y + uint(fc.y) );
     uint col = ac.x / bitstride;
     uint row = ac.y;
     
     uint bitidx = col * colstride + row * bitstride + ac.x % bitstride;
     
     uint tex_off = bitidx / 8u;
     uint tex_bit_off = bitidx % 8u;
     
     if (row > (colstride/bitstride) || tex_off >= datalen) {
     	color = vec4(0.0,0.0,0.4,1.0);
     	return;
	}	
    uint tex_off_x = tex_off % texwidth;
    uint tex_off_y = tex_off / texwidth;
    uint rv = (texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r >> (7u-tex_bit_off)) & 1u;
    vec4 c = vec4(float(rv),float(rv),float(rv), 1.0);
    if (bitidx >= selection[0] && bitidx < selection[1]) {
        c.b = 0.0; c.g = 0.0;
    }
     color = c; 
}
