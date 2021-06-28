
#version 330

uniform sampler2D egui_texture;

in vec2 tex_coords_frag;
in vec4 tex_color;
out vec4 frag_color;

void main() {
  frag_color = tex_color * texture(egui_texture, tex_coords_frag);
  // frag_color = tex_color * vec4(0.2, 0.3, 0.8, 1.0);
  
}