#version 450

layout(location = 0) in vec4 color;
layout(location = 1) in vec3 surfaceNormal;
layout(location = 2) in vec3 toLightVector;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 unitNormal = normalize(surfaceNormal);
    vec3 unitLight = normalize(toLightVector);

    float nDotl = max(dot(unitNormal,unitLight),0.5);

    f_color = vec4(color.xyz*nDotl,color.w);
}