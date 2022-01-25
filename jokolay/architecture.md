An Overlay Window that handles position/size/transparent/focus/alwaysontop attributes and collects events from glfw window. 
A client object which waits for events from main window and after processing them, sends back any data/commands back to the main window. 
A Scene Object managest opengl state for its scene. we have a egui, marker and trail scenes probably. 

An App class that holds all of these and runs an event loop. 



shaders load/compile
marker category/POIs
get map from jokolink
load POIs and setup buffers when loading into a new map
get camera/fvars from link
draw to screen

