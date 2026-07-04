#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 f_normal;
layout(location = 1) out vec3 f_position;
layout(location = 2) out vec4 f_position_light;

layout(set = 0, binding = 0) uniform RenderVertexData {
    mat4 model;
    mat4 model_normals;
    mat4 view_proj_camera;
    mat4 view_proj_light;
} uniforms;

void main() {
    f_normal = normalize((uniforms.model_normals * vec4(normal, 1.0)).xyz);
    f_position = (uniforms.model * vec4(position, 1.0)).xyz;
    gl_Position = uniforms.view_proj_camera * vec4(f_position, 1.0);
    f_position_light = uniforms.view_proj_light * vec4(f_position, 1.0);
}
