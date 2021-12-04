#version 460

uniform sampler2DArray sampler;

in vec3 v_tc;
out vec4 color;
void main() {
    color = texture(sampler,v_tc);
    if (color.a < 0.1) {
        discard;
    }
}
/*
[06:07] Jakey: this is how I'd implement atlas wrapping in the fragment shader
(hopefully the wrapping math is correct :KEKW:)
layout(std140, binding = 0) uniform AtlasUVs
{
  vec4 uvs[ATLAS_ENTRY_COUNT];
};

in flat int vAtlasTextureIndex;
in vec2 vTexCoord; // normalized

vec2 GetActualUV(vec2 uv, int atlasIndex)
{
  vec4 ua = uvs[atlasIndex];
  return mod(uv * (ua.zw - ua.xy), ua-zw - ua.xy) + ua.xy; // repeat wrapping
}

int main()
{
  vec2 actualUV = GetActualUV(vTexCoord, vAtlasTextureIndex);
  // ... do whatever you want with actualUV
}
*/