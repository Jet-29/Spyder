#version 460

layout (location=0) in vec4 in_position;
layout (location=1) in float in_size;
layout (location=2) in vec4 in_colour;

layout (location=0) out vec4 out_colour;

void main() {
    gl_PointSize = in_size;
    out_colour = in_colour;
    gl_Position = in_position;
}