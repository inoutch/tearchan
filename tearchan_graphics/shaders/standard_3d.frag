#version 450

layout(set = 0, binding = 2) uniform sampler2D texSampler;
layout(binding = 3) uniform Color {
    vec3 lightColor;
};
layout(binding = 4) uniform Ambient {
    float ambientStrength;
};

layout(location = 0) in vec4 vColor;
layout(location = 1) in vec2 vTexcoord;
layout(location = 2) in vec3 vNormal;
layout(location = 3) in vec3 vLightDir;

layout(location = 0) out vec4 outColor;

void main(void) {
    vec3 ambient = ambientStrength * lightColor;

    float diff = max(dot(vNormal, vLightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    vec3 lightEffect = ambient + diffuse;
    outColor = texture(texSampler, vTexcoord) * vColor * vec4(lightEffect, 1.0);
}
