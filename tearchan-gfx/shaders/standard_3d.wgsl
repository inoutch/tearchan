struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] texcoord: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] frag_pos: vec3<f32>;
};

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
    [[location(3)]] normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = r_locals.transform * vec4<f32>(position, 1.0);
    out.texcoord = texcoord;
    out.color = color;
    out.normal = normal;
    out.frag_pos = position;
    return out;
}

struct FragmentOutput {
    [[location(0)]] target: vec4<f32>;
};
struct LightAmbient {
    strength: f32;
};
struct LightColor {
    color: vec4<f32>;
};
struct LightPosition {
    position: vec4<f32>;
};

[[group(0), binding(1)]]
var r_texture: texture_2d<f32>;
[[group(0), binding(2)]]
var r_sampler: sampler;
[[group(0), binding(3)]]
var<uniform> r_light_ambient: LightAmbient;
[[group(0), binding(4)]]
var<uniform> r_light_color: LightColor;
[[group(0), binding(5)]]
var<uniform> r_light_position: LightPosition;

fn calc_light(normal: vec3<f32>, frag_pos: vec3<f32>) -> vec3<f32> {
    let ambient = r_light_ambient.strength * r_light_color.color.rgb;

    let light_dir = normalize(r_light_position.position.xyz - frag_pos);
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * r_light_color.color.rgb;

    return ambient + diffuse;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let light = calc_light(in.normal, in.frag_pos);
    let tex = textureSample(r_texture, r_sampler, in.texcoord);
    out.target = vec4<f32>(tex.xyz, 1.0) * in.color * vec4<f32>(light, 1.0);

    return out;
}
