use base64::Engine;
use indexmap::IndexMap;
use joko_core::prelude::*;
use std::io::Write;
use xot::{Element, Node, SerializeOptions, Xot};

use crate::{
    pack::{Category, CommonAttributes, Marker, PackCore, Trail},
    BASE64_ENGINE,
};

use super::XotAttributeNameIDs;
/// Save the pack core as xml pack using the given directory as pack root path.
pub fn save_pack_core_to_dir(pack_core: &PackCore, dir: &Dir) -> Result<()> {
    {
        // save categories
        let mut tree = Xot::new();
        let names = XotAttributeNameIDs::register_with_xot(&mut tree);
        let od = tree.new_element(names.overlay_data);
        let root_node = tree
            .new_root(od)
            .into_diagnostic()
            .wrap_err("failed to create new root with overlay data node")?;
        recursive_cat_serializer(&mut tree, &names, &pack_core.categories, od)
            .wrap_err("failed to serialize cats")?;
        let cats = tree
            .with_serialize_options(SerializeOptions { pretty: true })
            .to_string(root_node)
            .into_diagnostic()
            .wrap_err("failed to convert cats xot to string")?;
        dir.create("categories.xml")
            .into_diagnostic()
            .wrap_err("failed to create categories.xml")?
            .write_all(cats.as_bytes())
            .into_diagnostic()
            .wrap_err("failed to write to categories.xml")?;
    }
    // save maps
    {
        for (map_id, map_data) in &pack_core.maps {
            let mut tree = Xot::new();
            let names = XotAttributeNameIDs::register_with_xot(&mut tree);
            let od = tree.new_element(names.overlay_data);
            let root_node: Node = tree
                .new_root(od)
                .into_diagnostic()
                .wrap_err("failed to create root wiht overlay data for pois")?;
            let pois = tree.new_element(names.pois);
            tree.append(od, pois)
                .into_diagnostic()
                .wrap_err("faild to append pois to od node")?;
            for marker in &map_data.markers {
                let poi = tree.new_element(names.poi);
                tree.append(pois, poi)
                    .into_diagnostic()
                    .wrap_err("failed to append poi (marker) to pois")?;
                let ele = tree.element_mut(poi).unwrap();
                serialize_marker_to_element(marker, ele, &names);
            }
            for trail in &map_data.trails {
                let trail_node = tree.new_element(names.trail);
                tree.append(pois, trail_node)
                    .into_diagnostic()
                    .wrap_err("failed to append a trail node to pois")?;
                let ele = tree.element_mut(trail_node).unwrap();
                serialize_trail_to_element(trail, ele, &names);
            }
            let map_xml = tree
                .with_serialize_options(SerializeOptions { pretty: true })
                .to_string(root_node)
                .into_diagnostic()
                .wrap_err("failed to serialize map data to string")?;
            dir.create(format!("{map_id}.xml"))
                .into_diagnostic()
                .wrap_err("failed to create map xml file")?
                .write_all(map_xml.as_bytes())
                .into_diagnostic()
                .wrap_err("failed to write map data to file")?;
        }
    }
    // save images
    {
        for (img_path, img) in &pack_core.textures {
            if let Some(parent) = img_path.parent() {
                dir.create_dir_all(parent.as_str())
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        miette::miette!("failed to create parent dir for an image: {img_path}")
                    })?;
            }
            dir.create(img_path.as_str())
                .into_diagnostic()
                .wrap_err_with(|| miette::miette!("failed to create file for image: {img_path}"))?
                .write(&img)
                .into_diagnostic()
                .wrap_err_with(|| {
                    miette::miette!("failed to write image bytes to file: {img_path}")
                })?;
        }
    }
    // save tbins
    {
        for (tbin_path, tbin) in &pack_core.tbins {
            if let Some(parent) = tbin_path.parent() {
                dir.create_dir_all(parent.as_str())
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        miette::miette!("failed to create parent dir of tbin: {tbin_path}")
                    })?;
            }
            let mut bytes: Vec<u8> = vec![];
            bytes.reserve(8 + tbin.nodes.len() * 12);
            bytes.extend_from_slice(&tbin.version.to_ne_bytes());
            bytes.extend_from_slice(&tbin.map_id.to_ne_bytes());
            for node in &tbin.nodes {
                bytes.extend_from_slice(&node[0].to_ne_bytes());
                bytes.extend_from_slice(&node[1].to_ne_bytes());
                bytes.extend_from_slice(&node[2].to_ne_bytes());
            }
            dir.create(tbin_path.as_str())
                .into_diagnostic()
                .wrap_err_with(|| miette::miette!("failed to create tbin file: {tbin_path}"))?
                .write_all(&bytes)
                .into_diagnostic()
                .wrap_err_with(|| miette::miette!("failed to write tbin to path: {tbin_path}"))?;
        }
    }
    Ok(())
}
fn recursive_cat_serializer(
    tree: &mut Xot,
    names: &XotAttributeNameIDs,
    cats: &IndexMap<String, Category>,
    parent: Node,
) -> Result<()> {
    for (cat_name, cat) in cats {
        let cat_node = tree.new_element(names.marker_category);
        tree.append(parent, cat_node).into_diagnostic()?;
        {
            let ele = tree.element_mut(cat_node).unwrap();
            ele.set_attribute(names.display_name, &cat.display_name);
            // let cat_name = tree.add_name(cat_name);
            ele.set_attribute(names.name, cat_name);
            // no point in serializing default values
            if !cat.default_enabled {
                ele.set_attribute(names.default_enabled, "0");
            }
            if cat.separator {
                ele.set_attribute(names.separator, "1");
            }
            serialize_common_attributes_to_element(&cat.props, ele, names);
        }
        recursive_cat_serializer(tree, names, &cat.children, cat_node)?;
    }
    Ok(())
}
fn serialize_trail_to_element(trail: &Trail, ele: &mut Element, names: &XotAttributeNameIDs) {
    ele.set_attribute(names.guid, BASE64_ENGINE.encode(trail.guid));
    ele.set_attribute(names.category, &trail.category);
    serialize_common_attributes_to_element(&trail.props, ele, &names);
}

fn serialize_marker_to_element(marker: &Marker, ele: &mut Element, names: &XotAttributeNameIDs) {
    ele.set_attribute(names.xpos, format!("{}", marker.position[0]));
    ele.set_attribute(names.ypos, format!("{}", marker.position[1]));
    ele.set_attribute(names.zpos, format!("{}", marker.position[2]));
    ele.set_attribute(names.guid, BASE64_ENGINE.encode(marker.guid));
    ele.set_attribute(names.map_id, format!("{}", marker.map_id));
    ele.set_attribute(names.category, &marker.category);
    serialize_common_attributes_to_element(&marker.props, ele, names);
}

fn serialize_common_attributes_to_element(
    ca: &CommonAttributes,
    ele: &mut Element,
    names: &XotAttributeNameIDs,
) {
    if let Some(icon_file) = &ca.icon_file {
        ele.set_attribute(names.icon_file, icon_file.as_str());
    }
    if let Some(texture) = &ca.texture {
        ele.set_attribute(names.texture, texture.as_str());
    }
    if let Some(trail_data) = &ca.trail_data_file {
        ele.set_attribute(names.trail_data, trail_data.as_str());
    }
}
