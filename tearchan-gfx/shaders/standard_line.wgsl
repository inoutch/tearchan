struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = r_locals.transform * vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}