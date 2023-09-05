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
all units are in pixels.
conclusions:
1. total width // 10 = width of each icon. There are some extra pixels between some icons which add a few extra pixels
2. width of each icon = height of each icon. so they are technically squares
3. scaling beyond 2.0 doesn't changed the sizes of icons (atleast at 4k resolutin)
4.
dpi width / non-dpi width = 1.877-ish. so, instead of 2.0, they are increased by this scale.
dpi height / non-dpi height = 1.9-ish.
the 1.9 or 1.8 could be influenced by the width/height ratio which is 0.9 (which gives 1.8 when multiplying scale 2.0)
Blish assumes larger as the 1.0 and measure the rest of the UIsizes as the ratio against larger
https://github.com/blish-hud/Blish-HUD/blob/dev/Blish%20HUD/GameServices/GraphicsService.cs#L130
small / large = 0.81
normal / large = 0.89
large / large = 1.0
larger / large = 1.1
dpi or non-dpi. This ratio seems consistent (within a precision of 0.02).
this ratio applies to both widths AND heights.

The first column is the width of the menu row (10 icons in total). and second row is the height of the row.

with dpi disabled, values were same regardless of scale/platform
small  -> 288     27
normal -> 319     31
large  -> 355     34
larger -> 391     38

with dpi enabled, there's some math involved it seems.
Linux ->
width 1920 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 2.0. dpi enabled
small  -> 540     53
normal -> 599     59
large  -> 667     65
larger -> 734     72


Windows ->
width 1920 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 2.0. dpi enabled.
small  -> 540     53
normal -> 599     59
large  -> 667     65
larger -> 734     72

width 1914 pixels. height 2072 pixels. ratio 0.92. fov 1.01. scaling 3.0. dpi enabled. dpi 288
small  -> 538     52
normal -> 598     58
large  -> 665     65
larger -> 731     72

width 3840. height 2160. ratio 1.78. scaling 3. dpi true. dpi 288 (windowed fullscreen)
small  -> 810     80
normal -> 900     89
large  -> 1000    99
larger -> 1100    109

width 1916 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 1.5. dpi enabled. dpi 144
small  -> 432     42
normal -> 480     47
large  -> 533     52
larger -> 586     57

width 1000 pixels. height 1000 pixels. ratio 1. fov 1.01. scaling 2.0. dpi enabled.
small  -> 281     26
normal -> 312     29
large  -> 347     33
larger -> 382     36

width 2000 pixels. height 1000 pixels. ratio 2. fov 1.01. scaling 2.0. dpi enabled.
small  -> 375     36
normal -> 416     40
large  -> 463     45
larger -> 509     49

width 2000 pixels. height 2000 pixels. ratio 1. fov 1.01. scaling 2.0. dpi enabled.
small  -> 562     55
normal -> 624     61
large  -> 694     68
larger -> 764     75


*/
mod linux {}
