struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] texcoord: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Locals {
    transform: mat4x4<f32>;
};
struct BillboardCamera {
    right: vec3<f32>;
    up: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> r_locals: Locals;
[[group(0), binding(1)]]
var<uniform> r_camera: BillboardCamera;
[[group(0), binding(2)]]
var r_texture: texture_2d<f32>;
[[group(0), binding(3)]]
var r_sampler: sampler;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] texcoord: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
    [[location(3)]] origin: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let pos = origin + r_camera.right * position.x + r_camera.up * position.y;

    out.position = r_locals.transform * vec4<f32>(pos, 1.0);
    out.texcoord = texcoord;
    out.color = color;
    return out;
}

struct FragmentOutput {
    [[location(0)]] target: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.target = textureSample(r_texture, r_sampler, in.texcoord) * in.color;
    return out;
}
