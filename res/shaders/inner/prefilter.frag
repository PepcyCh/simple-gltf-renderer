#version 450

layout (location = 0) in vec3 v_position;

layout (location = 0) out vec4 f_color;

layout (set = 1, binding = 0) uniform textureCube skybox_tex;
layout (set = 1, binding = 1) uniform sampler skybox_tex_sampler;
layout (set = 1, binding = 2) uniform Uniform {
    float u_roughness;
};

const float PI = 3.14159265359;
const int SAMPLE_COUNT = 1024;

float RadicalInverseVdc(uint bits) {
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}

vec2 Hammersley(int i, int n) {
    return vec2(float(i) / float(n), RadicalInverseVdc(uint(i)));
}

vec3 ImportanceSampleGgx(vec2 sam, vec3 normal_dir, float a2) {
    float theta = 2.0 * PI * sam.x;
    float cos_phi_sqr = (1.0 - sam.y) / (1.0 + (a2 - 1.0) * sam.y);
    float cos_phi = sqrt(cos_phi_sqr);
    float sin_phi = sqrt(1.0 - cos_phi_sqr);

    vec3 half_dir = vec3(sin_phi * cos(theta), sin_phi * sin(theta), cos_phi);
    vec3 bitangent_dir = vec3(0.0, 1.0, 0.0);
    vec3 tangent_dir = cross(bitangent_dir, normal_dir);
    bitangent_dir = cross(normal_dir, tangent_dir);
    vec3 sample_dir = tangent_dir * half_dir.x + bitangent_dir * half_dir.y + normal_dir * half_dir.z;
    return sample_dir;
}

float pow2(float x) {
    return x * x;
}

float NdfGgx(float ndoth, float a2) {
    return a2 / max(PI * pow2(ndoth * ndoth * (a2 - 1) + 1), 0.0001);
}

void main() {
    vec3 normal_dir = normalize(v_position);
    vec3 reflect_dir = normal_dir;
    vec3 view_dir = normal_dir;

    vec3 prefiltered_color = vec3(0.0);
    float weight_sum = 0.0;

    float roughness = u_roughness * u_roughness;
    float roughness_sqr = roughness * roughness;

    for (int i = 0; i < SAMPLE_COUNT; i++) {
        vec2 sam = Hammersley(i, SAMPLE_COUNT);
        vec3 half_dir = ImportanceSampleGgx(sam, normal_dir, roughness_sqr);
        vec3 light_dir = reflect(-view_dir, half_dir);
        float ndotl = max(dot(normal_dir, light_dir), 0.0);
        if (ndotl > 0.0) {
            float ndoth = max(dot(normal_dir, half_dir), 0.0);
            float hdotv = max(dot(half_dir, view_dir), 0.0);
            float dis = NdfGgx(ndoth, roughness_sqr);
            float pdf = dis * ndoth / (4 * hdotv) + 0.0001;
            prefiltered_color += texture(samplerCube(skybox_tex, skybox_tex_sampler), light_dir).rgb * ndotl;
            weight_sum += ndotl;
        }
    }

    prefiltered_color /= weight_sum;
    f_color = vec4(prefiltered_color, 1.0);
}
