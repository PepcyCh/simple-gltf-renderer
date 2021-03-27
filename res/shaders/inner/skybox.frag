#version 450

layout (location = 0) in vec3 v_position;

layout (location = 0) out vec4 f_color;

layout (set = 1, binding = 0) uniform textureCube skybox_tex;
layout (set = 1, binding = 1) uniform sampler skybox_tex_sampler;

void main() {
    vec3 final_color = texture(samplerCube(skybox_tex, skybox_tex_sampler), v_position).rgb;
    f_color = vec4(final_color, 1.0);
}
