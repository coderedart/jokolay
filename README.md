# jokolay
An Overlay for Guild Wars 2 in Rust

​

I'm trying to make a taco port for linux (well, technically, it is crossplatform). i'm learning programming too, so it is going slow. it is made of 3 parts basically. and written in rust ❤️
1. Jokolink: This is what you will run from the same wine prefix as you are running gw2 from. it reads the shared memory (https://wiki.guildwars2.com/wiki/API:MumbleLink ) of gw2 to get player location/camera pos/which character is logged in  etc..  and ofcourse, window locattion. any client that wants to know this info can request it over a socket, so there can be multiple overlays or other addons requesting data. . this is really scary because of using windows api that is horrible. anyway, this is like 90% done and just needs some polishing  and stuff during this week.
2. jokoapi: yeah. just an api library to use in our overlay, but its fine for generic use too. the easiest part and its mostly just grunt work of copy pasting api endpoints and filling out all the required fields of structs etc.. if anyone wants to contribute, this is the best place and people with the any amount of programming can do this. the bottleneck here is literally copy/pasting and typing speed.
3. Jokolay: this is the actual overlay. it depends on 3 conditions. first is being transparent and only allow markers to be visible. second is it must allow the input like clicks to passthrough it(so, must not be focusable). third and final condition is that it must stay above gw2 during windowed fullscreen or windowed mode.
Jokolay, right now, uses a patched version of glfw3.4 and can do all of the above things.
Todo:
the present hurdle is learning about game engine development bcoz taco/jokolay are basically a first person game that take inputs from the gw2 sharedmemory to update the camera/player position instead of what most games do like wsad. the markers are just the objects or whatever you wanna call them in a transparent "map" or "world" so to speak. i need to learn all about that stuff and there's some hardcore math involved about matrices and stuff.
the ability to read the marker/trail format.
once we get the previous 2 things done with (i estimate maybe a month and a half at the earliest), the rest are pretty easy, like the gui to enable/disable markers or displaying timers, creating markers (i hope to make it as easy as possible so that lore/rp people can make mini-D&D games for others to play), and so on.
 



Been thinking of making a taco port forever on linux and when taco went opensource, i thought i can just change some code here and there to get it working on linux. yeah, i was wrong. in one of taco's blogposts, he mentions that he made it from scratch without using stdlib. that was true. that's already enough for me (a noobie) to accept defeat on working with that codebase. and the codebase being very windows specific means, i have no other choice but to rewrite. 
​ofcourse, i did get a rough idea of how much work it is to make an overlay because of skimming over the code. that's why i just chose using well established libraries to avoid having to write from scratch to reduce the burden. and make it crossplatform as its actually *less* work to use glfw3 than to deal with X11. 
