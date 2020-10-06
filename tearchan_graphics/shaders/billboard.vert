#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in vec3 origin;

layout(binding = 0) uniform Matrix {
    mat4 viewProjectionMatrix;
};

layout(binding = 2) uniform BillboardCamera {
    vec3 cameraRight;
    vec3 cameraUp;
};

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec2 outTexcoord;

void main(void) {
    outColor = color;
    outTexcoord = texcoord;
    vec3 p = origin + cameraRight * position.x + cameraUp * position.y;
    gl_Position = viewProjectionMatrix * vec4(p, 1.0);
}
