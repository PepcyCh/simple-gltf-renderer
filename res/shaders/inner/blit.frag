#version 450

layout (location = 0) in vec2 v_texcoords;

layout (location = 0) out vec4 f_color;

layout (set = 0, binding = 0) uniform texture2D tex;
layout (set = 0, binding = 1) uniform sampler tex_sampler;

void main() {
    f_color = texture(sampler2D(tex, tex_sampler), v_texcoords);
}
