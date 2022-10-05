
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc_pos: vec2<f32>
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    
    result.position = vec4<f32>(position.xy, 0.5, 1.0);
    result.ndc_pos = position;
    return result;
}


@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let alpha: f32  = distance(vertex.ndc_pos.xy, vec2<f32>(0.0, 0.0));
    return vec4<f32>(0.0, 0.0, 0.0, pow(alpha, 5.0) / 2.0);
}

