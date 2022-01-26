An Overlay Window that handles position/size/transparent/focus/alwaysontop attributes and collects events from glfw window.  
A Scene Object manages vulkan state for its scene. we have a egui, marker and trail scenes probably. 

An App class that holds all of these and runs an event loop. 



shaders load/compile
marker category/POIs
get map from jokolink
load POIs and setup buffers when loading into a new map
get camera/fvars from link
draw to screen

The Overlay Window:
provides events/event_loop. also sends the relevant changes like resize to vulkan/swapchain or egui.

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
