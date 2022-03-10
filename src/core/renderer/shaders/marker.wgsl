
struct VertexOutput {
@location(0) color: vec4<f32>  ;
@location(1) tc: vec2<f32>  ;
@builtin(position) pos: vec4<f32>;
};

struct VertexInput {
@location(0) pos: vec2<f32>;
@location(1) tc: vec2<f32>;
@location(2) color: vec4<f32>;
};




@stage(vertex)
fn vs_main(input: VertexInput ) -> VertexOutput {
let pos = input.pos;
var output: VertexOutput;
  output.pos = pos ;
  output.color = input.color;
  output.tc =  input.tc;
  return output;
}


@group(1) @binding(0) var u_sampler: sampler;
@group(1) @binding(1) var u_texture: texture_2d<f32>;

@stage(fragment)
fn fs_main(
in: VertexOutput
)
-> @location(0) vec4<f32>
{
return in.color *
textureSample(u_texture, u_sampler, in.tc);
}
