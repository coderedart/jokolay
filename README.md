# jokolay
An Overlay for Guild Wars 2 in Rust

​STATUS: there is another overlay for linux in development. its not open source yet. but if the author releases it, and its in rust <3 and if i think its features (even planned ones) are better than jokolay, i will gladly embrace it. otherwise, i will continue to develop this.

1. Jokolink: This is what you will run from the same wine prefix as you are running gw2 from. it reads the shared memory (https://wiki.guildwars2.com/wiki/API:MumbleLink ) of gw2 to get player location/camera pos/which character is logged in  etc..  and ofcourse, window location. it can then copy all that stuff as is into a shared memory file under /dev/shm for linux native apps to use, so there can be multiple overlays or other addons requesting data. 

2. jokoapi: yeah. just an api library to use in our overlay, but its fine for generic use too. the easiest part and its mostly just grunt work of copy pasting api endpoints and filling out all the required fields of structs etc.. if anyone wants to contribute, this is the best place and people with the any amount of programming can do this. the bottleneck here is literally copy/pasting and typing speed.
3. Jokolay: this is the actual overlay.there's a rewrite going on for the following reasons.


1. read the marker pack into an internal representation to support editing them live as well as use proper UUID for performance and other reasons. 
2. make the structs themselves represent state for egui rather than separate window structs.
3. seperate the rendering parts and core logic to allow future upgrade to wgpu-rs / vulkano. 
4. try to use the same texture manager for both egui and markers/trails.
5. simplify the rendering to use only the simplest features and also allow for future performance upgrades if users hardware supports it. eg: SSBO, GS, persistent mapped buffers (dynamic resizing) and so on.

 i am running manjaro/kde/fhd 24" monitor right now . hope we can get some people to be guinea pigs and try it out. we will need to work on some polishing like the default size of markers and such as the people will have wide range of setups like 4k/hidpi screens, weird distros and so on. 

performance is decent (i think, need to check the cpu/gpu usages of blish/taco), but trails and other features like timer window need to be done before we can start talking about performance. 

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

as for why develop a new overlay instead of porting taco/blish. 
taco is too low level C (pretending to be C++ ). doesn't use stdlib, doesn't use cmake, merged with windows/directx code, abondoned for the forseeable future as taco dev is nowhere to be seen :( to merge PRs and pretty much zero documentation. too hard for me.
blish is great otoh. little documentation, blish-hud discord will always be there to help, so its not a problem. there's a lot of momentum especially with modules and its surrounding community. but idk C# (and i don't want to learn a microsoft language honestly if i can afford to), C# on linux doesn't feel like a first class language and C# itself is not suitable imo for addons/modules/plugins kinda system. compare that to something like javascript/lua/python which feel much more natural on linux and absolutely suitable for writing quick/easy addons. 
and finally, i am not a programmer. i'm learning programming as a hobby. so, this project will help me learn programming. rust seems great with very easy setup stuff like crosscompilation/cargo/dependencies and the safety would save me a lot of time spent debugging seg faults. by using this, i can get speed/performance like taco, but with easier documentation/development setup/more safety in general. Although i do agree that the libraries in c++ would have made jokolay come to life a LOT LOT sooner. just using a renderer like bgfx would have taken care of a lot of stuff for me. anyway, once this is reasonably finished (might take a month or more for editing markers and trails features), i plan to look at multithreading, and finally scripting. 

