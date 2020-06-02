#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in vec3 normal;

layout(binding = 0) uniform Matrix {
    mat4 viewProjectionMatrix;
};
layout(binding = 1) uniform Matrix2 {
    mat4 invViewProjectionMatrix;
};
layout(binding = 3) uniform Position {
    vec3 lightPosition;
};

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec2 outTexcoord;
layout(location = 2) out vec3 outNormal;
layout(location = 3) out vec3 outLightDir;

void main(void) {
    outColor = color;
    outTexcoord = texcoord;
    outNormal = normalize(normal);
    outLightDir = normalize(lightPosition - position);
    gl_Position = viewProjectionMatrix * vec4(position, 1.0);
}
