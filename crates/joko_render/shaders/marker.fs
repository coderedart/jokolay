#version 450

layout(location = 0) in vec2 vtex_coord;
layout(location = 1) in float valpha;
layout(location = 2) in vec4 vcolor;

layout(location = 0) out vec4 ocolor;

layout(location = 1) uniform sampler2D sam;

void main() {
    vec4 color = texture(sam, vtex_coord, -2.0);
    color.a = color.a * valpha;
    if (color.a < 0.01) {
        discard;
    }
    ocolor = color;
}
