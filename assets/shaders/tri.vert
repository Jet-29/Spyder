#version 450

vec4 positions[3] = vec4[](
vec4(0.0, -0.5, 0.0, 1.0),
vec4(0.5, 0.5, 0.0, 1.0),
vec4(-0.5, 0.5, 0.0, 1.0)
);

vec4 colours[3] = vec4[](
vec4(1., 0., 0., 1.),
vec4(0., 1., 0., 1.),
vec4(0., 0., 1., 1.)
);

layout (location=0) out vec4 colour;

void main() {
    colour = colours[gl_VertexIndex];
    gl_Position = positions[gl_VertexIndex];
}