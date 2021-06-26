#version 330

layout (location = 0) in vec3 Position;
//uniform vec3 cpos;
void main()
{
    gl_Position = vec4(Position, 1.0);
}