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

// --- Input and output ---

layout(location = 0) in vec3 f_normal;
layout(location = 1) in vec3 f_position;
layout(location = 2) in vec4 f_position_light;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform RenderFragmentData {
     PhongMaterial material;
     Lights lights;
     vec3 camera_pos;
} uniforms;

layout(set = 0, binding = 2) uniform sampler2D shadow_map;

// --- Functionality ---

vec3 get_light_dir(Lights lights, vec3 position) {
     if (lights.point_light.used) {
          return normalize(lights.point_light.position - position);
     }
     if (lights.directional_light.used) {
          return -lights.directional_light.direction;
     }
}

float get_shadow(vec4 f_position_light, vec3 light_dir) {
     vec3 proj_position = f_position_light.xyz / f_position_light.w;
     float point_depth = proj_position.z;
     vec2 proj_coords = proj_position.xy * 0.5 + 0.5;
     float shadow_map_depth = texture(shadow_map, proj_coords).x;

     float bias = max(0.05 * (1 - dot(light_dir, f_normal)), 0.005);

     return point_depth <= (shadow_map_depth + 0.005) ? 1.0 : 0.2;
}

// ------

void main() {
//      f_color = vec4((f_normal + 1) / 2, 1.0);

     float a = uniforms.camera_pos.x;
     float b = texture(shadow_map, vec2(0, 0)).x;

//     vec3 proj_coords = f_position_light.xyz / f_position_light.w;
//     float point_depth = proj_coords.z;
//     proj_coords = proj_coords * 0.5 + 0.5;
//     float shadow_map_depth = texture(shadow_map, proj_coords.xy).x;
//     float point_depth = proj_coords.z;
//     f_color = vec4(vec3(point_depth), 1.0);

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

     float diffuse_shadow = get_shadow(f_position_light, light_dir);
     float specular_shadow = diffuse_shadow < 1.0 ? 0.0 : 1.0;

     f_color = vec4(
          ambient.color / 255 * ambient.coefficient
          + diffuse.color / 255 * diffuse.coefficient * diffuse_power * diffuse_shadow
          + specular.color / 255 * specular.coefficient * specular_power * specular_shadow,
          1.0
     );
}
