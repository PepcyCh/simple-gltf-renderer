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

layout (set = 4, binding = 0) uniform textureCube skybox_tex;
layout (set = 4, binding = 1) uniform sampler skybox_tex_sampler;
layout (set = 4, binding = 2) uniform textureCube skybox_irradiance_tex;
layout (set = 4, binding = 3) uniform sampler skybox_irradiance_tex_sampler;
layout (set = 4, binding = 4) uniform textureCube skybox_prefiltered_tex;
layout (set = 4, binding = 5) uniform sampler skybox_prefiltered_tex_sampler;
layout (set = 4, binding = 6) uniform texture2D brdf_lut_tex;
layout (set = 4, binding = 7) uniform sampler brdf_lut_tex_sampler;

const float PI = 3.14159265359;
const vec3 DIELECTRTIC_R0 = vec3(0.04);
const vec3 AMBIENT = vec3(0.05);

float pow2(float x) {
    return x * x;
}

float pow5(float x) {
    float x2 = x * x;
    return x * x2 * x2;
}

vec3 SchlickFresnel(vec3 r0, float ndotv) {
    return r0 + (vec3(1.0) - r0) * pow5(1 - ndotv);
}

float NdfGgx(float ndoth, float a2) {
    return a2 / (PI * pow2(ndoth * ndoth * (a2 - 1) + 1));
}

float SeparableVisible(float ndotv, float ndotl, float a2) {
    float v = abs(ndotv) + sqrt((1 - a2) * ndotv * ndotv + a2);
    float l = abs(ndotl) + sqrt((1 - a2) * ndotl * ndotl + a2);
    return 1.0 / (v * l);
}

void main() {
    // albedo, alpha
    vec4 albedo_all = base_color * texture(sampler2D(base_color_tex, base_color_tex_sampler), v_texcoords);
    vec3 albedo = albedo_all.xyz;
    float alpha = albedo_all.a;
    if (alpha < 0.1) {
        discard;
    }

    // normal
    vec3 normal_tspace = texture(sampler2D(normal_tex, normal_tex_sampler), v_texcoords).xyz;
    normal_tspace = (normal_tspace - vec3(0.5)) * 2.0;
    vec3 normal_dir = normalize(
        v_tangent * normal_tspace.x +
        v_bitangent * normal_tspace.y +
        v_normal * normal_tspace.z
    );

    // roughness, metallic, fresnel_r0
    vec4 mr = texture(sampler2D(metallic_roughness_tex, metallic_roughness_tex_sampler), v_texcoords);
    float metallic = mr.b * metallic_factor;
    vec3 fresnel_r0 = mix(vec3(0.04), albedo, metallic);
    float p_roughness = mr.g * roughness_factor;
    float roughness = p_roughness * p_roughness;
    float roughness_sqr = roughness * roughness;

    // emissive
    vec3 emissive = emissive_factor * texture(sampler2D(emissive_tex, emissive_tex_sampler), v_texcoords).xyz;

    // light, view, half
    vec3 light_dir = mix(light_position.xyz, normalize(light_position.xyz - v_position), light_position.w);
    vec3 view_dir = normalize(camera_position - v_position);
    vec3 half_dir = normalize(view_dir + light_dir);

    // reflect
    vec3 reflect_dir = reflect(-view_dir, normal_dir);

    // dots
    float ndoth = max(dot(normal_dir, half_dir), 0.0);
    float ndotv = max(dot(normal_dir, view_dir), 0.0);
    float ndotl = max(dot(normal_dir, light_dir), 0.0);
    float hdotv = max(dot(half_dir, view_dir), 0.0);

    // diffuse
    vec3 diffuse = albedo / PI;

    // NDF
    float ndf = NdfGgx(ndoth, roughness_sqr);

    // Visible
    float visible = SeparableVisible(ndotv, ndotl, roughness_sqr);

    // Fresnel
    vec3 fresnel = SchlickFresnel(fresnel_r0, hdotv);

    // direct lighting
    vec3 direct_lighting = (diffuse * (vec3(1.0) - fresnel) + ndf * visible * fresnel) * light_color.xyz * ndotl;

    // indirect lighting
#ifdef FORWARD_BASE
    vec3 prefiltered_color = textureLod(samplerCube(skybox_prefiltered_tex, skybox_prefiltered_tex_sampler),
        reflect_dir, p_roughness * 6).rgb;
    vec2 brdf = texture(sampler2D(brdf_lut_tex, brdf_lut_tex_sampler), vec2(ndotv, p_roughness)).rg;
    vec3 indirect_specular = prefiltered_color * (fresnel * brdf.x + brdf.y);
    vec3 indirect_diffuse = texture(samplerCube(skybox_irradiance_tex, skybox_irradiance_tex_sampler), normal_dir).rgb;
    vec3 indirect_lighting = indirect_diffuse * (vec3(1.0) - fresnel) + indirect_specular;
#else
    vec3 indirect_lighting = vec3(0.0);
#endif

#ifdef FORWARD_BASE
    vec3 normal_visualized = (normal_dir + vec3(1.0)) * 0.5;
#else
    vec3 normal_visualized = vec3(0.0);
#endif

    // final color
//    vec3 final_color = albedo;
//    vec3 final_color = normal_visualized;
//    vec3 final_color = vec3(ndf);
//    vec3 final_color = vec3(visible);
//    vec3 final_color = fresnel;
//    vec3 final_color = emissive;
//    vec3 final_color = direct_lighting;
//    vec3 final_color = indirect_lighting;
    vec3 final_color = direct_lighting + emissive + indirect_lighting;
    f_color = vec4(final_color, alpha);
}
