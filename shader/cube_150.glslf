#version 150 core

in vec2 vTexCoord;
out vec4 oTarget;
uniform sampler2D tBlocks;

void main() {
    oTarget = texture(tBlocks, vTexCoord);
}
