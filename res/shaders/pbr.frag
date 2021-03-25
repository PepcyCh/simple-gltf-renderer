#version 450

layout (location = 0) in vec3 v_position;
layout (location = 1) in vec2 v_texcoords;
layout (location = 2) in vec3 v_normal;
layout (location = 3) in vec3 v_tangent;
layout (location = 4) in vec3 v_bitangent;

layout (location = 0) out vec4 f_color;

layout (set = 0, binding = 0) uniform MaterialUniform {
    vec4 base_color;
    vec3 emissive_factor;
    float metallic_factor;
    float roughness_factor;
};

layout (set = 0, binding = 1) uniform texture2D base_color_tex;
layout (set = 0, binding = 2) uniform sampler base_color_tex_sampler;

layout (set = 0, binding = 3) uniform texture2D normal_tex;
layout (set = 0, binding = 4) uniform sampler normal_tex_sampler;

layout (set = 0, binding = 5) uniform texture2D metallic_roughness_tex;
layout (set = 0, binding = 6) uniform sampler metallic_roughness_tex_sampler;

layout (set = 0, binding = 7) uniform texture2D emissive_tex;
layout (set = 0, binding = 8) uniform sampler emissive_tex_sampler;

layout (set = 1, binding = 0) uniform ObjectUniform {
    mat4 matrix_model;
    mat4 matrix_model_iv;
};

layout (set = 2, binding = 0) uniform LightUniform {
    vec4 light_position;
    vec4 light_color;
};

layout (set = 3, binding = 0) uniform CameraUniform {
    mat4 matrix_view;
    mat4 matrix_proj;
    vec3 camera_position;
    float _padding0;
    float camera_znear;
    float camera_zfar;
};

void main() {
    vec4 albedo_all = base_color * texture(sampler2D(base_color_tex, base_color_tex_sampler), v_texcoords);
    vec3 albedo = albedo_all.xyz;
    float alpha = albedo_all.a;
    if (alpha < 0.1) {
        discard;
    }

    f_color = vec4(albedo, 1.0);
}
