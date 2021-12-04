#version 460

in vec3 v_tex_coords;
in float v_alpha;
out vec4 color;

uniform sampler2DArray sampler;


void main() {
    color = texture(sampler, v_tex_coords);
    if (color.a < 0.1) {
        discard;
    }
    color.a = color.a * v_alpha;

}