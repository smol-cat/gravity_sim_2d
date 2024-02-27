#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 velocity;

layout(location = 0) out vec4 fragColor;

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    gl_PointSize = 2.0;
    gl_Position = vec4(inPosition, 0.0, 1.0);
    float flatVelocity = length(velocity);
    float factor = 0.02;
    float hue = -factor / (flatVelocity + factor) + 1;
    fragColor = vec4(hsv2rgb(vec3(hue, 1.0, 1.0)), 0.03);
}
