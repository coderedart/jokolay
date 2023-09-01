use crate::{
    pack::{Category, CommonAttributes, Marker, PackCore, RelativePath, TBin, Trail},
    BASE64_ENGINE,
};
use base64::Engine;
use cap_std::fs_utf8::Dir;
use glam::Vec3A;
use indexmap::IndexMap;
use miette::{bail, Context, IntoDiagnostic, Result};
use std::{collections::BTreeMap, io::Read};
use tracing::{info, info_span, instrument, warn};
use uuid::Uuid;
use xot::{Element, Node, Xot};

use super::XotAttributeNameIDs;

pub fn load_pack_core_from_dir(dir: &Dir) -> Result<PackCore> {
    let mut pack = PackCore::default();
    // walks the directory and loads all files into the hashmap
    recursive_walk_dir_and_read_images_and_tbins(
        dir,
        &mut pack.textures,
        &mut pack.tbins,
        &RelativePath::default(),
    )
    .wrap_err("failed to walk dir when loading a markerpack")?;

    // parse map data of the pack
    for entry in dir
        .entries()
        .into_diagnostic()
        .wrap_err("failed to read entries of pack dir")?
    {
        let entry = entry
            .into_diagnostic()
            .wrap_err("entry error whiel reading xml files")?;
        let name = entry
            .file_name()
            .map_err(|e| miette::miette!("file name is not utf-8: {:?} {e:#?}", entry.file_name()))?
            .to_string();

        if name.ends_with("xml") {
            if let Some(name) = name.strip_suffix(".xml") {
                match name {
                    "categories" => {
                        // parse categories
                        {
                            let cats_xml = dir
                                .read_to_string("categories.xml")
                                .into_diagnostic()
                                .wrap_err("failed to read categories.xml")?;
                            parse_categories_file(&cats_xml, &mut pack)
                                .wrap_err("failed to parse category file")?;
                        }
                    }
                    map_id => {
                        // parse map file
                        if let Ok(map_id) = map_id.parse() {
                            let mut xml_str = String::new();
                            entry
                                .open()
                                .into_diagnostic()
                                .wrap_err("failed to open xml file")?
                                .read_to_string(&mut xml_str)
                                .into_diagnostic()
                                .wrap_err("faield to read xml string")?;
                            parse_map_file(map_id, &xml_str, &mut pack).wrap_err_with(|| {
                                miette::miette!("error parsing map file: {map_id}")
                            })?;
                        } else {
                            info!("unrecognized xml file {map_id}")
                        }
                    }
                }
            }
        }
    }
    Ok(pack)
}
fn recursive_walk_dir_and_read_images_and_tbins(
    dir: &Dir,
    images: &mut BTreeMap<RelativePath, Vec<u8>>,
    tbins: &mut BTreeMap<RelativePath, TBin>,
    parent_path: &RelativePath,
) -> Result<()> {
    for entry in dir
        .entries()
        .into_diagnostic()
        .wrap_err("failed to get directory entries")?
    {
        let entry = entry
            .into_diagnostic()
            .wrap_err("dir entry error when iterating dir entries")?;
        let name = entry.file_name().into_diagnostic()?;
        let path = parent_path.join_str(&name);

        if entry
            .file_type()
            .into_diagnostic()
            .wrap_err("failed to get file type")?
            .is_file()
        {
            if path.ends_with("png") || path.ends_with("trl") {
                let mut bytes = vec![];
                entry
                    .open()
                    .into_diagnostic()
                    .wrap_err("failed to open file")?
                    .read_to_end(&mut bytes)
                    .into_diagnostic()
                    .wrap_err("failed to read file contents")?;
                if name.ends_with("png") {
                    images.insert(path, bytes);
                } else if name.ends_with("trl") {
                    if let Some(tbin) = parse_tbin_from_slice(&bytes) {
                        tbins.insert(path, tbin);
                    } else {
                        info!("invalid tbin: {path}");
                    }
                }
            }
        } else {
            recursive_walk_dir_and_read_images_and_tbins(
                &entry.open_dir().into_diagnostic()?,
                images,
                tbins,
                &path,
            )?;
        }
    }
    Ok(())
}
fn parse_tbin_from_slice(bytes: &[u8]) -> Option<TBin> {
    let content_length = bytes.len();
    // content_length must be atleast 8 to contain version + map_id
    if content_length < 8 {
        info!("failed to parse tbin because the len is less than 8");
        return None;
    }

    let mut version_bytes = [0_u8; 4];
    version_bytes.copy_from_slice(&bytes[4..8]);
    let version = u32::from_ne_bytes(version_bytes);
    let mut map_id_bytes = [0_u8; 4];
    map_id_bytes.copy_from_slice(&bytes[4..8]);
    let map_id = u32::from_ne_bytes(map_id_bytes);

    // this will either be empty vec or series of vec3s.
    let nodes: Vec<Vec3A> = bytes[8..]
        .chunks_exact(12)
        .map(|float_bytes| {
            // make [f32 ;3] out of those 12 bytes
            let arr = [
                f32::from_le_bytes([
                    // first float
                    float_bytes[0],
                    float_bytes[1],
                    float_bytes[2],
                    float_bytes[3],
                ]),
                f32::from_le_bytes([
                    // second float
                    float_bytes[4],
                    float_bytes[5],
                    float_bytes[6],
                    float_bytes[7],
                ]),
                f32::from_le_bytes([
                    // third float
                    float_bytes[8],
                    float_bytes[9],
                    float_bytes[10],
                    float_bytes[11],
                ]),
            ];

            Vec3A::from_array(arr)
        })
        .collect();
    Some(TBin {
        map_id,
        version,
        nodes,
    })
}
// a recursive function to parse the marker category tree.
fn recursive_marker_category_parser(
    tree: &Xot,
    tags: impl Iterator<Item = Node>,
    cats: &mut IndexMap<String, Category>,
    names: &XotAttributeNameIDs,
) {
    for tag in tags.filter(|node| tree.is_element(*node)) {
        let ele = tree.element(tag).unwrap();
        if ele.name() != names.marker_category {
            continue;
        }

        let name = ele.get_attribute(names.name).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let mut ca = CommonAttributes::default();
        update_common_attributes_from_element(&mut ca, ele, names);

        let display_name = ele.get_attribute(names.display_name).unwrap_or_default();

        let separator = ele
            .get_attribute(names.separator)
            .unwrap_or_default()
            .parse()
            .map(|u: u8| if u == 0 { false } else { true })
            .unwrap_or_default();

        let default_enabled = ele
            .get_attribute(names.default_enabled)
            .unwrap_or_default()
            .parse()
            .map(|u: u8| if u == 0 { false } else { true })
            .unwrap_or(true);
        recursive_marker_category_parser(
            tree,
            tree.children(tag),
            &mut cats
                .entry(name.to_string())
                .or_insert_with(|| Category {
                    display_name: display_name.to_string(),
                    separator,
                    default_enabled,
                    props: ca.into(),
                    children: Default::default(),
                })
                .children,
            names,
        );
    }
}
fn update_common_attributes_from_element(
    ca: &mut CommonAttributes,
    ele: &Element,
    names: &XotAttributeNameIDs,
) {
    if let Some(path) = ele.get_attribute(names.icon_file) {
        ca.icon_file = Some(RelativePath::parse_from_str(path));
    }
    if let Some(path) = ele.get_attribute(names.texture) {
        ca.texture = Some(RelativePath::parse_from_str(path));
    }
    if let Some(path) = ele.get_attribute(names.trail_data) {
        ca.trail_data_file = Some(RelativePath::parse_from_str(path));
    }
}
fn parse_categories_file(cats_xml_str: &str, pack: &mut PackCore) -> Result<()> {
    let mut tree = xot::Xot::new();
    let xot_names = XotAttributeNameIDs::register_with_xot(&mut tree);
    let root_node = tree
        .parse(&cats_xml_str)
        .into_diagnostic()
        .wrap_err("invalid xml")?;

    let overlay_data_node = tree
        .document_element(root_node)
        .into_diagnostic()
        .wrap_err("no doc element")?;

    if let Some(od) = tree.element(overlay_data_node) {
        if od.name() == xot_names.overlay_data {
            recursive_marker_category_parser_categories_xml(
                &tree,
                tree.children(overlay_data_node),
                &mut pack.categories,
                &xot_names,
            );
        } else {
            bail!("root tag is not OverlayData")
        }
    } else {
        bail!("doc element is not element???");
    }
    Ok(())
}
fn parse_map_file(map_id: u32, map_xml_str: &str, pack: &mut PackCore) -> Result<()> {
    let mut tree = Xot::new();
    let root_node = tree
        .parse(map_xml_str)
        .into_diagnostic()
        .wrap_err("invalid xml")?;
    let names = XotAttributeNameIDs::register_with_xot(&mut tree);
    let overlay_data_node = tree
        .document_element(root_node)
        .into_diagnostic()
        .wrap_err("missing doc element")?;

    let overlay_data_element = tree
        .element(overlay_data_node)
        .ok_or_else(|| miette::miette!("no doc ele"))?;

    if overlay_data_element.name() != names.overlay_data {
        bail!("root tag is not OverlayData");
    }
    let pois = tree
        .children(overlay_data_node)
        .find(|node| match tree.element(*node) {
            Some(ele) => ele.name() == names.pois,
            None => false,
        })
        .ok_or_else(|| miette::miette!("missing pois node"))?;
    for child in tree.children(pois) {
        if let Some(child) = tree.element(child) {
            let category = child
                .get_attribute(names.category)
                .unwrap_or_default()
                .to_lowercase();

            let guid = child
                .get_attribute(names.guid)
                .and_then(|guid| {
                    let mut buffer = [0u8; 20];
                    BASE64_ENGINE
                        .decode_slice(guid, &mut buffer)
                        .ok()
                        .and_then(|_| Uuid::from_slice(&buffer[..16]).ok())
                })
                .ok_or_else(|| miette::miette!("invalid guid"))?;
            if child.name() == names.poi {
                if child
                    .get_attribute(names.map_id)
                    .and_then(|map_id| map_id.parse::<u32>().ok())
                    .ok_or_else(|| miette::miette!("invalid mapid"))?
                    != map_id
                {
                    bail!("mapid doesn't match the file name");
                }
                let xpos = child
                    .get_attribute(names.xpos)
                    .unwrap_or_default()
                    .parse::<f32>()
                    .into_diagnostic()?;
                let ypos = child
                    .get_attribute(names.ypos)
                    .unwrap_or_default()
                    .parse::<f32>()
                    .into_diagnostic()?;
                let zpos = child
                    .get_attribute(names.zpos)
                    .unwrap_or_default()
                    .parse::<f32>()
                    .into_diagnostic()?;
                let mut ca = CommonAttributes::default();
                update_common_attributes_from_element(&mut ca, &child, &names);

                let marker = Marker {
                    position: [xpos, ypos, zpos].into(),
                    map_id,
                    category,
                    props: ca.into(),
                    guid,
                };

                pack.maps.entry(map_id).or_default().markers.push(marker);
            } else if child.name() == names.trail {
                if child
                    .get_attribute(names.map_id)
                    .and_then(|map_id| map_id.parse::<u32>().ok())
                    .ok_or_else(|| miette::miette!("invalid mapid"))?
                    != map_id
                {
                    bail!("mapid doesn't match the file name");
                }
                let mut ca = CommonAttributes::default();
                update_common_attributes_from_element(&mut ca, &child, &names);

                let trail = Trail {
                    category,
                    map_id,
                    props: ca.into(),
                    guid,
                };
                pack.maps.entry(map_id).or_default().trails.push(trail);
            }
        }
    }
    Ok(())
}

