#version 450 core

layout(location = 0) in vec4 color;
layout(location = 0) out vec4 outColor;

void main(void) {
    outColor = color;
}
