#version 450          
layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 surfaceNormal;
layout(location = 2) out vec3 toLightVector;

void main() {
    vec4 worldPosition = ubo.model * vec4(position, 1.0);
    gl_Position = ubo.proj * ubo.view * worldPosition;
    fragColor = color;

    surfaceNormal = ( ubo.model * vec4(normal,0.0)).xyz;
    toLightVector = inverse(ubo.view)[3].xyz - worldPosition.xyz; //
}