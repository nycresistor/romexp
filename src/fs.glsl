#version 130

// The texture coordinates are mapped over the window from (0.0,0.0) to (1.0,1.0)
in vec2 v_tex_coords;
out vec4 color;

// The data display is organized into a number of columns of a fixed height, displayed side by side with a specified spacing between them.

uniform uvec4 win;        // bounds of physical window

uniform float zoom;       // zoom factor (2.0 = 2x)
uniform vec2 ul_offset;   // offset of upper left hand corner in pixels at the current zoom level

uniform uint colwidth;    // width of a column of data, in elements
uniform uint colspace;    // spacing between adjacent columns, in elements
uniform uint colheight;   // height of a column of data, in elements

uniform uint datalen;     // total length of data, in bytes
uniform uint dataoff;     // offset before start of data to display, in bytes
uniform uint bpp;         // bits per pixel (1 for bitmap, 8 for bytemap, etc)

// Disabling endian swap for now
// uniform bool swap_endian; // swap byte-endianness when true

uniform uvec2 selection;  // start and end of selection, in bytes
uniform uint texwidth;    // width of data texture

uniform usampler2D romtex;  // data texture
uniform usampler2D annotex; // annotation texture



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

    // Compute which column, the row of the column, and the element index within it
    uint col = ac.x / (colwidth + colspace);
    uint row = ac.y;
    uint el_in_row = ac.x % (colwidth + colspace);

    // Handle points below the data or in the gutters between columns.
    if (el_in_row >= colwidth || row > colheight) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }

    // compute the element index in the array.
    uint elidx = (colwidth * colheight * col) + (row * colwidth) + el_in_row;

    // find the offset into the texture data.
    uint el_per_b = 8u / bpp; // elements per byte
    uint tex_off = (elidx / el_per_b) + dataoff; // byte into array
    uint tex_rem = elidx % el_per_b; // element into array; bits into array

    // Disabling endianness swap until we find a more reasonable way of expressing it.
    //if (swap_endian == true && bitstride > 8u) {
    //    uint bytes = bitstride / 8u;
    //    tex_off = ((tex_off / bytes) * bytes) + (bytes - (1u+(tex_off % bytes)));
    //}

    // Handle points past the end of the data.
    if (tex_off >= datalen) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }


    // Find the data in the texture.
    uint tex_off_x = tex_off % texwidth;
    uint tex_off_y = tex_off / texwidth;
    uint tex_byte = texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r;

    uint tex_mask = (1u<<bpp)-1u;
    uint tex_shift = 7u - (tex_rem * bpp);
    uint tex_val = (tex_byte >> tex_shift) & tex_mask;
    float rv = float(tex_val); // / float(tex_mask);

    // get annotation
    uint anno = texelFetch(annotex, ivec2(int(tex_off_x),int(tex_off_y)),0).r;
    vec4 c = vec4(rv,rv+float(anno),rv, 1.0);
    if (anno != 0u) {
        c.r = 0.0; 
    }
    if (selection[0] != selection[1] && elidx >= selection[0] && elidx <= selection[1]+7u) {
        c.b = 0.0; c.g = 0.0;
    }
    color = c;
    //color = vec4(0.0,0.0,0.4,1.0);
}
