#include "jmf/rapid/rapid.hpp"
#include "jmf/rapid/rapidxml.hpp"
#include "jmf/rapid/rapidxml_print.hpp"
#include "jmf/src/xmlpack/rapid.rs.h"
#include <string>
#include <sstream>
#include <string>
#include <set>
void remove_duplicate_nodes(rapidxml::xml_node<char> *node)
{

    std::set<std::string> duplicates;
    rapidxml::xml_attribute<char> *attr = node->first_attribute();
    while (attr)
    {
        std::string name(attr->name(), attr->name_size());
        if (duplicates.count(name) == 1)
        {
            rapidxml::xml_attribute<char> *prev = attr;
            attr = attr->next_attribute();
            node->remove_attribute(prev);
        }
        else
        {
            duplicates.insert(name);
            attr = attr->next_attribute();
        }
    }
    for (rapidxml::xml_node<char> *child = node->first_node(); child; child = child->next_sibling())
    {
        remove_duplicate_nodes(child);
    }
}

namespace rapidwrap
{

    rust::String rapid_filter(rust::String src_xml)
    {
        // return std::string(src_xml);
        std::string src = static_cast<std::string>(src_xml);
        std::string dst;
        using namespace rapidxml;
        // create document
        xml_document<char> doc;
        // rapid xml throws exception if there's a parsing error
        try
        {
            // parse the xml text. if there's exceptions we go to catch block from here
            doc.parse<0>((char *)src.c_str());
            // delete all the duplicate attributes, so that there's no obvious errors for rust deserializers
            for (rapidxml::xml_node<char> *child = doc.first_node(); child; child = child->next_sibling())
            {
                remove_duplicate_nodes(child);
            }
            std::ostringstream oss;
            oss << doc;
            dst = oss.str();
        }
        catch (const parse_error &e)
        {
            return "";
        }
        return dst;
    }
}