// a temporary recursive function to parse the marker category tree.
fn recursive_marker_category_parser_categories_xml(
    tree: &Xot,
    tags: impl Iterator<Item = Node>,
    cats: &mut IndexMap<String, Category>,
    names: &XotAttributeNameIDs,
) {
    for tag in tags {
        if let Some(ele) = tree.element(tag) {
            if ele.name() != names.marker_category {
                continue;
            }

            let name = ele.get_attribute(names.name).unwrap_or_default();
            if name.is_empty() {
                info!("category doesn't have a name attribute: {ele:#?}");
                continue;
            }
            let span_guard = info_span!("category {name}").entered();
            let mut ca = CommonAttributes::default();
            update_common_attributes_from_element(&mut ca, ele, names);

            let display_name = ele.get_attribute(names.display_name).unwrap_or_default();

            let separator = match ele.get_attribute(names.separator).unwrap_or("0") {
                "0" => false,
                "1" => true,
                ors => {
                    info!("separator attribute has invalid value: {ors}");
                    false
                }
            };

            let default_enabled = match ele.get_attribute(names.default_enabled).unwrap_or("1") {
                "0" => false,
                "1" => true,
                ors => {
                    info!("default_enabled attribute has invalid value: {ors}");
                    true
                }
            };
            recursive_marker_category_parser_categories_xml(
                tree,
                tree.children(tag),
                &mut cats
                    .entry(name.to_string())
                    .or_insert_with(|| Category {
                        display_name: display_name.to_string(),
                        separator,
                        default_enabled,
                        props: ca,
                        children: Default::default(),
                    })
                    .children,
                names,
            );
            std::mem::drop(span_guard);
        }
    }
}

