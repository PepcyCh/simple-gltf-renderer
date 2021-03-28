#version 450

layout (location = 0) out vec3 v_position;

layout (set = 0, binding = 0) uniform CameraUniform {
    mat4 matrix_view;
    mat4 matrix_proj;
    mat4 matrix_view_inv;
    mat4 matrix_proj_inv;
};

void main() {
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    vec4 pos = vec4(2.0 * uv.x - 1.0, 2.0 * uv.y - 1.0, 1.0, 1.0);
    v_position = mat3(matrix_view_inv) * (matrix_proj_inv * pos).xyz;
    gl_Position = pos;
}