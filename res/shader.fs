
#version 330                                                                        
                                                                                    
uniform sampler2D color_map;                                                        
                                                                                    
in vec2 tex_coords;                                                                   
out vec4 frag_color;                                                                 
                                                                                    
void main()                                                                         
{                                                                                   
    frag_color = texture2D(color_map, tex_coords);                                     
                                                                                    
    if (frag_color.a == 0) {
        discard;                                                                    
    }     
}