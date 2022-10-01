#pragma once
#include "jokolay/src/jmf/mod.rs.h"
#include "rust/cxx.h"

namespace rapid {
    rust::String rapid_filter(rust::String src_xml);
}