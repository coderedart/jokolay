/*
maintain a list of active markers which
-> belong to current map AND
-> belong to enabled cats AND
-> not triggered yet

get the textures used by current active markers. load them into gpu (bindgroups) and active markers will keep a Arc or something to keep this texture alive

active markers will also store a default mesh (south facing? rectangle) using the texture's width + height.

every frame, we take the mesh + try culling it + store them all into buffers and draw them.

*/
