#pragma once
#include "jmf/src/xmlpack/rapid.rs.h"
#include "rust/cxx.h"

namespace rapid {
    rust::String rapid_filter(rust::String src_xml);
}