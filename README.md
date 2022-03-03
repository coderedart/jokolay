# jokolay
An Overlay for Guild Wars 2 in Rust



1. Jokolink: This is what you will run from the same wine prefix as you are running gw2 from. it reads the shared memory (https://wiki.guildwars2.com/wiki/API:MumbleLink ) of gw2 to get player location/camera pos/which character is logged in  etc..  and ofcourse, window location. it can then copy all that stuff as is into a shared memory file under /dev/shm for linux native apps to use, so there can be multiple overlays or other addons requesting data. 

2. jokoapi: yeah. just an api library to use in our overlay, but its fine for generic use too. the easiest part and its mostly just grunt work of copy pasting api endpoints and filling out all the required fields of structs etc.. if anyone wants to contribute, this is the best place and people with the any amount of programming can do this. the bottleneck here is literally copy/pasting and typing speed.
3. Jokolay: this is the actual overlay.there's a rewrite going on for the following reasons.


1. ~~read the marker pack into an internal representation to support editing them live as well as use proper UUID for performance and other reasons.~~
2. ~~make the structs themselves represent state for egui rather than separate window structs.~~
3. ~~seperate the rendering parts and core logic to allow future upgrade to wgpu-rs / vulkano.~~
4. ~~try to use the same texture manager for both egui and markers/trails.~~
5. ~~simplify the rendering to use only the simplest features and also allow for future performance upgrades if users hardware supports it. eg: SSBO, GS, persistent mapped buffers (dynamic resizing) and so on.~~

much of the rewrite is done. we are preparing for a 0.2 alpha release by the first week of october.

i am running manjaro/kde/fhd 24" monitor right now . hope we can get some people to be guinea pigs and try it out. we will need to work on some polishing like the default size of markers and such as the people will have wide range of setups like 4k/hidpi screens, weird distros and so on. 

performance is definitely great as long as you reach the minimum requirements.

### the steps to complete jokolay are 8. 
[] get a very stable core renderer working. 
[*] rawinput from keyboard mouse while not in focus. //done
[*] get a very stable egui/user interface working. //mostly done. just need to do some theme stuff
[] start adding features like timer windows/kp lookup/switching markers. // need to start
[] Marker/trail recorder/editor // once we figure out whether to support undo, we can start with this ez
[] Notification system like Gw2Pao // delayed until we get the other features polished. 
[] Documentation/tutorials // delayed until jokolay is like 70% done ,so we can create github wiki with screenshots for easy tutorials. 
[] polish and collaborate with other projects for extension of markers format/modules like blish/customization and such.

The stability of the core renderer, egui and input are the most important components as the rest of the features will be based on those three components




## Minimum Requirements
### Linux
#### Requires OpenGL 4.6
you can check your opengl version with the following command
`glxinfo | grep "OpenGL version"`
which should return something like `OpenGL version string: 4.6.0 NVIDIA 470.63.01` . the `4.6.0` is what we need. most gpu after gtx 750 should have opengl 4.6.
#### Requires X11
Wayland is NOT supported. 
### Compiling
#### Requirements
1. install rust toolchain with rustup
2. on windows, install visual studio to get your c/c++ compiler/linker sdk.
3. install cmake 
4. on linux, install glfw. on windows, glfw sources will be auto-built from source using cmake.
5. on linux, you need xorg and gtk headers (used for native file dialogs). `xorg-dev libgtk-3-dev` are ubuntu package names.
6. now, you can build it using `cargo` like any other rust project. 

