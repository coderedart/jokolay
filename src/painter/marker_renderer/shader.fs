#version 460

in vec3 v_tex_coords;

out vec4 color;

uniform sampler2DArray sampler;


void main() {
    color = texture(sampler, v_tex_coords);
}