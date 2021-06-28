#version 330

layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 tex_coords;
layout (location = 2) in vec4 color; //0-255.0 range of colors
out vec2 tex_coords_frag;
out vec4 tex_color;

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
void main()
{
    gl_Position = vec4(2.0 * Position.x / screen_size.x - 1.0,
    1.0 - 2.0 * Position.y / screen_size.y,
     0.0,
      1.0);
    tex_coords_frag =  tex_coords;
    // tex_color = vec4(color.x / 255.0 , color.y / 255.0 , color.z / 255.0  , color.z / 255.0); 
    tex_color = linear_from_srgba(color);
}