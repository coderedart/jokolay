
## Status
still in early stages of development

## Online
Marker Packs should be saved, loaded and edited offline. but for convenience, we will provide a method to download 
the packs from internet within jokolay itself. 

## Validation

### RapidXML Integration
Taco uses RapidXML, which is very very lenient in its parsing. 
this led to marker packs not caring about their xml being valid xml.
Blish instead created a custom parsing library to deal with this and have workarounds for known issues. 

rapidxml does fix these issues itself when we export it. so, we have a function called `rapid_filter` which takes in
xml string and returns a "filtered" xml string that fixes a bunch of issues like escaping special characters like
ampersand, gt, lt etc.. with proper xml formatting i.e `&amp;`, `&gt;` etc..

sources of rapidxml in the vendor folder. it is a custom fork from https://github.com/timniederhausen/rapidxml which
has some more cleanup compared to the original rapidxml library. its stil a mess with compiler warnings, but its an
improvement.

we use cxxbridge crate. 
`rapid.hpp` is our header with declaration for `rapid_filter` inside `rapid` namespace. (includes `jmf/src/lib.rs.h`)
`lib.rs` has extern declaration which has the same signature but in rust. (includes `jmf/vendor/rapid/rapid.hpp`)
`build.rs` has the compilation instructions. it uses `lib.rs` extern declaration, `rapid.cpp` as compilation unit as it
    contains the definition of `rapid_filter` and finally outputs a `librapid.a` for linking.

with this, we now filter the xml with `rapid_filter` before deserializing it in rust. if we still have errors we just 
complain about it. 

### Json Schema
It is very important that the marker packs have a json schema to validate against. this will catch a lot of semantic 
errors before we even start deserializing. this will also be very useful to validate *without* jokolay running.
eg: CI/CD or local IDE environment when editing

