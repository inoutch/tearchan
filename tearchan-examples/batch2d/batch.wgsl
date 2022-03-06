struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] texcoord: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[block]]
struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] texcoord: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = r_locals.transform * vec4<f32>(position, 1.0);
    out.texcoord = texcoord;
    out.color = color;
    return out;
}

[[group(0), binding(1)]]
var r_texture: texture_2d<f32>;
[[group(0), binding(2)]]
var r_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let tex = textureSample(r_texture, r_sampler, in.texcoord);
    let mag = length(in.texcoord - vec2<f32>(0.5));
    return vec4<f32>(mix(tex.xyz, vec3<f32>(0.0), mag * mag), 1.0) * in.color;
}