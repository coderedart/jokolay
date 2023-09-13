
## Status
still in early stages of development




### RapidXML Integration
Taco uses RapidXML, which is very very lenient in its parsing. 
this led to marker packs not caring about their xml being valid xml.
Blish instead created a custom parsing library to deal with this and have workarounds for known issues. 

rapidxml does fix these issues itself when we roundtrip xml through it. so, we have a function called `rapid_filter` which takes in xml string and returns a "filtered" xml string that fixes a bunch of issues like escaping special characters like
ampersand, gt, lt etc.. with proper xml formatting i.e `&amp;`, `&gt;` etc..

Sources of rapidxml are in the vendor folder. it is a custom fork from https://github.com/timniederhausen/rapidxml which
added some fixes / enhancements. its stil a mess with compiler warnings, but whatever.

we use cxxbridge crate. 
`rapid.hpp` is our header with declaration for `rapid_filter` inside `rapid` namespace. (includes `joko_marker_format/src/lib.rs.h`)
`lib.rs` has extern declaration which has the same signature but in rust. (includes `joko_marker_format/vendor/rapid/rapid.hpp`)
`build.rs` has the compilation instructions. it uses `lib.rs` extern declaration, `rapid.cpp` as compilation unit as it
    contains the definition of `rapid_filter` and finally outputs a `librapid.a` for linking.

with this, we now filter the xml with `rapid_filter` before deserializing it in rust. if we still have errors we just 
complain about it. 



### XML Marker Format
Marker Pack

1. Textures
	1. identified by the relative path. case sensitive. But to accommodate case-insensitive MS windows packs, we will convert all paths to lowercase when importing.
	2. png format.
	3. need to convert to a srgba texture and upload to gpu to use it
	4. mostly tiny images. here's the composition of tekkit's pack textures

| count | dimensions    |
|-------|---------------|
| 630   |   100x100     |
| 7     |   150x150     |
| 89    |   200x200     |
| 683   |   250x250     |
| 42    |   256x256     |
| 435   |   500x500     |

2. Tbins
	1. binary data of a series of vec3 positions. + mapid + a version (just ver 2 for now) 
	2. need to generate a mesh to be usable to upload on gpu. different mesh for 2d map / minimap. trail_scale an affect width of the generated mesh
	3. anim_speed attr needs dynamic texture coords (probably based on time delta offset)
	4. color attribute requires blending.
	5. uses texture
	6. can be statically or dynamically filtered (culled). but no cooldowns. 
   
3. MarkerCategories
   1. create a tree structure of menu to be displayed. 
   2. identified by their name (and parents in the hierarchy) as a unique path.
   3. can be enabled or disabled. need to persist this data in activation data or somewhere else.
   4. enabled / disabled categories act as dynamic filters for markers / trails.
   5. attributes get inherited by children unless overrided. and also inehrited by the markers / trails. 
   6. can be enabled / disabled by a marker action (toggle_category attribute)
4. Markers
   1. render a quad. either billbaord or static rotation.
   2. needs texture + alpha attribute + color attribute for blending. 
   3. alpha is also affected by fadenear and fadefar attributes. 
   4. static filters like ingamevisibility or map visibility or minimap visibility. 
   5. can display text via info / tip-description.
   6. dynamic filters like behavior + race + profession + specialization + mount + map type + category + festival + achievement.
   7. size is determined by texture + minSize / maxSize + scale. map quad rendering affected by scale on map and mapdisplaysize attribute
   8. triggers actions of behavior + copy-message (copy clipboard) + bounce?? + toggling category based on player proximity and pressing of a special action key (usually F)
5. Trails
   1. render the tbin mesh.
   2. same filters as marker
   3. no triggering / activation / cooldowns though. 


3D:
1. can match blish
2. need to ignore certain attributes like minSize and maxSize. 


2D:
1. can match taco
2. more performance because 2d?


