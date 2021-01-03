#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in vec3 normal;

layout(binding = 0) uniform Matrix {
    mat4 viewProjectionMatrix;
};
layout(binding = 1) uniform Position {
    vec3 lightPosition;
};

layout(location = 0) out vec4 vColor;
layout(location = 1) out vec2 vTexcoord;
layout(location = 2) out vec3 vNormal;
layout(location = 3) out vec3 vLightDir;

void main(void) {
    vColor = color;
    vTexcoord = texcoord;
    vNormal = normalize(normal);
    vLightDir = normalize(lightPosition - position);
    gl_Position = viewProjectionMatrix * vec4(position, 1.0);
}
