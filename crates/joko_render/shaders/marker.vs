#version 450

layout(location = 0) in vec4 position;
layout(location = 1) in float alpha;
layout(location = 2) in vec2 tex_coord;
layout(location = 3) in vec2 fade_near_far;
layout(location = 4) in vec4 color;

layout(location = 0) out vec2 vtex_coord;
layout(location = 1) out float valpha;
layout(location = 2) out vec4 vcolor;


layout(location = 0) uniform vec3 camera_pos;
// location 1 is for sampler in frag shader
layout(location = 2) uniform mat4 transform;


void main(
)  {
    valpha = alpha;
    vtex_coord = tex_coord;
    gl_Position = transform * position;
    vcolor = color;

    float dist = distance(camera_pos, position.xyz);
    if (fade_near_far.x > 0.0 && dist >= fade_near_far.x) {
            // if distance is exactly fade_near, we will multiply with 1.0
            // if its more, then we will multiply with how far we are in between fade_near and fade_far
        float ratio = 1.0 - (abs(dist - fade_near_far.x) / abs(fade_near_far.y - fade_near_far.x));
            // The actual alpha
        valpha *= ratio;
    }
    if (fade_near_far.y > 0.0 && dist >= fade_near_far.y) {
        valpha = 0.0;
    }
}