/// This first parses all the files in a zipfile into the memory and then it will try to parse a zpack out of all the files.
/// will return error if there's an issue with zipfile.
///
/// but any other errors like invalid attributes or missing markers etc.. will just be logged.
/// the intention is "best effort" parsing and not "validating" xml marker packs.
/// we will ignore any issues like unknown attributes or xml tags. "unknown" attributes means Any attributes that jokolay doesn't parse into Zpack.
#[instrument(skip_all)]
pub fn get_pack_from_taco_zip(taco: &[u8]) -> Result<PackCore> {
    // all the contents of ZPack
    let mut pack = PackCore::default();
    // parse zip file
    let mut zip_archive = zip::ZipArchive::new(std::io::Cursor::new(taco))
        .into_diagnostic()
        .wrap_err("failed to read zip archive")?;

    // file paths of different file types
    let mut images = vec![];
    let mut tbins = vec![];
    let mut xmls = vec![];
    // we collect the names first, because reading a file from zip is a mutating operation.
    // So, we can't iterate AND read the file at the same time
    for name in zip_archive.file_names() {
        if name.ends_with("png") {
            images.push(name.to_string());
        } else if name.ends_with("trl") {
            tbins.push(name.to_string());
        } else if name.ends_with("xml") {
            xmls.push(name.to_string());
        } else if name.ends_with('/') {
            // directory. so, we can ignore this.
        } else {
            info!("ignoring file: {name}");
        }
    }
    for name in images {
        let span = info_span!("load image {name}").entered();
        let file_path = RelativePath::parse_from_str(&name);
        if let Some(bytes) = read_file_bytes_from_zip_by_name(&name, &mut zip_archive) {
            match image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
                Ok(_) => assert!(
                    pack.textures.insert(file_path, bytes).is_none(),
                    "duplicate image file {name}"
                ),
                Err(e) => {
                    info!("failed to parse image file {name} due to {e:#?}");
                }
            }
        }
        std::mem::drop(span);
    }

    for name in tbins {
        let span = info_span!("load tbin {name}").entered();

        let file_path = RelativePath::parse_from_str(&name);
        if let Some(bytes) = read_file_bytes_from_zip_by_name(&name, &mut zip_archive) {
            if let Some(tbin) = parse_tbin_from_slice(&bytes) {
                assert!(
                    pack.tbins.insert(file_path, tbin).is_none(),
                    "duplicate tbin file {name}"
                );
            } else {
                info!("failed to parse tbin from slice: {file_path}");
            }
        } else {
            info!(name, "failed to read tbin from zipfile");
        }
        std::mem::drop(span);
    }
    for name in xmls {
        let mut xml_str = String::new();
        let xml_file_name = name.clone();
        let span_guard = info_span!("deserialize xml", xml_file_name).entered();
        if zip_archive
            .by_name(&name)
            .ok()
            .and_then(|mut file| file.read_to_string(&mut xml_str).ok())
            .is_none()
        {
            info!("failed to read file from zip");
            continue;
        };

        let filtered_xml_str = crate::rapid_filter_rust(xml_str);
        let mut tree = Xot::new();
        let root_node = match tree.parse(&filtered_xml_str) {
            Ok(root) => root,
            Err(e) => {
                info!(?e, "failed to parse as xml");
                continue;
            }
        };
        let names = XotAttributeNameIDs::register_with_xot(&mut tree);
        let od = match tree
            .document_element(root_node)
            .ok()
            .filter(|od| (tree.element(*od).unwrap().name() == names.overlay_data))
        {
            Some(od) => od,
            None => {
                info!("missing overlay data tag");
                continue;
            }
        };

        // parse_categories
        recursive_marker_category_parser(&tree, tree.children(od), &mut pack.categories, &names);

        let pois = match tree.children(od).find(|node| {
            tree.element(*node)
                .map(|ele: &xot::Element| ele.name() == names.pois)
                .unwrap_or_default()
        }) {
            Some(pois) => pois,
            None => {
                info!("missing pois tag");
                continue;
            }
        };

        for child_node in tree.children(pois).filter(|node| tree.is_element(*node)) {
            let child = tree.element(child_node).unwrap();
            let category = child
                .get_attribute(names.category)
                .unwrap_or_default()
                .to_lowercase();

            let guid = child
                .get_attribute(names.guid)
                .and_then(|guid| {
                    let mut buffer = [0u8; 20];
                    BASE64_ENGINE
                        .decode_slice(guid, &mut buffer)
                        .ok()
                        .and_then(|_| Uuid::from_slice(&buffer[..16]).ok())
                        .or_else(|| {
                            info!(guid, "failed to deserialize guid");
                            None
                        })
                })
                .unwrap_or_else(|| Uuid::new_v4());
            if child.name() == names.poi {
                if let Some(map_id) = child
                    .get_attribute(names.map_id)
                    .and_then(|map_id| map_id.parse::<u32>().ok())
                {
                    let xpos = child
                        .get_attribute(names.xpos)
                        .unwrap_or_default()
                        .parse::<f32>()
                        .unwrap_or_default();
                    let ypos = child
                        .get_attribute(names.ypos)
                        .unwrap_or_default()
                        .parse::<f32>()
                        .unwrap_or_default();
                    let zpos = child
                        .get_attribute(names.zpos)
                        .unwrap_or_default()
                        .parse::<f32>()
                        .unwrap_or_default();
                    let mut common_attributes = CommonAttributes::default();
                    update_common_attributes_from_element(&mut common_attributes, &child, &names);
                    if let Some(t) = common_attributes.icon_file.as_ref() {
                        if !pack.textures.contains_key(t) {
                            info!(%t, "failed to find this texture in this pack");
                        }
                    } else {
                        if let Some(icf) = child.get_attribute(names.icon_file) {
                            info!(icf, "marker's icon file attribute failed to parse");
                        }
                    }
                    let marker = Marker {
                        position: [xpos, ypos, zpos].into(),
                        map_id,
                        category,
                        props: common_attributes,
                        guid,
                    };
                    pack.maps.entry(map_id).or_default().markers.push(marker);
                } else {
                    info!("missing map id")
                }
            } else if child.name() == names.trail {
                if let Some(map_id) = child
                    .get_attribute(names.trail_data)
                    .and_then(|trail_data| {
                        let path = RelativePath::parse_from_str(trail_data);
                        pack.tbins.get(&path).map(|tb| tb.map_id)
                    })
                {
                    let mut common_attributes = CommonAttributes::default();
                    update_common_attributes_from_element(&mut common_attributes, &child, &names);

                    if let Some(tex) = common_attributes.texture.as_ref() {
                        if !pack.textures.contains_key(tex) {}
                    }

                    let trail = Trail {
                        category,
                        map_id,
                        props: common_attributes.into(),
                        guid,
                    };
                    pack.maps.entry(map_id).or_default().trails.push(trail);
                } else {
                    let td = child.get_attribute(names.trail_data);
                    let rp = RelativePath::parse_from_str(td.unwrap_or_default());
                    let tbin = pack.tbins.get(&rp).map(|tbin| (tbin.map_id, tbin.version));
                    info!("missing map_id: {td:?} {rp} {tbin:?}");
                }
            } else {
                info!("unknown tag: {:?}", child.name());
            }
        }

        drop(span_guard);
    }

    Ok(pack)
}

