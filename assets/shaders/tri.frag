#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec4 colour;



void main(){
    theColour = colour;
}