#version 330

// The texture coordinates are mapped over the window from (0.0,0.0) to (1.0,1.0)
in vec2 v_tex_coords;
out vec4 color;

// The data display is organized into a number of columns of a fixed height, displayed side by side with a specified spacing between them.

uniform uvec4 win;        // bounds of physical window

uniform float zoom;       // zoom factor (2.0 = 2x)
uniform vec2 ul_offset;   // offset of upper left hand corner in pixels at the current zoom level

uniform uvec2 column_dim;      // the width and height of a column, in elements
uniform uint column_spacing;   // spacing between adjacent columns, in elements

uniform uvec2 data_bounds;     // start and end points of data to display
uniform uint bpp;              // bits per pixel (1 for bitmap, 8 for bytemap, etc)

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
    uint intercolumn = column_dim[0] + column_spacing;
    uint col = ac.x / intercolumn;
    uint row = ac.y;
    uint el_in_row = ac.x % intercolumn;

    // Handle points below the data or in the gutters between columns.
    if (el_in_row >= column_dim[0]|| row > column_dim[1]) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }

    // compute the element index in the array.
    uint elidx = (column_dim[0] * column_dim[1] * col) + (row * column_dim[0]) + el_in_row;

    // find the offset into the texture data.
    uint el_per_b = 8u / bpp; // elements per byte
    uint tex_off = (elidx / el_per_b) + data_bounds[0]; // byte into array
    uint tex_rem = elidx % el_per_b; // element into array; bits into array

    // Disabling endianness swap until we find a more reasonable way of expressing it.
    //if (swap_endian == true && bitstride > 8u) {
    //    uint bytes = bitstride / 8u;
    //    tex_off = ((tex_off / bytes) * bytes) + (bytes - (1u+(tex_off % bytes)));
    //}

    // Handle points past the end of the data.
    if (tex_off >= data_bounds[1]) {
        color = vec4(0.0,0.0,0.4,1.0);
        return;
    }


    // Find the data in the texture.
    uint tex_off_x = tex_off % texwidth;
    uint tex_off_y = tex_off / texwidth;
    uint tex_byte = texelFetch(romtex, ivec2(int(tex_off_x),int(tex_off_y)),0).r;

    // Mask: 1bpp = 1, 2bpp = 0x3, 4bpp = 0x0f, 8bpp = 0xff
    uint tex_mask = (1u<<bpp)-1u;
    // Shift for this element. We start at the high bits. (8-bpp) - (elem*bpp)
    uint tex_shift = (8u - bpp) - (tex_rem * bpp);
    uint tex_val = (tex_byte >> tex_shift) & tex_mask;
    float rv = float(tex_val) / float(tex_mask);

    // get annotation
    uint anno = texelFetch(annotex, ivec2(int(tex_off_x),int(tex_off_y)),0).r;
    vec4 c = vec4(rv,rv+float(anno),rv, 1.0);
    if (anno != 0u) {
        c.r = 0.0; 
    }
    uint select_start = selection[0] * el_per_b;
    uint select_end = selection[1] * el_per_b;
    if (select_start != select_end && elidx >= select_start && elidx <= select_end) {
        c.b = 0.0; c.g = 0.0;
    }
    color = c;
    //color = vec4(0.0,0.0,0.4,1.0);
}