fn read_file_bytes_from_zip_by_name<T: std::io::Read + std::io::Seek>(
    name: &str,
    zip_archive: &mut zip::ZipArchive<T>,
) -> Option<Vec<u8>> {
    let mut bytes = vec![];
    match zip_archive.by_name(&name) {
        Ok(mut file) => match file.read_to_end(&mut bytes) {
            Ok(size) => {
                if size == 0 {
                    info!("empty file {name}");
                } else {
                    return Some(bytes);
                }
            }
            Err(e) => {
                info!("failed to read file {name} due to error {e:#?}");
            }
        },
        Err(e) => {
            info!("failed ot read file {name} due to error {e:#?}");
        }
    }
    None
}
// #[cfg(test)]
// mod test {

//     use indexmap::IndexMap;
//     use rstest::*;

//     use semver::Version;
//     use similar_asserts::assert_eq;
//     use std::io::Write;
//     use std::sync::Arc;

//     use zip::write::FileOptions;
//     use zip::ZipWriter;

//     use crate::{
//         pack::{xml::zpack_from_xml_entries, Pack, MARKER_PNG},
//         INCHES_PER_METER,
//     };

//     const TEST_XML: &str = include_str!("test.xml");
//     const TEST_MARKER_PNG_NAME: &str = "marker.png";
//     const TEST_TRL_NAME: &str = "basic.trl";

