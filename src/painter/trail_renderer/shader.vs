#version 460

in layout(location = 0) vec4 position;
in layout(location = 1) vec3 tc;
in layout(location = 2) float alpha;
out vec3 v_tc;
void main() {
    gl_Position =  position;
    v_tc = tc;
}