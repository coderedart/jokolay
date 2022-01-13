use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

use elementtree::Element;
use image::GenericImageView;

use crate::{
    json::{
        category::{Cat, CatTree},
        marker::{Achievement, Behavior, Info, Marker, MarkerFlags, Trigger},
        trail::{TBinDescription, Trail},
        ImageDescription,
    },
    xmlpack::MarkerTemplate,
};

pub fn xml_to_json_pack(pack_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut image_path_id: BTreeMap<String, u16> = BTreeMap::default();
    // trl path to (tbin_id, tbin_pos, tbin_map_id)
    let mut trail_path_id: BTreeMap<String, (u16, [f32; 3], u16)> = BTreeMap::default();
    // let mut trail_pos: BTreeMap<u16, > = BTreeMap::default();
    let mut images_descriptions: BTreeMap<u16, ImageDescription> = BTreeMap::default();
    let mut tbins_descriptions: BTreeMap<u16, TBinDescription> = BTreeMap::default();
    let mut images: BTreeMap<u16, Vec<u8>> = BTreeMap::default();
    let mut tbins: BTreeMap<u16, Vec<[f32; 3]>> = BTreeMap::default();

    // walk the directory and gather all the images + Tbins + xml files
    let mut etrees: Vec<(PathBuf, Element)> = vec![];
    let mut image_id = 0_u16;
    let mut tbin_id = 0_u16;
    dbg!("starting walk dir");
    for entry in walkdir::WalkDir::new(pack_dir) {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            match entry.path().extension().map(|e| e.to_str()).flatten() {
                // collect all xml strings, so that we can deal with them all at once
                Some("xml") => {
                    let mut src_xml = String::default();
                    std::fs::File::open(entry.path())?.read_to_string(&mut src_xml)?;
                    let post_xml = super::rapid::rapid_filter(src_xml);
                    let ele = elementtree::Element::from_reader(std::io::Cursor::new(post_xml))
                        .map_err(|e| {
                            dbg!(entry.path());
                            e
                        })?;

                    etrees.push((entry.path().to_path_buf(), ele));
                }

                Some("png") => {
                    // get the image into memory
                    let mut src_png = vec![];
                    std::fs::File::open(entry.path())?.read_to_end(&mut src_png)?;
                    // get the width/height
                    let img =
                        image::load_from_memory_with_format(&src_png, image::ImageFormat::Png)?;
                    let desc = ImageDescription {
                        name: entry
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        width: img.width().try_into()?,
                        height: img.height().try_into()?,
                    };
                    // start inserting into maps and increment id
                    images.insert(image_id, src_png);
                    images_descriptions.insert(image_id, desc);
                    image_path_id.insert(
                        entry
                            .path()
                            .strip_prefix(pack_dir)?
                            .to_string_lossy()
                            .to_lowercase(),
                        image_id,
                    );
                    image_id += 1;
                }
                Some("trl") => {
                    let mut src_trl = vec![];
                    std::fs::File::open(entry.path())?.read_to_end(&mut src_trl)?;
                    if src_trl.len() <= 12 {
                        continue;
                    }
                    let mut version_bytes = [0_u8; 4];
                    version_bytes.copy_from_slice(&src_trl[..4]);
                    let mut _version = u32::from_ne_bytes(version_bytes); // optional as we will convert to version 3

                    let mut map_id_bytes = [0_u8; 4];
                    map_id_bytes.copy_from_slice(&src_trl[4..8]);
                    let map_id = u32::from_ne_bytes(map_id_bytes);

                    let nodes: &[[f32; 3]] = bytemuck::cast_slice(&src_trl[8..]);
                    let mut nodes: Vec<[f32; 3]> = nodes.to_vec();
                    let position = if let Some(p) = nodes.first().copied() {
                        p
                    } else {
                        continue;
                    };
                    nodes.iter_mut().for_each(|p| {
                        let n = *p;
                        *p = [n[0] - position[0], n[1] - position[1], n[2] - position[2]];
                    });
                    let desc = TBinDescription {
                        name: entry
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        version: 3,
                    };
                    tbins.insert(tbin_id, nodes);
                    tbins_descriptions.insert(tbin_id, desc);
                    trail_path_id.insert(
                        entry
                            .path()
                            .strip_prefix(pack_dir)?
                            .to_string_lossy()
                            .to_lowercase(),
                        (tbin_id, position, map_id.try_into()?),
                    );
                    // trail_pos.insert(tbin_id, (position, map_id.try_into()?));
                    tbin_id += 1;
                }
                Some(_) | None => {
                    dbg!(entry.path().display());
                }
            }
        }
    }
    dbg!("done walking");
    let mut fullnames_to_catid: BTreeMap<String, u16> = BTreeMap::default();
    let mut catid_templates: BTreeMap<u16, MarkerTemplate> = BTreeMap::default();
    let mut cats: BTreeMap<u16, Cat> = BTreeMap::default();
    let mut cattree: Vec<CatTree> = vec![];
    let mut cat_id = 0_u16;
    dbg!("starting parsing mc");
    for (p, root) in etrees.iter() {
        for mc in root
            .children()
            .filter(|c| c.tag().name() == "MarkerCategory")
        {
            parse_recursive_mc(
                mc,
                &mut cattree,
                &mut fullnames_to_catid,
                &mut cats,
                &mut catid_templates,
                &MarkerTemplate::default(),
                &mut cat_id,
                "",
            )
            .map_err(|e| {
                dbg!(p);
                e
            })?;
        }
    }
    dbg!("done parsing mc");
    let mut markers: BTreeMap<u32, Marker> = BTreeMap::default();
    let mut trails: BTreeMap<u32, Trail> = BTreeMap::default();

    let mut marker_id = 0_u16;
    let mut trail_id = 0_u16;
    dbg!("starting parsing markers/trails");
    for (p, root) in etrees.iter() {
        for pois in root.children().filter(|c| c.tag().name() == "POIs") {
            parse_markers_trails(
                pois,
                &mut fullnames_to_catid,
                &mut catid_templates,
                &mut image_path_id,
                &mut trail_path_id,
                &mut marker_id,
                &mut trail_id,
                &mut markers,
                &mut trails,
            )
            .map_err(|e| {
                dbg!(p);
                e
            })?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn parse_recursive_mc(
    ele: &Element,
    cat_tree: &mut Vec<CatTree>,
    fullnames_to_catid: &mut BTreeMap<String, u16>,
    cats: &mut BTreeMap<u16, Cat>,
    catid_templates: &mut BTreeMap<u16, MarkerTemplate>,
    parent_template: &MarkerTemplate,
    cat_id: &mut u16,
    parent_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut template = parent_template.clone();
    template.override_from_element(ele).map_err(|e| {
        dbg!("{:?}", ele);
        e
    })?;
    let mut display_name = String::new();
    let mut is_separator = false;
    if let Some(dn) = ele.get_attr("DisplayName") {
        display_name = dn.to_string();
    }

    if let Some(issep) = ele.get_attr("IsSeparator=") {
        is_separator = issep.parse()?;
    }

    if let Some(name) = ele.get_attr("name") {
        let full_name = if parent_name.is_empty() {
            name.to_string()
        } else {
            parent_name.to_string() + "." + name
        };
        let id = *cat_id;
        fullnames_to_catid.insert(full_name.clone(), id);
        catid_templates.insert(id, template.clone());
        cats.insert(
            id,
            Cat {
                name: name.to_string(),
                display_name,
                is_separator,
                authors: vec![],
            },
        );
        *cat_id += 1;
        let mut children = vec![];
        for c in ele.children() {
            parse_recursive_mc(
                c,
                &mut children,
                fullnames_to_catid,
                cats,
                catid_templates,
                &template,
                cat_id,
                &full_name,
            )?;
        }
        cat_tree.push(CatTree { id, children });
    }
    Ok(())
}

impl MarkerTemplate {
    pub fn override_from_element(
        &mut self,
        ele: &Element,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (attr_name, attr_value) in ele.attrs() {
            let attr_value = attr_value.trim();
            match attr_name.name().trim() {
                "achievementId" => {
                    self.achievement_id = Some(attr_value.parse()?);
                }
                "achievementBit" => {
                    self.achievement_bit = Some(attr_value.parse()?);
                }
                "alpha" => {
                    self.alpha = Some(attr_value.parse()?);
                }
                "animSpeed" => {
                    self.anim_speed = Some(attr_value.parse()?);
                }
                "autotrigger" => {
                    self.auto_trigger = Some(attr_value.parse()?);
                }
                "behavior" => {
                    self.behavior = Some(serde_json::from_str(attr_value)?);
                }
                "color" => {
                    let mut color = [0u8; 4];
                    hex::decode_to_slice(attr_value, &mut color)?;
                    self.color = Some(color);
                }
                "fadeFar" => {
                    self.fade_far = Some(attr_value.parse()?);
                }
                "fadeNear" => {
                    self.fade_near = Some(attr_value.parse()?);
                }
                "hasCountdown" => {
                    self.has_countdown = Some(attr_value.parse()?);
                }
                "heightOffset" => {
                    self.height_offset = Some(attr_value.parse()?);
                }
                "inGameVisibility" => {
                    self.in_game_visibility = Some(attr_value.parse()?);
                }
                "iconFile" => {
                    self.icon_file = Some(attr_value.to_string());
                }
                "iconSize" => {
                    self.icon_size = Some(attr_value.parse()?);
                }
                "keepOnMapEdge" => {
                    self.keep_on_map_edge = Some(attr_value.parse()?);
                }
                "info" => {
                    self.info = Some(attr_value.to_string());
                }
                "infoRange" => {
                    self.info_range = Some(attr_value.parse()?);
                }
                "mapDisplaySize" => {
                    self.map_display_size = Some(attr_value.parse()?);
                }
                "mapFadeoutScaleLevel" => {
                    self.map_fade_out_scale_level = Some(attr_value.parse()?);
                }

                "mapVisibility" => {
                    self.map_visibility = Some(attr_value.parse()?);
                }
                "maxSize" => {
                    self.max_size = Some(attr_value.parse()?);
                }
                "minSize" => {
                    self.min_size = Some(attr_value.parse()?);
                }
                "miniMapVisibility" => {
                    self.mini_map_visibility = Some(attr_value.parse()?);
                }
                "resetLength" => {
                    self.reset_length = Some(attr_value.parse()?);
                }
                "resetOffset" => {
                    self.reset_offset = Some(attr_value.parse()?);
                }
                "scaleOnMapWithZoom" => {
                    self.scale_on_map_with_zoom = Some(attr_value.parse()?);
                }
                "texture" => {
                    self.texture = Some(attr_value.parse()?);
                }
                "toggleCategory" => {
                    self.toggle_cateogry = Some(attr_value.parse()?);
                }
                "trailData" => {
                    self.trail_data_file = Some(attr_value.parse()?);
                }
                "trailScale" => {
                    self.trail_scale = Some(attr_value.parse()?);
                }
                "triggerRange" => {
                    self.trigger_range = Some(attr_value.parse()?);
                }
                rest => match rest {
                    "DisplayName" | "name" | "xpos" | "ypos" | "zpos" | "IsSeparator" | "type"
                    | "mapID"  | "GUID" | "MapID" | "copy"=> {}
                    rest => match rest {
                        "triggerrange" => {
                            dbg!(rest);
                        }
                        rest => {
                            panic!("invalid attribute. name: {}", rest);
                        }
                    },
                },
            }
        }
        // if let Some(achievement_id) = ele.get_attr("achievementId") {}
        // if let Some(achievement_bit) = ele.get_attr("achievementBit") {}
        // if let Some(alpha) = ele.get_attr("alpha") {}
        // if let Some(auto_trigger) = ele.get_attr("autotrigger") {
        //     self.auto_trigger = Some(auto_trigger.parse()?);
        // }
        // if let Some(_behavior) = ele.get_attr("behavior") {
        //     // template.behavior = Some();
        // }
        // if let Some(color_hex) = ele.get_attr("color") {
        //     let mut color = [0u8; 4];
        //     hex::decode_to_slice(color_hex, &mut color)?;
        //     self.color = Some(color);
        // }
        // if let Some(ff) = ele.get_attr("fadeFar") {
        // }
        // if let Some(fnear) = ele.get_attr("fadeNear") {
        // }
        // if let Some(has_countdown) = ele.get_attr("hasCountdown") {
        // }
        // if let Some(height_offset) = ele.get_attr("heightOffset") {
        //     self.height_offset = Some(height_offset.parse()?);
        // }
        // if let Some(in_game_visibility) = ele.get_attr("inGameVisibility") {
        //     self.in_game_visibility = Some(in_game_visibility.parse()?);
        // }
        // if let Some(icon_file) = ele.get_attr("iconFile") {
        //     self.icon_file = Some(icon_file.to_string());
        // }
        // if let Some(icon_size) = ele.get_attr("iconSize") {
        //     self.icon_size = Some(icon_size.parse()?);
        // }
        // if let Some(kome) = ele.get_attr("keepOnMapEdge") {
        //     self.keep_on_map_edge = Some(kome.parse()?);
        // }
        // if let Some(info) = ele.get_attr("info") {
        //     self.info = Some(info.to_string());
        // }
        // if let Some(info_range) = ele.get_attr("infoRange") {
        //     self.info_range = Some(info_range.parse()?);
        // }
        // if let Some(map_display_size) = ele.get_attr("mapDisplaySize") {
        //     self.map_display_size = Some(map_display_size.parse()?);
        // }
        // if let Some(map_fade_out_scale_level) = ele.get_attr("mapFadeoutScaleLevel") {
        //     self.map_fade_out_scale_level = Some(map_fade_out_scale_level.parse()?);
        // }
        // if let Some(map_visibility) = ele.get_attr("mapVisibility") {
        //     self.map_visibility = Some(map_visibility.parse()?);
        // }
        // if let Some(max_size) = ele.get_attr("maxSize") {
        //     self.max_size = Some(max_size.parse()?);
        // }
        // if let Some(min_size) = ele.get_attr("minSize") {
        //     self.min_size = Some(min_size.parse()?);
        // }
        // if let Some(mini_map_visibility) = ele.get_attr("miniMapVisibility") {
        //     self.mini_map_visibility = Some(mini_map_visibility.parse()?);
        // }
        // if let Some(reset_length) = ele.get_attr("resetLength") {
        //     self.reset_length = Some(reset_length.parse()?);
        // }
        // if let Some(reset_offset) = ele.get_attr("resetOffset") {
        //     self.reset_offset = Some(reset_offset.parse()?);
        // }
        // if let Some(scale_on_map_with_zoom) = ele.get_attr("scaleOnMapWithZoom") {
        //     self.scale_on_map_with_zoom = Some(scale_on_map_with_zoom.parse()?);
        // }
        // if let Some(toggle_cateogry) = ele.get_attr("toggleCategory") {
        //     self.toggle_cateogry = Some(toggle_cateogry.parse()?);
        // }
        // if let Some(trigger_range) = ele.get_attr("triggerRange") {
        //     self.trigger_range = Some(trigger_range.parse()?);
        // }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_markers_trails(
    ele: &Element,
    fullnames_to_catid: &mut BTreeMap<String, u16>,
    catid_templates: &mut BTreeMap<u16, MarkerTemplate>,
    image_path_id: &mut BTreeMap<String, u16>,
    trail_path_id: &mut BTreeMap<String, (u16, [f32; 3], u16)>,
    marker_id: &mut u16,
    trail_id: &mut u16,
    markers: &mut BTreeMap<u32, Marker>,
    trails: &mut BTreeMap<u32, Trail>,
) -> Result<(), Box<dyn std::error::Error>> {
    if ele.tag().name() == "POIs" {
        for (index, mt) in ele.children().enumerate() {
            dbg!(index);
            match mt.tag().name() {
                "POI" => {
                    let mut position = [0_f32; 3];
                    if let Some(xpos) = mt.get_attr("xpos") {
                        position[0] = xpos.trim().parse().map_err(|e| {
                            dbg!(mt);
                            e
                        })?;
                    }
                    if let Some(ypos) = mt.get_attr("ypos") {
                        position[1] = ypos.trim().parse().map_err(|e| {
                            dbg!(mt);
                            e
                        })?;
                    }
                    if let Some(zpos) = mt.get_attr("zpos") {
                        position[2] = zpos.trim().parse().map_err(|e| {
                            dbg!(mt);
                            e
                        })?;
                    }

                    let cat = if let Some(fullname) = mt.get_attr("type") {
                        if let Some(catid) = fullnames_to_catid.get(fullname) {
                            *catid
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };
                    let mut m = Marker {
                        position,
                        cat,
                        ..Default::default()
                    };
                    let mut template = if let Some(t) = catid_templates.get(&cat) {
                        t.clone()
                    } else {
                        continue;
                    };
                    template.override_from_element(mt).map_err(|e| {
                        dbg!("{:?}", ele);
                        e
                    })?;

                    m.alpha = template.alpha.map(|a| (a * 255.0) as u8);
                    m.color = template.color;
                    if template.fade_far.is_some() || template.fade_near.is_some() {
                        m.fade_range = Some([
                            template.fade_near.unwrap_or_default() as f32,
                            template.fade_far.unwrap_or_default() as f32,
                        ]);
                    }
                    m.min_size = template.min_size;
                    m.max_size = template.max_size;
                    m.map_display_size = template.map_display_size;
                    m.map_fade_out_scale_level = template.map_fade_out_scale_level;
                    m.scale = template.icon_size;

                    m.flags.set(
                        MarkerFlags::AUTO_TRIGGER,
                        template.auto_trigger.unwrap_or(0) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::COUNT_DOWN,
                        template.has_countdown.unwrap_or(0) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::IN_GAME_VISIBILITY,
                        template.in_game_visibility.unwrap_or(1) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::MAP_SCALE,
                        template.scale_on_map_with_zoom.unwrap_or(1) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::MAP_VISIBILITY,
                        template.map_visibility.unwrap_or(1) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::MINI_MAP_EDGE_HERD,
                        template.keep_on_map_edge.unwrap_or(0) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::MINI_MAP_VISIBILITY,
                        template.mini_map_visibility.unwrap_or(1) != 0,
                    );

                    if let Some(info) = template.info {
                        m.dynamic_props = Some(m.dynamic_props.unwrap_or_default());
                        let info = Info {
                            text: info,
                            range: template.info_range,
                        };
                        if let Some(ref mut d) = m.dynamic_props {
                            d.info = Some(info);
                        }
                    }
                    if let Some(aid) = template.achievement_id {
                        m.dynamic_props = Some(m.dynamic_props.unwrap_or_default());
                        let achievement = Achievement {
                            id: aid,
                            bit: template.achievement_bit.unwrap_or(u8::MAX),
                        };
                        if let Some(ref mut d) = m.dynamic_props {
                            d.achievement = Some(achievement);
                        }
                    }
                    let range = template.trigger_range.unwrap_or(2.0);
                    let behavior = if let Some(b) = template.behavior {
                        Some(match b {
                            super::xml_marker::Behavior::AlwaysVisible => Behavior::AlwaysVisible,
                            super::xml_marker::Behavior::ReappearOnMapChange => {
                                Behavior::ReappearOnMapChange
                            }
                            super::xml_marker::Behavior::ReappearOnDailyReset => {
                                Behavior::ReappearOnDailyReset
                            }
                            super::xml_marker::Behavior::OnlyVisibleBeforeActivation => {
                                Behavior::OnlyVisibleBeforeActivation
                            }
                            super::xml_marker::Behavior::ReappearAfterTimer => {
                                Behavior::ReappearAfterTimer {
                                    reset_length: template.reset_length.unwrap_or(10),
                                }
                            }
                            super::xml_marker::Behavior::ReappearOnMapReset => {
                                Behavior::ReappearOnMapReset {
                                    map_cycle_length: template.reset_length.unwrap_or(7200),
                                    map_cycle_offset_after_reset: template
                                        .reset_offset
                                        .unwrap_or_default(),
                                }
                            }
                            super::xml_marker::Behavior::OncePerInstance => {
                                Behavior::OncePerInstance
                            }
                            super::xml_marker::Behavior::DailyPerChar => Behavior::DailyPerChar,
                            super::xml_marker::Behavior::OncePerInstancePerChar => {
                                Behavior::OncePerInstancePerChar
                            }
                            super::xml_marker::Behavior::WvWObjective => Behavior::WvWObjective,
                        })
                    } else {
                        None
                    };
                    let toggle_cat = template
                        .toggle_cateogry
                        .map(|full_name| fullnames_to_catid.get(&full_name).copied())
                        .flatten();
                    if toggle_cat.is_some() || behavior.is_some() {
                        m.dynamic_props = Some(m.dynamic_props.unwrap_or_default());

                        if let Some(ref mut d) = m.dynamic_props {
                            d.trigger = Some(Trigger {
                                range,
                                behavior,
                                toggle_cat,
                            });
                        }
                    }
                    if let Some(tex) = template.icon_file {
                        let texture_id = image_path_id.get(&tex.to_lowercase()).unwrap();
                        m.texture = Some(*texture_id);
                    }
                    let map_id: u16 = mt.get_attr("MapID").expect(&format!("no mapID in element {:?}", mt)).parse().unwrap();
                    loop {
                        let mut id = [0_u8; 4];
                        let mid_bytes = marker_id.to_ne_bytes();
                        let map_bytes = map_id.to_ne_bytes();
                        id[0..2].copy_from_slice(&mid_bytes);
                        id[2..4].copy_from_slice(&map_bytes);

                        let id = u32::from_ne_bytes(id);
                        if let std::collections::btree_map::Entry::Vacant(e) = markers.entry(id) {
                            e.insert(m);
                            break;
                        }
                        *marker_id += 1;
                    }
                }
                "Trail" => {
                    let tdfile = mt.get_attr("trailData").unwrap();
                    let (tbin_id, tposition, map_id) =
                        trail_path_id.get(&tdfile.to_lowercase()).copied().unwrap();
                    let cat = if let Some(fullname) = mt.get_attr("type") {
                        if let Some(catid) = fullnames_to_catid.get(fullname) {
                            *catid
                        } else {
                            println!("missing category: {} in element: {:?}", fullname, mt);
                            panic!("{:#?}", fullnames_to_catid);
                            continue;
                        }
                    } else {
                        panic!("missing category in element: {:?}", mt);
                    };
                    let mut m = Trail {
                        pos: tposition,
                        tbin: tbin_id,
                        cat,
                        ..Default::default()
                    };
                    let mut template = if let Some(t) = catid_templates.get(&cat) {
                        t.clone()
                    } else {
                        continue;
                    };
                    template.override_from_element(mt).map_err(|e| {
                        dbg!("{:?}", ele);
                        e
                    })?;

                    m.alpha = template.alpha.map(|a| (a * 255.0) as u8);
                    m.color = template.color;
                    if template.fade_far.is_some() || template.fade_near.is_some() {
                        m.fade_range = Some([
                            template.fade_near.unwrap_or_default() as f32,
                            template.fade_far.unwrap_or_default() as f32,
                        ]);
                    }
                    m.anim_speed = template.anim_speed;

                    m.map_display_size = template.map_display_size;
                    m.map_fade_out_scale_level = template.map_fade_out_scale_level;
                    m.scale = template.trail_scale;

                    m.flags.set(
                        MarkerFlags::IN_GAME_VISIBILITY,
                        template.in_game_visibility.unwrap_or(1) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::MAP_SCALE,
                        template.scale_on_map_with_zoom.unwrap_or(1) != 0,
                    );
                    m.flags.set(
                        MarkerFlags::MAP_VISIBILITY,
                        template.map_visibility.unwrap_or(1) != 0,
                    );

                    m.flags.set(
                        MarkerFlags::MINI_MAP_VISIBILITY,
                        template.mini_map_visibility.unwrap_or(1) != 0,
                    );

                    if let Some(aid) = template.achievement_id {
                        let achievement = Achievement {
                            id: aid,
                            bit: template.achievement_bit.unwrap_or(u8::MAX),
                        };
                        m.achievement = Some(achievement);
                    }

                    if let Some(tex) = template.icon_file {
                        let texture_id = image_path_id.get(&tex.to_lowercase()).unwrap();
                        m.texture = Some(*texture_id);
                    }
                    loop {
                        let mut id = [0_u8; 4];
                        let tid_bytes = trail_id.to_ne_bytes();
                        let map_bytes = map_id.to_ne_bytes();
                        id[0..2].copy_from_slice(&tid_bytes);
                        id[2..4].copy_from_slice(&map_bytes);

                        let id = u32::from_ne_bytes(id);
                        if let std::collections::btree_map::Entry::Vacant(e) = trails.entry(id) {
                            e.insert(m);
                            break;
                        }
                        *trail_id += 1;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
