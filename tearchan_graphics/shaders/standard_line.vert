#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(binding = 0) uniform Matrix {
    mat4 viewProjectionMatrix;
};

layout(location = 0) out vec4 outColor;

void main(void) {
    outColor = color;
    gl_Position = viewProjectionMatrix * vec4(position, 1.0);
}
