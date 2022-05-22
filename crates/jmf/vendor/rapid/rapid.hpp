#pragma once
#include "jmf/src/lib.rs.h"
#include "rust/cxx.h"

namespace rapid {
    rust::String rapid_filter(rust::String src_xml);
}