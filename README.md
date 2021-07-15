# jokolay
An Overlay for Guild Wars 2 in Rust

​

1. Jokolink: This is what you will run from the same wine prefix as you are running gw2 from. it reads the shared memory (https://wiki.guildwars2.com/wiki/API:MumbleLink ) of gw2 to get player location/camera pos/which character is logged in  etc..  and ofcourse, window location. any client that wants to know this info can request it over a socket, so there can be multiple overlays or other addons requesting data. right now, we just expose the socket api because async is only supported in wine 6.11+. then, we can use async socket or grpc which can be used by even browser apps. 

2. jokoapi: yeah. just an api library to use in our overlay, but its fine for generic use too. the easiest part and its mostly just grunt work of copy pasting api endpoints and filling out all the required fields of structs etc.. if anyone wants to contribute, this is the best place and people with the any amount of programming can do this. the bottleneck here is literally copy/pasting and typing speed.
3. Jokolay: this is the actual overlay. pretty much most of the barebones stuff is done. all we need for now is to organize the categories properly. look at taco/blish implementations and try to copy them.

  i am doing this release primarily to see if it is working for everyone. i am running manjaro/kde/fhd 24" monitor right now and i will test it on fedora gnome too before release. hope we can get some people to be guinea pigs and try it out. we will need to work on some polishing like the default size of markers and such as the people will have wide range of setups like 4k/hidpi screens, weird distros and so on. 

on my 1070ti, i get around 1k fps with a few markers, but there's a bunch of optmizations that i plan to do in future to improve it.

### the steps to complete jokolay are 8. 
[] get a very stable core renderer working. 
[] rawinput from keyboard mouse while not in focus.
[] get a very stable egui/user interface working.
[] start adding features like timer windows/kp lookup/switching markers.
[] Marker/trail recorder/editor
[] Notification system like Gw2Pao
[] Documentation/tutorials
[] polish and collaborate with other projects for extension of markers format/modules like blish/customization and such.

The stability of the core renderer, egui and input are the most important components as the rest of the features will be based on those three components