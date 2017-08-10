#version 150 core

uniform ivec2 uChunkOffset;
uniform mat4 uWorldToScreen;

in int x, y, z;
in float u, v;

out vec2 vTexCoord;

void main() {
    vTexCoord = vec2(u, v) / 16.0;

    ivec3 pos = ivec3(x, y, z);
    pos.x += uChunkOffset.x * 16;
    pos.z += uChunkOffset.y * 16;
    gl_Position = uWorldToScreen * vec4(vec3(pos), 1.0);

    //gl_ClipDistance[0] = 1.0;
}
