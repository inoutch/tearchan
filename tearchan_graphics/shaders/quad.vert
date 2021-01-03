#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 texcoord;
layout(location = 0) out vec2 vTexcoord;
layout(location = 1) out vec4 vColor;

layout(binding = 2) uniform Matrix {
    mat4 viewProjectionMatrix;
};
out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    vTexcoord = texcoord;
    vColor = color;
    gl_Position = viewProjectionMatrix * vec4(position, 1.0);
}