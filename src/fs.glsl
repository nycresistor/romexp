#version 130

// The texture coordinates are mapped over the window from (0.0,0.0) to (1.0,1.0)
in vec2 v_tex_coords;
out vec4 color;

uniform uvec4 win;        // bounds of physical window
uniform uint bitstride;   // width of column in bits
uniform uint colstride;   // number of bits between successive columns
uniform uint spacing;     // spacing between columns, in pixels
uniform uint datalen;     // total length of data, in bytes
uniform uint dataoff;     // offset before start of data to display, in bytes
uniform uint bpp;         // bits per pixel (1 for bitmap, 8 for bytemap, etc)
uniform bool swap_endian; // swap byte-endianness when true

uniform uvec2 selection;  // start and end of selection, in bytes
uniform uint texwidth;    // width of data texture

uniform usampler2D romtex;  // data texture
uniform usampler2D annotex; // annotation texture

/// zoom factor (2.0 = 2x)
uniform float zoom;
/// offset of upper left hand corner in pixels at the current zoom level
uniform vec2 ul_offset;

void main() {
    // Convert from texture coordinates to screen coordinates.
    vec2 fc = vec2( v_tex_coords[0] * float(win[2]),
                    (1.0 - v_tex_coords[1]) * float(win[3]));
    // Scale the coordinates by the zoom factor and adjust for the panning location.
    fc = (fc + ul_offset) / zoom;
    // Handle points above or to the left of the bitmap display.
    if (fc[0] < 0 || fc[1] < 0) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }
    // Adjust for the top left corner of the window (normally 0,0)
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
