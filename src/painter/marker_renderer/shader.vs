#version 460

in layout(location = 0) vec3 pos;
in layout(location = 1) vec3 tex_coords;

out vec3 v_tex_coords;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_tex_coords = tex_coords;
}