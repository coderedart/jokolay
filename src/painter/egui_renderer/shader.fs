#version 460

uniform sampler2DArray sampler;
uniform int sampler_layer;
in vec2 v_tc;
in vec4 v_color;
out vec4 f_color;

void main() {
    // egui texture dimensions = 2048x64. actual texture dimensions = 2048x2048. so, we normalize egui height dimension with ratio of 64/2048 to convert it into actual tex dimensions
    f_color =  v_color * texture(sampler, vec3(v_tc.x, v_tc.y * 0.03125, sampler_layer)) ;
}