#version 330                                                                        
                                                                                    
layout(points) in;                                                                  
layout(triangle_strip) out;                                                         
layout(max_vertices = 4) out;                                                       
                                                                                    
uniform mat4 VP;                                                                   
uniform vec3 cam_pos;                                                            
                                                                                    
out vec2 tex_coords;                                                                  
                                                                                    
void main()                                                                         
{                                                                                   
    vec3 Pos = gl_in[0].gl_Position.xyz;                                            
    vec3 toCamera = normalize(cam_pos - Pos);                                    
    vec3 up = vec3(0.0, 1.0, 0.0);                                                  
    vec3 right = cross(toCamera, up);                                               
                                                                                    
    Pos -= (right * 0.5);                                                           
    gl_Position = VP * vec4(Pos, 1.0);                                             
    tex_coords = vec2(0.0, 0.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    Pos.y += 1.0;                                                                   
    gl_Position = VP * vec4(Pos, 1.0);                                             
    tex_coords = vec2(0.0, 1.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    Pos.y -= 1.0;                                                                   
    Pos += (right * 1.0);                                                                   
    gl_Position = VP * vec4(Pos, 1.0);                                             
    tex_coords = vec2(1.0, 0.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    Pos.y += 1.0;                                                                   
    gl_Position = VP * vec4(Pos, 1.0);                                             
    tex_coords = vec2(1.0, 1.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    EndPrimitive();                                                                 
}             