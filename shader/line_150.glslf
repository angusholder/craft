#version 150 core

out vec4 oTarget;
in vec4 fColor;

void main() {
    oTarget = fColor;
}
