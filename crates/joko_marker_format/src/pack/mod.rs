mod common;
mod error;
mod marker;
mod trail;

use base64::Engine;
use cap_std::fs::Dir;
use indexmap::IndexMap;
use miette::{bail, Context, IntoDiagnostic};

pub const MARKER_PNG: &[u8] = include_bytes!("marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("trail.png");

use relative_path::RelativePathBuf;
use semver::Version;
use serde::{Deserialize, Serialize};

use tracing::warn;

use std::{collections::{BTreeMap, BTreeSet}, io::Write, path::Path};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use relative_path::RelativePath;

use std::collections::HashMap;

use std::io::Read;

use xot::{NameId, Node, SerializeOptions, Xot};

pub use common::*;
pub use error::*;
pub use marker::*;
use std::sync::Arc;
pub use trail::*;

#[derive(Default)]
pub struct Pack {
    pub textures: BTreeMap<RelativePathBuf, Vec<u8>>,
    pub tbins: BTreeMap<RelativePathBuf, TBin>,
    pub categories: IndexMap<String, Category>,
    pub maps: BTreeMap<u32, MapData>,
}

#[derive(Default)]
pub struct MapData {
    pub markers: Vec<Marker>,
    pub trails: Vec<Trail>,
}

impl Pack {
    pub fn from_dir(dir: &Dir) -> miette::Result<Self> {
        let mut files: HashMap<RelativePathBuf, Vec<u8>> = HashMap::new();
        Self::recursive_walk_dir(dir, &mut files, RelativePath::new(""))
            .wrap_err("failed to walk dir when loading a markerpack")?;
        let mut pack = Pack::default();
        let cats_xml = files
            .remove(RelativePath::new("categories.xml"))
            .ok_or(miette::miette!("missing categories.xml"))?;
        // a temporary recursive function to parse the marker category tree.
        fn recursive_marker_category_parser_categories_xml(
            tree: &Xot,
            tags: impl Iterator<Item = Node>,
            cats: &mut IndexMap<String, Category>,
            names: &XotAttributeNameIDs,
        ) -> miette::Result<()> {
            for tag in tags.filter(|node| tree.is_element(*node)) {
                let ele = tree.element(tag).unwrap();
                if ele.name() != names.marker_category {
                    continue;
                }

                let name = ele.get_attribute(names.name).unwrap_or_default();
                if name.is_empty() {
                    miette::bail!("category doesn't have a name attribute");
                }
                let mut common_attributes = CommonAttributes::default();
                common_attributes.update_from_element(ele, names);

                let display_name = ele.get_attribute(names.display_name).unwrap_or_default();

                let separator = ele
                    .get_attribute(names.separator)
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or_default();

                let default_enabled = ele
                    .get_attribute(names.default_enabled)
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(true);
                recursive_marker_category_parser_categories_xml(
                    tree,
                    tree.children(tag),
                    &mut cats
                        .entry(name.to_string())
                        .or_insert_with(|| Category {
                            display_name: display_name.to_string(),
                            separator,
                            default_enabled,
                            props: common_attributes.into(),
                            children: Default::default(),
                        })
                        .children,
                    names,
                )?;
            }
            Ok(())
        }
        let mut tree = xot::Xot::new();
        let overlay_data = tree
            .parse(
                std::str::from_utf8(&cats_xml)
                    .into_diagnostic()
                    .wrap_err("failed to parse cats as xml text")?,
            )
            .into_diagnostic()?;
        let xot_names = XotAttributeNameIDs::register_with_xot(&mut tree);
        if let Some(od) = tree.element(overlay_data) {
            if od.name() != xot_names.overlay_data {
                bail!("root tag is not OverlayData")
            }
            recursive_marker_category_parser_categories_xml(
                &tree,
                tree.children(overlay_data),
                &mut pack.categories,
                &xot_names,
            )?;
        } else {
            miette::bail!("failed to get overlay data tag of cats xml");
        }

        for (path, contents) in files {
            let ext = path
                .extension()
                .ok_or(miette::miette!("no file extension"))?;
            match ext {
                "xml" => {
                    let map_id: u32 = path
                        .file_stem()
                        .ok_or(miette::miette!("missing file name"))?
                        .parse()
                        .into_diagnostic()?;
                    let mut tree = Xot::new();
                    let overlay_data = tree
                        .parse(std::str::from_utf8(&contents).into_diagnostic()?)
                        .into_diagnostic()?;
                    let names = XotAttributeNameIDs::register_with_xot(&mut tree);
                    let od = tree
                        .element(overlay_data)
                        .ok_or(miette::miette!("failed to get overlay data tag"))?;
                    if od.name() != names.overlay_data {
                        bail!("missingoverlay data tag ");
                    }
                    let pois = tree
                        .children(overlay_data)
                        .find(|node| match tree.element(*node) {
                            Some(ele) => ele.name() == names.pois,
                            None => false,
                        })
                        .ok_or(miette::miette!("missing pois node"))?;
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
                                    Self::BASE64_ENGINE
                                        .decode_slice(guid, &mut buffer)
                                        .ok()
                                        .and_then(|_| Uuid::from_slice(&buffer[..16]).ok())
                                })
                                .ok_or(miette::miette!("invalid guid"))?;
                            if child.name() == names.poi {
                                if child
                                    .get_attribute(names.map_id)
                                    .and_then(|map_id| map_id.parse::<u32>().ok())
                                    .ok_or(miette::miette!("invalid mapid"))?
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
                                let mut common_attributes = CommonAttributes::default();
                                common_attributes.update_from_element(&child, &names);

                                let marker = Marker {
                                    position: [xpos, ypos, zpos].into(),
                                    map_id,
                                    category,
                                    props: common_attributes.into(),
                                    guid,
                                };

                                pack.maps.entry(map_id).or_default().markers.push(marker);
                            } else if child.name() == names.trail {
                                if child
                                    .get_attribute(names.trail_data)
                                    .and_then(|trail_data| {
                                        RelativePathBuf::try_from(trail_data.to_lowercase())
                                            .ok()
                                            .map(|t| pack.tbins.get(&t).map(|tb| tb.map_id))
                                            .flatten()
                                    })
                                    .ok_or(miette::miette!("missing mapid of trail"))?
                                    != map_id
                                {
                                    bail!("mapid doesn't match the file name");
                                }

                                let mut common_attributes = CommonAttributes::default();
                                common_attributes.update_from_element(&child, &names);

                                let trail = Trail {
                                    category,
                                    props: common_attributes.into(),
                                    guid,
                                };
                                pack.maps.entry(map_id).or_default().trails.push(trail);
                            }
                        }
                    }
                }
                "png" => {
                    pack.textures.insert(path, contents);
                }
                "trl" => {
                    pack.tbins
                        .insert(path, TBin::parse_from_slice(&contents).into_diagnostic()?);
                }
                _ => {
                    bail!("unrecognized file extension: {path}")
                }
            }
        }
        Ok(pack)
    }
    fn recursive_walk_dir(
        dir: &Dir,
        files: &mut HashMap<RelativePathBuf, Vec<u8>>,
        parent_path: &RelativePath,
    ) -> miette::Result<()> {
        for file in dir.entries().into_diagnostic()? {
            let file = file.into_diagnostic()?;
            let name = file
                .file_name()
                .to_str()
                .ok_or(miette::miette!("file name is not utf-8"))?
                .to_lowercase();
            let path = parent_path.join(&name);

            if file.file_type().into_diagnostic()?.is_file() {
                if name.ends_with("xml") || name.ends_with("png") || name.ends_with("trl") {
                    let mut bytes = vec![];
                    file.open()
                        .into_diagnostic()?
                        .read_to_end(&mut bytes)
                        .into_diagnostic()?;
                    files.insert(path, bytes);
                } else if name == "info.json" || name == "activation.json" {
                    // they are jokolay files, so ignore them
                } else {
                    warn!("weird file while loading marker pack: {path}");
                }
            } else {
                Self::recursive_walk_dir(&file.open_dir().into_diagnostic()?, files, &path)?;
            }
        }
        Ok(())
    }
    pub fn save_to_dir(&self, dir: &Dir) -> miette::Result<()> {
        {
            // save categories
            let mut tree = Xot::new();
            let names = XotAttributeNameIDs::register_with_xot(&mut tree);
            let od = tree.new_element(names.overlay_data);
            // let od = tree.document_element(od).into_diagnostic()?;
            let root_node = tree
                .new_root(od)
                .into_diagnostic()
                .wrap_err("failed to create new root with overlay data node")?;
            Self::recursive_cat_serializer(&mut tree, &names, &self.categories, od)
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
            warn!("map count: {}", self.maps.len());
            for (map_id, map_data) in &self.maps {
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
                    marker.serialize_to_element(ele, &names);
                }
                for trail in &map_data.trails {
                    let trail_node = tree.new_element(names.trail);
                    tree.append(pois, trail_node)
                        .into_diagnostic()
                        .wrap_err("failed to append a trail node to pois")?;
                    let ele = tree.element_mut(trail_node).unwrap();
                    trail.serialize_to_element(ele, &names);
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
            for (img_path, img) in &self.textures {
                if let Some(parent) = img_path.parent() {
                    dir.create_dir_all(parent.as_str())
                        .into_diagnostic()
                        .wrap_err("failed to create parent for an image")?;
                }
                dir.create(img_path.as_str())
                    .into_diagnostic()
                    .wrap_err("failed to create file for image")?
                    .write(&img)
                    .into_diagnostic()
                    .wrap_err("failed to write image bytes to file")?;
            }
        }
        // save tbins
        {
            for (tbin_path, tbin) in &self.tbins {
                if let Some(parent) = tbin_path.parent() {
                    dir.create_dir_all(parent.as_str()).into_diagnostic()?;
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
                    .into_diagnostic()?
                    .write_all(&bytes)
                    .into_diagnostic()?;
            }
        }
        Ok(())
    }
    fn recursive_cat_serializer(
        tree: &mut Xot,
        names: &XotAttributeNameIDs,
        cats: &IndexMap<String, Category>,
        parent: Node,
    ) -> miette::Result<()> {
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
                cat.props.serialize_to_element(ele, names);
            }
            Self::recursive_cat_serializer(tree, names, &cat.children, cat_node)?;
        }
        Ok(())
    }
    pub const BASE64_ENGINE: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::GeneralPurposeConfig::new(),
    );
    /// This first parses all the files in a zipfile into the memory and then it will try to parse a zpack out of all the files.
    /// will return error if there's an issue with zipfile.
    ///
    /// but any other errors like invalid attributes or missing markers etc.. will just be logged into the Failures struct that is returned.
    /// the intention is "best effort" parsing and not "validating" xml marker packs.
    /// we will ignore any issues like unknown attributes or xml tags. "unknown" attributes means Any attributes that jokolay doesn't parse into Zpack.
    ///
    /// Generally speaking, if a pack works in `Taco` or `BlishHUD`, it should work here too.
    pub fn get_pack_from_taco_zip(taco: &[u8]) -> miette::Result<Pack> {
        // all the contents of ZPack
        let mut pack = Pack::default();
        // parse zip file
        let mut zip_archive = zip::ZipArchive::new(std::io::Cursor::new(taco)).into_diagnostic()?;
        // file paths of different file types
        let mut images = vec![];
        let mut tbins = vec![];
        let mut xmls = vec![];
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
                // what file is this??
            }
        }
        for name in images {
            let file_path: Arc<RelativePath> =
                match RelativePath::from_path(Path::new(&name.to_lowercase())) {
                    Ok(file_path) => Arc::from(file_path),
                    Err(_) => {
                        continue;
                    }
                };

            let mut bytes = vec![];
            if zip_archive
                .by_name(&name)
                .ok()
                .and_then(|mut file| file.read_to_end(&mut bytes).ok())
                .is_none()
            {
                continue;
            };
            match image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
                Ok(_) => {
                    if pack
                        .textures
                        .insert(file_path.to_relative_path_buf(), bytes)
                        .is_some()
                    {}
                }
                Err(_e) => {}
            }
        }

        // while parsing tbins, we store the map ids. Later, when parsing xml files,
        // we can sort the trails into their respective maps by using this map to look up the mapid from respective tbin
        let mut tbin_map_ids: HashMap<Arc<RelativePath>, u32> = HashMap::new();
        for name in tbins {
            let file_path: Arc<RelativePath> =
                match RelativePath::from_path(Path::new(&name.to_lowercase())) {
                    Ok(file_path) => Arc::from(file_path),
                    Err(_) => {
                        continue;
                    }
                };
            let _this_file_error = PerFileErrors {
                file_path: Some(file_path.clone()),
                errors: vec![],
                warnings: vec![],
            };

            let mut bytes = vec![];
            if zip_archive
                .by_name(&name)
                .ok()
                .and_then(|mut file| file.read_to_end(&mut bytes).ok())
                .is_none()
            {
                continue;
            };
            match TBin::parse_from_slice(&bytes) {
                Ok(tbin) => {
                    tbin_map_ids.insert(file_path.clone(), tbin.map_id);

                    if pack
                        .tbins
                        .insert(file_path.to_relative_path_buf(), tbin)
                        .is_some()
                    {}
                }
                Err(_e) => {}
            }
        }

        for name in xmls {
            let _file_path: Arc<RelativePath> =
                match RelativePath::from_path(Path::new(&name.to_lowercase())) {
                    Ok(file_path) => Arc::from(file_path),
                    Err(_) => {
                        continue;
                    }
                };

            let mut bytes = vec![];
            if zip_archive
                .by_name(&name)
                .ok()
                .and_then(|mut file| file.read_to_end(&mut bytes).ok())
                .is_none()
            {
                continue;
            };

            let xml_str = match String::from_utf8(bytes) {
                Ok(xml) => xml,
                Err(_e) => {
                    continue;
                }
            };
            let filtered_xml_str = crate::rapid_filter_rust(xml_str);
            let mut tree = Xot::new();
            let root_node = match tree.parse(&filtered_xml_str) {
                Ok(root) => root,
                Err(_e) => {
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
                    continue;
                }
            };

            // parse_categories
            recursive_marker_category_parser(
                &tree,
                tree.children(od),
                &mut pack.categories,
                &names,
            );

            if let Some(pois) = tree.children(od).find(|node| {
                tree.element(*node)
                    .map(|ele: &xot::Element| ele.name() == names.pois)
                    .unwrap_or_default()
            }) {
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
                            Self::BASE64_ENGINE
                                .decode_slice(guid, &mut buffer)
                                .ok()
                                .and_then(|_| Uuid::from_slice(&buffer[..16]).ok())
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
                            common_attributes.update_from_element(&child, &names);
                            if let Some(t) = common_attributes.icon_file.as_ref() {
                                if !pack.textures.contains_key(t) {}
                            }
                            let marker = Marker {
                                position: [xpos, ypos, zpos].into(),
                                map_id,
                                category,
                                props: common_attributes.into(),
                                guid,
                            };
                            pack.maps.entry(map_id).or_default().markers.push(marker);
                        } else {
                        }
                    } else if child.name() == names.trail {
                        if let Some(map_id) =
                            child
                                .get_attribute(names.trail_data)
                                .and_then(|trail_data| {
                                    RelativePathBuf::try_from(trail_data.to_lowercase())
                                        .ok()
                                        .map(|t| pack.tbins.get(&t).map(|tb| tb.map_id))
                                        .flatten()
                                })
                        {
                            let mut common_attributes = CommonAttributes::default();
                            common_attributes.update_from_element(&child, &names);

                            if let Some(tex) = common_attributes.texture.as_ref() {
                                if !pack.textures.contains_key(tex) {}
                            }

                            let trail = Trail {
                                category,
                                props: common_attributes.into(),
                                guid,
                            };
                            pack.maps.entry(map_id).or_default().trails.push(trail);
                        } else {
                            // this_file_error.warnings.push(PackWarning::);
                        }
                    }
                }
            };
        }

        Ok(pack)
    }
}

