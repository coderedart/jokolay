#version 450


layout(push_constant) uniform PushConstants {vec2 u_screen_size;} push_constant;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tc;
layout(location = 2) in vec4 color;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_tc;

// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, cutoff);
}

vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a);
}
vec4 toLinear(vec4 sRGB)
{
  bvec3 cutoff = lessThan(sRGB.rgb, vec3(0.04045));
  vec3 higher = pow((sRGB.rgb + vec3(0.055))/vec3(1.055), vec3(2.4));
  vec3 lower = sRGB.rgb/vec3(12.92);

  return vec4(mix(higher, lower, cutoff), sRGB.a);
}
void main() {
  gl_Position = vec4(2.0 * pos.x / push_constant.u_screen_size.x - 1.0,
                      2.0 * pos.y / push_constant.u_screen_size.y - 1.0, 0.0 , 1.0) ;
  // egui encodes vertex colors in gamma space, so we must decode the colors
  // here:
  vec4 rgba_color = toLinear(color);

  v_color = rgba_color;
  v_tc = tc;
}
