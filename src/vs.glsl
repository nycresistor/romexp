#version 130
in vec2 position;
in vec2 tex_coords;
out vec2 v_tex_coords;
uniform vec2 center_point;
uniform float zoom;

void main() {
     gl_Position = vec4( (position-center_point) *zoom, 0.0, 1.0);
     v_tex_coords = tex_coords;
}