#[derive(Debug)]
pub struct Category {
    pub display_name: String,
    pub separator: bool,
    pub default_enabled: bool,
    pub props: CommonAttributes,
    pub children: IndexMap<String, Category>,
}

pub struct ActivationData {
    pub cats_status: BTreeSet<String>,
    /// the key is marker id. and the value is the timestamp at which we can remove this entry.
    /// and we store the data separate for each map, so that duplicate ids across maps don't conflict.
    pub markers_status: BTreeMap<u32, BTreeMap<Uuid, u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PackInfo {
    /// name of the pack.
    pub name: String,
    /// The version of the pack. This will allow us in future to have two versions of the same pack and be able to diff them.
    /// default is 0.0.1 if nothing is specified
    pub version: Option<Version>,
    /// The url from which this was downloaded from. None if it was imported from a local zip file.
    pub url: Option<Url>,
    /// The timestamp when it was download/created/imported.
    pub created: Option<OffsetDateTime>,
    /// The timestamp when it was last modified.
    pub modified: Option<OffsetDateTime>,
}

impl Default for PackInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            version: Default::default(),
            url: Default::default(),
            created: Default::default(),
            modified: Default::default(),
        }
    }
}

// Welcome to the messiest part of the code
#[allow(clippy::too_many_arguments)]
// a temporary recursive function to parse the marker category tree.
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
        let mut common_attributes = CommonAttributes::default();
        common_attributes.update_from_element(ele, names);

        let display_name = ele.get_attribute(names.display_name).unwrap_or_default();

        let separator = ele
            .get_attribute(names.separator)
            .unwrap_or_default()
            .parse()
            .unwrap_or_default();

        let default_enabled = ele
            .get_attribute(names.default_enabled)
            .unwrap_or_default()
            .parse()
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
                    props: common_attributes.into(),
                    children: Default::default(),
                })
                .children,
            names,
        );
    }
}

