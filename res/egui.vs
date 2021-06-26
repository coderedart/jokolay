#version 330

layout (location = 0) in vec2 Position;
// layout (location = 1) in vec2 tex_coords;
// layout (location = 3) in uint color;
// out vec2 tex_coords_frag;
void main()
{
    gl_Position = vec4(Position , 0.3, 1.0);
    // tex_coords_frag = tex_coords;
}