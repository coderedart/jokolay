mod app;

pub use app::start_jokolay;

/*
Linux:
When gw2 is set to 1000 points width. on a 4k monitor with 192 (2.0) scale
menu bar lengths (based on UI size) are (in points according to jokolay which respects dpi scaling)
small  -> 253
normal -> 280.5
large  -> 312.5
larger -> 344.0

when gw2 is set to 1000 points width on 4k monitor with no dpi scaling
small  -> 143.5
normal -> 159.0
large  -> 177.0
larger -> 195.0

Windows:
gw2 1000 points width. 4k monitor with 2.0 scale
small  -> 266.0
normal -> 295.0
large  -> 328
larger -> 359.5

with no dpi scaling
small  -> 148
normal -> 163.5
large  -> 181.5
larger -> 199.5

Windows: measure again, with details about dpi awareness and using client rect to properly deal with viewport values *only* in width/height
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

same setup as above, but with 500 points (1000 pixels width)
small  -> 140.5
normal -> 156
large  -> 173.5
larger -> 191

clearly, they follow a certain "ratio" as to how much space the menu icons want to occupy.
The only question is how to know if dpi scaling is enabled or not.
*/