pub struct XotAttributeNameIDs {
    overlay_data: NameId,
    marker_category: NameId,
    pois: NameId,
    poi: NameId,
    trail: NameId,
    category: NameId,
    xpos: NameId,
    ypos: NameId,
    zpos: NameId,
    icon_file: NameId,
    texture: NameId,
    trail_data: NameId,
    separator: NameId,
    display_name: NameId,
    default_enabled: NameId,
    name: NameId,
    map_id: NameId,
    guid: NameId,
}
impl XotAttributeNameIDs {
    fn register_with_xot(tree: &mut Xot) -> Self {
        Self {
            overlay_data: tree.add_name("OverlayData"),
            marker_category: tree.add_name("MarkerCategory"),
            pois: tree.add_name("POIs"),
            poi: tree.add_name("POI"),
            trail: tree.add_name("Trail"),
            category: tree.add_name("type"),
            xpos: tree.add_name("xpos"),
            ypos: tree.add_name("ypos"),
            zpos: tree.add_name("zpos"),
            icon_file: tree.add_name("iconfile"),
            texture: tree.add_name("texture"),
            trail_data: tree.add_name("trailfile"),
            separator: tree.add_name("IsSeparator"),
            name: tree.add_name("name"),
            default_enabled: tree.add_name("defaulttoggle"),
            display_name: tree.add_name("DisplayName"),
            map_id: tree.add_name("MapID"),
            guid: tree.add_name("GUID"),
        }
    }
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
//     fn test_file_entries(test_zip: &[u8]) -> IndexMap<Arc<RelativePath>, Vec<u8>> {
//         let file_entries = super::read_files_from_zip(test_zip).expect("failed to deserialize");
//         assert_eq!(file_entries.len(), 3);
//         let test_xml = std::str::from_utf8(
//             file_entries
//                 .get(RelativePath::new("category.xml"))
//                 .expect("failed to get category.xml"),
//         )
//         .expect("failed to get str from category.xml contents");
//         assert_eq!(test_xml, TEST_XML);
//         let test_marker_png = file_entries
//             .get(RelativePath::new("marker.png"))
//             .expect("failed to get marker.png");
//         assert_eq!(test_marker_png, MARKER_PNG);
//         file_entries
//     }
//     #[fixture]
//     #[once]
//     fn test_pack(test_file_entries: IndexMap<Arc<RelativePath>, Vec<u8>>) -> Pack {
//         let (pack, failures) = zpack_from_xml_entries(test_file_entries, Version::new(0, 0, 0));
//         assert!(failures.errors.is_empty() && failures.warnings.is_empty());
//         assert_eq!(pack.tbins.len(), 1);
//         assert_eq!(pack.textures.len(), 1);
//         assert_eq!(
//             pack.textures
//                 .get(RelativePath::new(TEST_MARKER_PNG_NAME))
//                 .expect("failed to get marker.png from textures"),
//             MARKER_PNG
//         );

//         let tbin = pack
//             .tbins
//             .get(RelativePath::new(TEST_TRL_NAME))
//             .expect("failed to get basic trail")
//             .clone();

//         assert_eq!(tbin.nodes[0], [0.0f32; 3].into());
//         pack
//     }

//     // #[rstest]
//     // fn test_tag(test_pack: &Pack) {
//     //     let mut test_category_menu = CategoryMenu::default();
//     //     let parent_path = RelativePath::new("parent");
//     //     let child1_path = RelativePath::new("parent/child1");
//     //     let subchild_path = RelativePath::new("parent/child1/subchild");
//     //     let child2_path = RelativePath::new("parent/child2");
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
//             RelativePath::new(TEST_MARKER_PNG_NAME)
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
//             RelativePath::new(TEST_TRL_NAME)
//         );
//         assert_eq!(
//             trail.props.trail_texture.as_ref().unwrap(),
//             RelativePath::new(TEST_MARKER_PNG_NAME)
//         );
//     }
// }
