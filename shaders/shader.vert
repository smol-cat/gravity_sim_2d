#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 velocity;

layout(location = 0) out vec4 fragColor;

void main() {
    gl_PointSize = 3.0;
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragColor = vec4(1.0, 0.2, 1.0, 0.01);
}
