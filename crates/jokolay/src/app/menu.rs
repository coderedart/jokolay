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
