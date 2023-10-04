#version 460

layout (location=0) in vec4 position;

layout (location=0) out vec4 out_colour;

void main() {
    out_colour = vec4(1., 0., 0., 1.);
    gl_Position = position;
}