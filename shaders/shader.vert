#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo; 

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 velocity;

layout(location = 0) out vec4 fragColor;

void main() {
    gl_PointSize = 2.0;
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragColor = vec4(1.0, 1.0, 1.0, 0.1);
}
