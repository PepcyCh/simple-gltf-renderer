#version 450

layout (location = 0) in vec3 v_position;

layout (location = 0) out vec4 f_color;

layout (set = 1, binding = 0) uniform textureCube skybox_tex;
layout (set = 1, binding = 1) uniform sampler skybox_tex_sampler;

const float PI = 3.14159265359;

void main() {
    vec3 normal_dir = normalize(v_position);
    vec3 bitangent_dir = vec3(0.0, 1.0, 0.0);
    vec3 tangent_dir = cross(bitangent_dir, normal_dir);
    bitangent_dir = cross(normal_dir, tangent_dir);

    vec3 irradiance = vec3(0.0);
    int sample_count = 0;
    float delta = 0.05;
    for (float theta = 0.0; theta < 2.0 * PI; theta += delta) {
        for (float phi = 0.0; phi < 0.5 * PI; phi += delta) {
            vec3 sample_dir = vec3(sin(phi) * cos(theta), sin(phi) * sin(theta), cos(phi));
            sample_dir = tangent_dir * sample_dir.x + bitangent_dir * sample_dir.y + normal_dir * sample_dir.z;
            irradiance += texture(samplerCube(skybox_tex, skybox_tex_sampler), sample_dir).rgb * cos(phi) * sin(phi);
            sample_count += 1;
        }
    }
//    irradiance = PI * irradiance / sample_count;
    irradiance = irradiance / sample_count;
    f_color = vec4(irradiance, 1.0);
}
