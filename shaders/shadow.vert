#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(set = 0, binding = 0) uniform ShadowVertexData {
    mat4 mvp_light;
} uniforms;

void main() {
    gl_Position = uniforms.mvp_light * vec4(position, 1.0);
}
