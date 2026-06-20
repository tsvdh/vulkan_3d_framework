#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 f_normal;
layout(location = 1) out vec3 f_position;

layout(set = 0, binding = 0) uniform VertexData {
    mat4 view_proj;
    mat4 model;
    mat4 model_normals;
    mat4 model_view_proj;
} uniforms;

void main() {
    f_normal = normalize((uniforms.model_normals * vec4(normal, 1.0)).xyz);
    f_position = (uniforms.model * vec4(position, 1.0)).xyz;
    gl_Position = uniforms.model_view_proj * vec4(position, 1.0);
}