//     #[fixture]
//     #[once]
//     fn test_zip() -> Vec<u8> {
//         let mut writer = ZipWriter::new(std::io::Cursor::new(vec![]));
//         // category.xml
//         writer
//             .start_file("category.xml", FileOptions::default())
//             .expect("failed to create category.xml");
//         writer
//             .write_all(TEST_XML.as_bytes())
//             .expect("failed to write category.xml");
//         // marker.png
//         writer
//             .start_file(TEST_MARKER_PNG_NAME, FileOptions::default())
//             .expect("failed to create marker.png");
//         writer
//             .write_all(MARKER_PNG)
//             .expect("failed to write marker.png");
//         // basic.trl
//         writer
//             .start_file(TEST_TRL_NAME, FileOptions::default())
//             .expect("failed to create basic trail");
//         writer
//             .write_all(&0u32.to_ne_bytes())
//             .expect("failed to write version");
//         writer
//             .write_all(&15u32.to_ne_bytes())
//             .expect("failed to write mapid ");
//         writer
//             .write_all(bytemuck::cast_slice(&[0f32; 3]))
//             .expect("failed to write first node");
//         // done
//         writer
//             .finish()
//             .expect("failed to finalize zip")
//             .into_inner()
//     }

//     #[fixture]
//     fn test_file_entries(test_zip: &[u8]) -> IndexMap<Arc<String>, Vec<u8>> {
//         let file_entries = super::read_files_from_zip(test_zip).expect("failed to deserialize");
//         assert_eq!(file_entries.len(), 3);
//         let test_xml = std::str::from_utf8(
//             file_entries
//                 .get(String::new("category.xml"))
//                 .expect("failed to get category.xml"),
//         )
//         .expect("failed to get str from category.xml contents");
//         assert_eq!(test_xml, TEST_XML);
//         let test_marker_png = file_entries
//             .get(String::new("marker.png"))
//             .expect("failed to get marker.png");
//         assert_eq!(test_marker_png, MARKER_PNG);
//         file_entries
//     }
//     #[fixture]
//     #[once]
//     fn test_pack(test_file_entries: IndexMap<Arc<String>, Vec<u8>>) -> Pack {
//         let (pack, failures) = zpack_from_xml_entries(test_file_entries, Version::new(0, 0, 0));
//         assert!(failures.errors.is_empty() && failures.warnings.is_empty());
//         assert_eq!(pack.tbins.len(), 1);
//         assert_eq!(pack.textures.len(), 1);
//         assert_eq!(
//             pack.textures
//                 .get(String::new(TEST_MARKER_PNG_NAME))
//                 .expect("failed to get marker.png from textures"),
//             MARKER_PNG
//         );

