#version 450

#include "included.glsl"

void main() {
    float pos = float(random_number());
    gl_Position = vec4(pos, 0.0, 0.0, 1.0);
}
