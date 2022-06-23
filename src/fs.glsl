#version 130

in vec2 v_tex_coords;
out vec4 color;

uniform uvec4 win;
uniform uint bitstride;
uniform uint colstride;
uniform uint spacing;
uniform uint datalen;
uniform uint dataoff;
uniform uint bpp;
uniform bool swap_endian;
uniform bool bitmode; // true to display data as bits, false as bytes. maybe a width?

uniform uvec2 selection;
uniform uint texwidth;

uniform usampler2D romtex;
uniform usampler2D annotex;

/// zoom factor (2.0 = 2x)
uniform float zoom;
/// offset of upper left hand corner in pixels at the current zoom level
uniform vec2 ul_offset;

void main() {
    vec2 fc = vec2( v_tex_coords[0] * float(win[2]),
                    (1.0 - v_tex_coords[1]) * float(win[3]));
    fc = (fc + ul_offset) / zoom;
    // fc is now absolute coordinates in bitmap
    if (fc[0] < 0 || fc[1] < 0) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }
    uvec2 ac = uvec2( win.x + uint(fc.x), win.y + uint(fc.y) );
    uint col = ac.x / (bitstride + spacing);
    uint row = ac.y;
     
    if (ac.x % (bitstride+spacing) >= bitstride) { 
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }

    uint bitidx = col * colstride + row * bitstride + ac.x % bitstride;
    if (bpp == 8u) {
       bitidx = col * colstride + row * bitstride / 8u + ac.x % bitstride /8u;
    }
    uint tex_off = (bitidx / 8u) + dataoff;
    uint tex_bit_off = bitidx % 8u;
    if (bpp == 8u) {
        tex_off = bitidx + dataoff;
        tex_bit_off = 0u;
    }    

    if (swap_endian == true && bitstride > 8u) {
        uint bytes = bitstride / 8u;
        tex_off = ((tex_off / bytes) * bytes) + (bytes - (1u+(tex_off % bytes)));
    } 

    if (row >= (colstride/bitstride) || tex_off >= datalen) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }       
    uint tex_off_x = tex_off % texwidth;
    uint tex_off_y = tex_off / texwidth;
    float rv = float((texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r >> (7u-tex_bit_off)) & 1u);
    if (bpp == 8u) {
       rv = float((texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r) / 255.0);
    }

    // get annotation
    uint anno = texelFetch(annotex, ivec2(int(tex_off_x),int(tex_off_y)),0).r;
    vec4 c = vec4(rv,rv+float(anno),rv, 1.0);
    if (anno != 0u) { 
        c.r = 0.0; 
    }
    if (selection[0] != selection[1] && bitidx >= selection[0] && bitidx <= selection[1]+7u) {
        c.b = 0.0; c.g = 0.0;
    }
    color = c;
}
