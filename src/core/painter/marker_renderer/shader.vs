#version 460

in layout(location = 0) vec4 pos;
in layout(location = 1) vec3 tex_coords;
in layout(location = 2) float alpha;
out vec3 v_tex_coords;
out float v_alpha;
void main() {
    gl_Position = pos;
    v_tex_coords = tex_coords;
    v_alpha = alpha;
}