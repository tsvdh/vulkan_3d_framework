#version 460

// --- Materials ---

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

// --- Lights ---

struct PointLight {
     vec3 position;
     bool used;
};

struct DirectionalLight {
     vec3 direction;
     bool used;
};

struct Lights {
     PointLight point_light;
     DirectionalLight directional_light;
};

vec3 get_light_dir(Lights lights, vec3 position) {
     if (lights.point_light.used) {
          return normalize(lights.point_light.position - position);
     }
     if (lights.directional_light.used) {
          return -lights.directional_light.direction;
     }
}

// --- Input and output ---

layout(location = 0) in vec3 f_normal;
layout(location = 1) in vec3 f_position;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform FragmentData {
     PhongMaterial material;
     Lights lights;
     vec3 camera_pos;
} uniforms;

// ------

void main() {
     // f_color = vec4((f_normal + 1) / 2, 1.0);

     vec3 light_dir = get_light_dir(uniforms.lights, f_position);
     vec3 camera_dir = normalize(uniforms.camera_pos - f_position);

     float diffuse_power = max(dot(f_normal, light_dir), 0);
     float specular_power = 0;
     if (diffuse_power > 0) {
          vec3 halfway = normalize(light_dir + camera_dir);
          specular_power = max(dot(f_normal, halfway), 0);
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