//         let tbin = pack
//             .tbins
//             .get(String::new(TEST_TRL_NAME))
//             .expect("failed to get basic trail")
//             .clone();

//         assert_eq!(tbin.nodes[0], [0.0f32; 3].into());
//         pack
//     }

//     // #[rstest]
//     // fn test_tag(test_pack: &Pack) {
//     //     let mut test_category_menu = CategoryMenu::default();
//     //     let parent_path = String::new("parent");
//     //     let child1_path = String::new("parent/child1");
//     //     let subchild_path = String::new("parent/child1/subchild");
//     //     let child2_path = String::new("parent/child2");
//     //     test_category_menu.create_category(subchild_path);
//     //     test_category_menu.create_category(child2_path);
//     //     test_category_menu.set_display_name(parent_path, "Parent".to_string());
//     //     test_category_menu.set_display_name(child1_path, "Child 1".to_string());
//     //     test_category_menu.set_display_name(subchild_path, "Sub Child".to_string());
//     //     test_category_menu.set_display_name(child2_path, "Child 2".to_string());

//     //     assert_eq!(test_category_menu, test_pack.category_menu)
//     // }

//     #[rstest]
//     fn test_markers(test_pack: &Pack) {
//         let marker = test_pack
//             .markers
//             .values()
//             .next()
//             .expect("failed to get queensdale mapdata");
//         assert_eq!(
//             marker.props.texture.as_ref().unwrap(),
//             String::new(TEST_MARKER_PNG_NAME)
//         );
//         assert_eq!(marker.position, [INCHES_PER_METER; 3].into());
//     }
//     #[rstest]
//     fn test_trails(test_pack: &Pack) {
//         let trail = test_pack
//             .trails
//             .values()
//             .next()
//             .expect("failed to get queensdale mapdata");
//         assert_eq!(
//             trail.props.tbin.as_ref().unwrap(),
//             String::new(TEST_TRL_NAME)
//         );
//         assert_eq!(
//             trail.props.trail_texture.as_ref().unwrap(),
//             String::new(TEST_MARKER_PNG_NAME)
//         );
//     }
// }
