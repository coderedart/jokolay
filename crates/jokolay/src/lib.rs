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
*/
