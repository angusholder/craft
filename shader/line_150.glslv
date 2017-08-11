#version 150 core

uniform mat4 uClipFromWorld;

in vec3 pos;
in vec4 color;

out vec4 fColor;

void main() {
    fColor = color;
    gl_Position = uClipFromWorld * vec4(vec3(pos), 1.0);
}
