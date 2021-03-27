#version 450

layout (location = 0) in vec3 a_position;

layout (location = 0) out vec3 v_position;

layout (set = 0, binding = 0) uniform CameraUniform {
    mat4 matrix_view;
    mat4 matrix_proj;
};

void main() {
    v_position = a_position;
    gl_Position = matrix_proj * matrix_view * vec4(a_position, 1.0);
}
