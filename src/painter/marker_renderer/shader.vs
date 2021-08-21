#version 460

in layout(location = 0) vec4 pos;
in layout(location = 1) vec3 tex_coords;
in layout(location = 3) float alpha;
out vec3 v_tex_coords;

void main() {
    gl_Position = pos;
    v_tex_coords = tex_coords;
}