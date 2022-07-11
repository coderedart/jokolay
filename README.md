# jokolay
An Overlay for Guild Wars 2 in Rust



1. Jokolink: This is what you will run from the same wine prefix as you are running gw2 from. it reads the shared memory (https://wiki.guildwars2.com/wiki/API:MumbleLink ) of gw2 to get player location/camera pos/which character is logged in  etc..  and ofcourse, window location. it can then copy all that stuff as is into a shared memory file under /dev/shm for linux native apps to use, so there can be multiple overlays or other addons requesting data.
2. jokoapi: yeah. just an api library to use in our overlay, but its fine for generic use too. the easiest part and its mostly just grunt work of copy pasting api endpoints and filling out all the required fields of structs etc.. if anyone wants to contribute, this is the best place and people with the any amount of programming can do this. the bottleneck here is literally copy/pasting and typing speed.
3. Jokolay: this is the actual overlay. its made up of multiple parts too.
   1. Overlay Window - The window which 
      1. stays on top of all other windows using always_on_top feature.  
      2. is trasparent by allowing a color depth of 32 bits instead of 24 bits for sRGBA8 surface.
      3. is able to be passthrough whenever we require it to using features like x11 shape or MS EX_Layered
   2. Renderer - A wgpu context using vulkan as backend. allows us to draw stuff, load textures into gpu etc..
   3. GUI - egui which enables us to draw Ui very easily, and supports theming to a reasonable extent.
   4. Scripting Engine - Lua allowing users to add their own scripts at runtime or make plugins for the Overlay like Blish's Module system
4. Joko Marker Format:
   1. A Json based Marker format because xml is too painful to deal with in rust. 
   2. Allows us to easily extend marker format as long as we can import/export marker packs used by blish/taco
   3. Make our own rules as to the format of the files, layout of the directory structure and other validations.
      
I will primarily be testing this on Endeavour (Arch) OS / KDE Plasma latest. need more guinea pigs to test things on other DEs. 

### the steps to complete jokolay are 8 
- [x] get a very stable core renderer working. especially texture manage ment. //done, we just use bevy now which will take care of rendering for us.
- [x] rawinput from keyboard mouse while not in focus. //done using device_query
- [x] get a very stable egui/user interface working. // done. using luaegui
- [ ] Documentation/tutorials // delayed until jokolay is like 70% done ,so we can create github wiki with screenshots for easy tutorials.
- [ ] start adding features like timer windows/kp lookup/switching markers. // need to start
- [ ] Marker/trail recorder/editor // once we figure out whether to support undo, we can start with this ez
- [ ] Notification system like Gw2Pao // delayed until we get the other features polished. 
- [ ] polish and collaborate with other projects for extension of markers format/modules like blish/customization and such.





## Minimum Requirements
### Linux
#### Requires Vulkan
most gpu after gtx 750 should be okay.
#### Requires X11
Wayland is NOT supported. 
### Compiling
TODO

