#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 vTexcoord;
layout(location = 1) in vec4 vColor;
layout(location = 0) out vec4 target0;

layout(set = 0, binding = 0) uniform sampler2D u_sampler;

layout(binding = 1) uniform Color {
    vec4 color;
};

void main() {
    target0 = color * texture(u_sampler, vTexcoord) * vColor;
}