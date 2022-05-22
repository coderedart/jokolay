### bevy_glfw
This crate is intended to replace bevy_winit as the windowing plugin for *desktop* platforms. web is not supported.
the main motivation was glfw's stability, considering most of the vulkan ecosystem uses it across a wide variety of programming languages.
this crate uses a patched glfw-rs to support `MOUSE_PASSTHROUGH` feature which is still not released in stable glfw version.




### NOTE
this crate is primarily intended to be used by `Jokolay` overlay internally. once `MOUSE_PASSTHROUGH` feature lands in glfw stable,
I will change dependency to piston's glfw-rs crate. 

### Usage
As you only need one windowing backend, disable bevy_winit and add bevy_glfw plugin when initializing the bevy application. 
 