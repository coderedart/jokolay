//! Guild Wars 2 has an array of menu icons on top left corner of the game.
//! Its size is affected by four different factors
//! 1. UISZ:
//!     This is a setting in graphics options of gw2 and it comes in 4 variants
//!     small, normal, large and larger.
//!     This is something we can get from mumblelink's context.
//! 2. Dimensions of the gw2 window
//!     This is something we get from mumble link and win32 api. We store this as client pos/size in mumble link
//!     It is not just the width or height, but their ratio (fov) which seems to change the size of menu icons
//! 3. DPI scaling
//!     This is a setting in graphics options too. When scaling is enabled, sizes of menu become bigger (if scaling is > 1.0)
//!     This is something we get from gw2's config file in AppData/Roaming and store in mumble link as dpi scaling

/*
Linux ->
width 1920 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 2.0. dpi enabled
small  -> 270       26.5
normal -> 299.5     29.5
large  -> 333.5     32.5
larger -> 367.0     36

same but with dpi disabled
small  -> 144       13.5
normal -> 159.5     15.5
large  -> 177.5     17
larger -> 195.5     19
*/
/*
Linux:
When gw2 is set to 1000 points width. on a 4k monitor with 192 (2.0) scale
menu bar lengths (based on UI size) and heights are (in points according to jokolay which respects dpi scaling)
small  -> 253       24.5
normal -> 280.5     27.5
large  -> 312.5     30.5
larger -> 344.0     33.5

when gw2 is set to 1000 points width on 4k monitor with no dpi scaling
small  -> 144
normal -> 159.5
large  -> 177.5
larger -> 195.5


Windows:
gw2 1000 points (2000 pixels) width. 4k monitor with 192 dpi (2.0 scale) and dpi scaling enabled in settings
small  -> 281
normal -> 312
large  -> 347
larger -> 382

gw2 with dpi scaling disabled
small  -> 144
normal -> 159.5
large  -> 177.5
larger -> 195.

same setup as above, but with 500 points (1000 pixels width) and dpi scaling enabled
small  -> 140.5
normal -> 156
large  -> 173.5
larger -> 191

clearly, they follow a certain "ratio" as to how much space the menu icons want to occupy.
The only question is how to know if dpi scaling is enabled or not.
*/
mod linux {}
