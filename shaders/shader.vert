#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 f_normal;
layout(location = 1) out vec3 f_position_camera_space;

layout(set = 0, binding = 0) uniform VertexData {
    mat4 proj;
    mat4 model_view;
    mat4 model_view_normals;
    mat4 model_view_proj;
} uniforms;

void main() {
    f_normal = normalize((uniforms.model_view_normals * vec4(normal, 1.0)).xyz);
    f_position_camera_space = (uniforms.model_view * vec4(position, 1.0)).xyz;
    gl_Position = uniforms.model_view_proj * vec4(position, 1.0);
}
