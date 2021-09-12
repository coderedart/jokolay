#version 460

uniform sampler2DArray sampler;
uniform int sampler_layer;
uniform float tc_x_offset;
uniform float tc_y_offset;
uniform float tc_x_scale;
uniform float tc_y_scale;

in vec2 v_tc;
in vec4 v_color;
out vec4 f_color;

void main() {
    // we scale the range of (0.0, 1.0) in model space to the whole texture atlast space by mutiplying with a scale of length of texture/ length of atlas. then, we add the offset
    // to consider that texture won't always be at the border.
    f_color =  v_color * texture(sampler, vec3(tc_x_offset +  (v_tc.x * tc_x_scale), tc_y_offset + (v_tc.y * tc_y_scale), sampler_layer)) ;
}