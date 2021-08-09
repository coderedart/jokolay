#version 460

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 tc;
layout(location = 2) in vec4 color;

out vec2 v_tc;
out vec4 v_color;

uniform vec2 screen_size;

vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, vec3(cutoff));
}

vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {

    gl_Position = vec4(2.0 * pos.x / screen_size.x - 1.0, 1.0 - 2.0 * pos.y / screen_size.y, 0.0, 1.0);
    v_tc = tc;
    v_color = linear_from_srgba(color);

}