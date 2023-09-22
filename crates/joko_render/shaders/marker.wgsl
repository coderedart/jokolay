struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) alpha: f32,
    @location(3) color: vec4<f32>,
    @location(4) fade_near_far: vec2<f32>,
}
struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};
struct UniformInput {
    transform: mat4x4<f32>,
    player_pos: vec3<f32>,
}

@group(0)
@binding(0)
var<uniform> uni: UniformInput;

@vertex
fn vs_main(
    vin: VertexInput
) -> VertexOutput {
    var result: VertexOutput;
    result.alpha = vin.alpha;

    var dist = distance(uni.player_pos, vin.position.xyz);
    if vin.fade_near_far.x > 0.0 && dist >= vin.fade_near_far.x {
            // if distance is exactly fade_near, we will multiply with 1.0
            // if its more, then we will multiply with how far we are in between fade_near and fade_far
        var ratio = 1.0 - (abs(dist - vin.fade_near_far.x) / abs(vin.fade_near_far.y - vin.fade_near_far.x));
            // The actual alpha
        result.alpha *= ratio;
    }
    if vin.fade_near_far.y > 0.0 && dist >= vin.fade_near_far.y {
        result.alpha = 0.0;
    }
    result.tex_coord = vin.tex_coord;
    result.position = uni.transform * vin.position;
    result.color = vin.color;

    return result;
}

@group(1) @binding(0) var r_tex_color: texture_2d<f32>;
@group(1) @binding(1) var r_tex_sampler: sampler;

@fragment
fn fs_main(vout: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(r_tex_color, r_tex_sampler, vout.tex_coord);
    color.a = color.a * vout.alpha;
    if color.a < 0.001 {
        discard;
    }
    return color;
}

