# Configuration
1. jokolay_directory
2. log levels
3. input configuration: scroll power, shortcuts.
4. egui: theme (style + fonts priority)

# Initialization

### Logging
1. first get the configuration for logging. the log directory and log levels.
2. initialize logging and make sure that the guard is stored at the top level, so that it catches/flushes all logs at panics
3. log a bunch of CARGO_ENV vars like app version, git commit etc..

### Wgpu
1. create instance and log its details like the adapters listed
2. create adapter and log its supported formats/features. always use vulkan
3. create a texture manager to hold on to textures, and things like linear samplers/bindgroup layouts
4. package all of them into a wgpu context so that it can be stored in multiple places.

### Overlay Windows
1. init glfw
2. set hints like transparent + passthrough + decorations false + resizeable false + NoApi vulkan 
3. create a window on primary_monitor with its full `work_area` 
4. create a surface




# LifeCycle
### Overlay Windows
1. 
NOTE: by making the resizeable = false, we also gain a advantage, as the surface is never outdated / suboptimal



TODO:
shaders load/compile
marker category/POIs
get map from jokolink
load POIs and setup buffers when loading into a new map
get camera/fvars from link
draw to screen

The egui Ctx:
1. depends on Overlay Window for gathering events
    1. scroll_delta: user scroll
    2. zoom_delta: from ctrl-scroll or pinch gesture
    3. screen_rect: window resize event (frame buffer size)
    4. pixels_per_point: desktop hidpi resolution/scale change
    5. time: time right now to calculate animations etc..
    6. predicted_dt: vsync speeds. not necessary
    7. modifiers: depends on key events
    8. events: Vec<Event>
        1. Copy: "ctrl + c" i guess. need to check for c and check if modifers has ctrl only without any other modifiers like shift/alt/super
        2. Cut: "ctrl + x" same as above
        3. Text(String): egui doesn't input text based on keys, we have to send this event by making strings
        4. Key
            1. key: Key // alphabets + numbers + ArrowDown,ArrowLeftArrowRight,ArrowUp,Escape,Tab,Backspace,Enter,Space,Insert,Delete,Home,End,PageUp,PageDown
            2. pressed: bool // release or press
            3. modifiers: Modifiers}
        5. PointerMoved(Pos2) // only if cursor changed position
        6. PointerButton {
            1. pos: Pos2 // cursor position at event
            2. button: PointerButton // only recognizes three - Primary, Secondary, Middle,
            3. pressed: bool // click or release
            4. modifiers: }
        7. PointerGone when cursor left window or if we lose focus or if we go passthrough mode
    9.  hovered_files: could be used for marker pack files or images import
    10. dropped_files: same as above
2. decides whether we go passthrough mode or not
3. and ofcourse depends on renderer to draw itself.
    1. needs clipping
    2. needs blending
    3. needs index buffer for drawing
    4. needs special handling of colors because srgb and premultiplied alpha
4. texture management

The Renderer:
1. depends on window for events like resizing to make changes to viewport sizes
2. needs access to buffers of egui/markers/trails to draw them.


# Limitations of the design
### Scaling
Because of our obsession that jokolay must scale naturally with the number of gw2 instances, the core windowing part is a little complicated
But, if we just care about supporting single user case, then its pretty easy and we can abstract that away. either for us or the addons.

### Lua / Scripting
multi-threading in Lua is not as smooth as we could have made it. but addons can use native dylibs to do that kind of off thread stuff.

### Window Managers
This one will be obvious. but we cannot test everything in all window managers, so officially, we only support KDE which is what i use.
One thing to warn people about is that, compositing must be turned on or transparency won't work.


