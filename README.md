# jokolay
An Overlay for Guild Wars 2 in Rust

Well, technically, this contains a family of crates related to jokolay.

1. `jokolink`: This is what you will run from the wine prefix of gw2 . it reads the *official* [shared memory](https://wiki.guildwars2.com/wiki/API:MumbleLink) of gw2 to get live player data and copy into a shared memory file under /dev/shm for linux native apps (like Jokolay) to use.
2. `jokoapi`: API bindings for gw2 api in rust. if anyone wants to contribute, this is the best place. its just copy pasting api endpoints and filling out all the required fields of structs, writing tests to verify.
3. `jokolay`: this is the actual overlay.    
4. `joko_marker_format`: deals with marker packs.
      
## Minimum Requirements
1. Requires Vulkan. most GPUs after gtx 750 should be okay.
2. X11. Wayland is NOT supported. (does not apply to windows obviously).

### Compiling
for now, just look at the github workflow file.
- dependencies: `cmake`, 
- linux deps: gtk, xorg, 

### Window Managers
- compositing must be turned on or transparency won't work. you will just see a black window other wise. 
- lutris can disable compositing when game launches, so turn that feature off in lutris game options. 

#### Officially supported
1. KDE.

I will primarily be testing Jokolay on Endeavour (Arch) OS / KDE Plasma latest. need more guinea pigs to test things on other DEs. 


