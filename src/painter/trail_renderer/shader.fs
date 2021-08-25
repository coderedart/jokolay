#version 460

uniform sampler2DArray sampler;

in vec3 v_tc;
out vec4 color;
void main() {
    color = texture(sampler,v_tc);
    if (color.a < 0.1) {
        discard;
    }
}