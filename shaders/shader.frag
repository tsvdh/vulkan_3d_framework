#version 460

struct PhongComponent {
     vec3 color;
     float coefficient;
};

struct PhongMaterial {
     PhongComponent ambient;
     PhongComponent diffuse;
     PhongComponent specular;
     uint shininess;
};

layout(location = 0) in vec3 f_normal;
layout(location = 1) in vec3 f_world_position;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform FragmentData {
     PhongMaterial material;
     vec3 light_pos;
     vec3 eye_pos;
} uniforms;

void main() {
     // f_color = vec4((f_normal + 1) / 2, 1.0);

     vec3 light_dir = normalize(uniforms.light_pos - f_world_position);
     vec3 eye_dir = normalize(uniforms.eye_pos - f_world_position);

     float diffuse_power = max(dot(f_normal, light_dir), 0);
     float specular_power = 0;
     if (diffuse_power > 0) {
          vec3 refl_light_dir = reflect(-light_dir, f_normal);
          specular_power = max(dot(eye_dir, refl_light_dir), 0);
          specular_power = pow(specular_power, uniforms.material.shininess);
     }

     PhongComponent ambient = uniforms.material.ambient;
     PhongComponent diffuse = uniforms.material.diffuse;
     PhongComponent specular = uniforms.material.specular;

     f_color = vec4(
          ambient.color / 255 * ambient.coefficient
          + diffuse.color / 255 * diffuse.coefficient * diffuse_power
          + specular.color / 255 * specular.coefficient * specular_power,
          1.0
     );
}
