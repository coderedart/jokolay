use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use elementtree::Element;
// use image::GenericImageView;
use itertools::Itertools;

use crate::json::{FullPack, ImageSrc, PackData};
use crate::{
    json::{
        Achievement, Behavior, Cat, CatTree, ImageDescription, Info, Marker, MarkerFlags, Pack,
        TBinDescription, Trail, Trigger,
    },
    xmlpack::MarkerTemplate,
    INCHES_PER_METER,
};

#[derive(Debug, Default)]
struct XmlPackEntries {
    /// has relative paths to image id
    img_relative_path_id: BTreeMap<String, u16>,
    /// relative trl path to (tbin_id, tbin_pos, tbin_map_id)
    trl_relative_path_id: BTreeMap<String, (u16, [f32; 3], u32)>,
    /// image descriptions for json pack
    img_desc: BTreeMap<u16, ImageDescription>,
    /// tbin descriptions for json pack
    tbin_desc: BTreeMap<u16, TBinDescription>,
    /// image raw bytes of image id
    images: BTreeMap<u16, Vec<u8>>,
    /// tbin raw bytes of tbin id
    tbins: BTreeMap<u16, Vec<[f32; 3]>>,
    /// Element Trees of parsed xml files along with their path
    elements: Vec<(Arc<PathBuf>, Element)>,
}

struct FileEntry {
    file_path: Arc<PathBuf>,
    file_raw_bytes: Vec<u8>,
    file_name: String,
    relative_path: String,
}

fn parse_file_entry(pack_dir: &Path, entry_path: Arc<PathBuf>) -> Result<FileEntry, XMLPackError> {
    let file_name = entry_path
        .as_path()
        .file_stem()
        .ok_or(XMLPackError::FileStemError)?
        .to_str()
        .ok_or(XMLPackError::FileNameError)?
        .to_lowercase();

    let relative_path = entry_path
        .strip_prefix(pack_dir)
        .map_err(|_| XMLPackError::StripPrefixError)?
        .to_str()
        .ok_or(XMLPackError::StripPrefixError)?
        .to_lowercase();
    let mut file_bytes = vec![];
    std::fs::File::open(entry_path.as_path())?.read_to_end(&mut file_bytes)?;
    Ok(FileEntry {
        file_path: entry_path,
        file_raw_bytes: file_bytes,
        file_name,
        relative_path,
    })
}

fn get_file_entries(
    pack_dir: Arc<PathBuf>,
    _warnings: &mut Vec<ErrorWithLocation>,
    errors: &mut Vec<ErrorWithLocation>,
) -> Vec<FileEntry> {
    let mut file_entries = vec![];
    for entry in walkdir::WalkDir::new(pack_dir.as_path()) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(ErrorWithLocation {
                    file_path: pack_dir.clone(),
                    tag: None,
                    error: e.into(),
                });
                continue;
            }
        };
        match entry.metadata() {
            Ok(e) => {
                if e.is_dir() {
                    continue;
                }
            }
            Err(e) => errors.push(ErrorWithLocation {
                file_path: pack_dir.clone(),
                tag: None,
                error: e.into(),
            }),
        }
        let entry_path = Arc::new(entry.path().to_path_buf());
        match parse_file_entry(pack_dir.as_path(), entry_path.clone()) {
            Ok(fe) => {
                file_entries.push(fe);
            }
            Err(e) => errors.push(ErrorWithLocation {
                file_path: entry_path.clone(),
                tag: None,
                error: e,
            }),
        }
    }
    file_entries
}

/// walk over all entries in a directory
/// if xml, we parse element tree and store it. if image or tbin, we parse the descriptions and bytes,
/// validate them before storing into entries
fn get_xml_pack_entries(
    pack_dir: &Path,
    warnings: &mut Vec<ErrorWithLocation>,
    errors: &mut Vec<ErrorWithLocation>,
) -> XmlPackEntries {
    // we use Arc Pathbuf as we will clone it a lot for a lot of errors, so its faster to clone with Arc.
    let pack_buf = Arc::new(pack_dir.to_path_buf());
    let mut entries = XmlPackEntries::default();
    // all the errors we encounter put into one vector
    let file_entries = get_file_entries(pack_buf, warnings, errors);
    // ids to be used.
    let mut image_id = 0_u16;
    let mut tbin_id = 0_u16;

    for fe in file_entries.into_iter() {
        let entry_path = fe.file_path.clone();
        match entry_path
            .as_path()
            .extension()
            .map(|ext| ext.to_str())
            .flatten()
        {
            // collect all xml strings, so that we can deal with them all at once
            Some("xml") => {
                let src_xml = match std::str::from_utf8(&fe.file_raw_bytes) {
                    Ok(s) => s,
                    Err(e) => {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: None,
                            error: e.into(),
                        });
                        continue;
                    }
                };

                let post_xml = super::rapid::rapid_filter(src_xml.to_string());
                match elementtree::Element::from_reader(std::io::Cursor::new(post_xml)) {
                    Ok(ele) => {
                        entries.elements.push((entry_path.clone(), ele));
                    }
                    Err(e) => errors.push(ErrorWithLocation {
                        file_path: entry_path.clone(),
                        tag: None,
                        error: e.into(),
                    }),
                }
            }

            Some("png") => {
                // get the image into memory
                // parse image
                let img = match image::load_from_memory_with_format(
                    &fe.file_raw_bytes,
                    image::ImageFormat::Png,
                ) {
                    Ok(i) => i,
                    Err(e) => {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: None,
                            error: e.into(),
                        });
                        continue;
                    }
                };

                // get ImageDescription
                let desc = ImageDescription {
                    name: fe.file_name,
                    width: img.width(),
                    height: img.height(),
                    source: ImageSrc::FS,
                    ..Default::default()
                };

                // start inserting into maps and increment id
                entries.images.insert(image_id, fe.file_raw_bytes);
                entries.img_desc.insert(image_id, desc);
                entries
                    .img_relative_path_id
                    .insert(fe.relative_path, image_id);
                image_id += 1;
            }
            Some("trl") => {
                if fe.file_raw_bytes.len() <= 12 {
                    errors.push(ErrorWithLocation {
                        file_path: entry_path.clone(),
                        tag: None,
                        error: XMLPackError::TrailBinaryError,
                    });
                    continue;
                }
                let mut version_bytes = [0_u8; 4];
                version_bytes.copy_from_slice(&fe.file_raw_bytes[..4]);
                let mut _version = u32::from_ne_bytes(version_bytes); // optional as we will convert to version 3

                let mut map_id_bytes = [0_u8; 4];
                map_id_bytes.copy_from_slice(&fe.file_raw_bytes[4..8]);
                let map_id = u32::from_ne_bytes(map_id_bytes);

                let nodes: &[[f32; 3]] = bytemuck::cast_slice(&fe.file_raw_bytes[8..]);
                let mut nodes: Vec<[f32; 3]> = nodes.to_vec();
                let position = if let Some(p) = nodes.first().copied() {
                    p
                } else {
                    errors.push(ErrorWithLocation {
                        file_path: entry_path.clone(),
                        tag: None,
                        error: XMLPackError::TrailBinaryError,
                    });
                    continue;
                };
                nodes.iter_mut().for_each(|p| {
                    let n = *p;
                    *p = [n[0] - position[0], n[1] - position[1], n[2] - position[2]];
                });
                let desc = TBinDescription {
                    name: fe.file_name,
                    version: 3,
                };
                entries.tbins.insert(tbin_id, nodes);
                entries.tbin_desc.insert(tbin_id, desc);
                entries
                    .trl_relative_path_id
                    .insert(fe.relative_path.clone(), (tbin_id, position, map_id));
                tbin_id += 1;
            }
            _rest => {
                warnings.push(ErrorWithLocation {
                    file_path: entry_path.clone(),
                    tag: None,
                    error: XMLPackError::UnrecognisedFile,
                });
            }
        }
    }

    entries
}

// fn load_xml_pack_entries(pack_dir: &Path) -> ()
/// loads an xml pack from the given pack directory and lossily converts it into json pack. we will
/// document all the errors too, so that they can either be ignored or fixed. as we are dealing with
/// filesystem, parsing, validating and conversions, there are LOTS or errors that are possible,
/// so this function is really really long.
pub fn xml_to_json_pack(
    pack_dir: &Path,
) -> (FullPack, Vec<ErrorWithLocation>, Vec<ErrorWithLocation>) {
    let mut errors = vec![];
    let mut warnings = vec![];
    let XmlPackEntries {
        mut img_relative_path_id,
        mut trl_relative_path_id,
        img_desc,
        tbin_desc,
        images,
        tbins,
        elements,
    } = get_xml_pack_entries(pack_dir, &mut warnings, &mut errors);
    let mut fullnames_to_catid: BTreeMap<String, u16> = BTreeMap::default();
    let mut catid_templates: BTreeMap<u16, MarkerTemplate> = BTreeMap::default();
    let mut cats: BTreeMap<u16, Cat> = BTreeMap::default();
    let mut cattree: Vec<CatTree> = vec![];
    let mut cat_id = 0_u16;
    for (entry_path, root) in elements.iter() {
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
                &mut warnings,
                &mut errors,
                entry_path.clone(),
            );
        }
    }
    let mut markers: BTreeMap<u32, Marker> = BTreeMap::default();
    let mut trails: BTreeMap<u32, Trail> = BTreeMap::default();

    for (p, root) in elements.iter() {
        for pois in root.children().filter(|c| c.tag().name() == "POIs") {
            parse_markers_trails(
                pois,
                &mut fullnames_to_catid,
                &mut catid_templates,
                &mut img_relative_path_id,
                &mut trl_relative_path_id,
                &mut markers,
                &mut trails,
                p.clone(),
                &mut warnings,
                &mut errors,
            );
        }
    }
    let pack = Pack {
        images_descriptions: img_desc,
        tbins_descriptions: tbin_desc,
        cats,
        cat_tree: cattree,
        markers,
        trails,
        ..Default::default()
    };
    let full_pack = FullPack {
        pack,
        pack_data: PackData { images, tbins },
    };
    (full_pack, warnings, errors)
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
    warnings: &mut Vec<ErrorWithLocation>,
    errors: &mut Vec<ErrorWithLocation>,
    entry_path: Arc<PathBuf>,
) {
    let mut template = parent_template.clone();
    template.override_from_element(ele, warnings, errors, entry_path.clone());
    let mut display_name = String::new();
    let mut is_separator = false;
    let mut enabled = false;
    if let Some(dn) = ele.get_attr("DisplayName") {
        display_name = dn.to_string();
    } else {
        warnings.push(ErrorWithLocation {
            file_path: entry_path.clone(),
            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
            error: XMLPackError::AttributeParseError("missing displayname attribute".to_string()),
        })
    }

    if let Some(issep) = ele.get_attr("IsSeparator=") {
        match issep.parse() {
            Ok(e) => is_separator = e,
            Err(_e) => {
                warnings.push(ErrorWithLocation {
                    file_path: entry_path.clone(),
                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                    error: XMLPackError::AttributeParseError("Is Seperator attribute".to_string()),
                });
            }
        }
    }
    if let Some(e) = ele.get_attr("defaulttoggle") {
        match e {
            "1" | "true" => {
                enabled = true;
            }
            "0" | "false" => {
                enabled = false;
            }
            _ => {
                warnings.push(ErrorWithLocation {
                    file_path: entry_path.clone(),
                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                    error: XMLPackError::AttributeParseError(
                        "Default Toggle attribute".to_string(),
                    ),
                });
            }
        }
    }
    if let Some(name) = ele.get_attr("name") {
        let full_name = if parent_name.is_empty() {
            name.to_string()
        } else {
            parent_name.to_string() + "." + name
        };
        let full_name = full_name.to_lowercase();
        let id = *cat_id;
        fullnames_to_catid.insert(full_name.clone(), id);
        catid_templates.insert(id, template.clone());
        cats.insert(
            id,
            Cat {
                name: name.to_string(),
                display_name,
                is_separator,
                enabled,
                ..Default::default()
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
                warnings,
                errors,
                entry_path.clone(),
            );
        }
        cat_tree.push(CatTree { id, children });
    }
}

impl MarkerTemplate {
    pub fn override_from_element(
        &mut self,
        ele: &Element,
        warnings: &mut Vec<ErrorWithLocation>,
        errors: &mut Vec<ErrorWithLocation>,
        entry_path: Arc<PathBuf>,
    ) {
        for (attr_name, attr_value) in ele.attrs() {
            let attr_value = attr_value.trim();
            match attr_name.name().trim() {
                "achievementId" => {
                    self.achievement_id = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "achievementId attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "achievementBit" => {
                    self.achievement_bit = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "achievementBit attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "alpha" => {
                    self.alpha = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "alpha attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "animSpeed" => {
                    self.anim_speed = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "animSpeed attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "autotrigger" => {
                    self.auto_trigger = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "autoTrigger attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "behavior" => {
                    self.behavior = Some(match serde_json::from_str(attr_value) {
                        Ok(b) => b,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "behavior attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "color" => {
                    let mut color = [0u8; 4];
                    match hex::decode_to_slice(attr_value, &mut color) {
                        Ok(_) => self.color = Some(color),
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "autoTrigger attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    };
                }
                "fadeFar" => {
                    self.fade_far = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "fadeFar attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "fadeNear" => {
                    self.fade_near = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "fadeFar attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "hasCountdown" => {
                    self.has_countdown = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "hasCountdown attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "heightOffset" => {
                    self.height_offset = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "heightOffset attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "inGameVisibility" => {
                    self.in_game_visibility = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "inGameVisibility attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "iconFile" => {
                    self.icon_file = Some(attr_value.to_string());
                }
                "iconSize" => {
                    self.icon_size = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "iconSize attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "keepOnMapEdge" => {
                    self.keep_on_map_edge = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "keepOnMapEdge attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "info" => {
                    self.info = Some(attr_value.to_string());
                }
                "infoRange" => {
                    self.info_range = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "infoRange attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "mapDisplaySize" => {
                    self.map_display_size = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "mapDisplaySize attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "mapFadeoutScaleLevel" => {
                    self.map_fade_out_scale_level = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "mapFadeoutScaleLevel attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }

                "mapVisibility" => {
                    self.map_visibility = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "mapVisibility attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "maxSize" => {
                    self.max_size = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "maxSize attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "minSize" => {
                    self.min_size = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "minSize attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "miniMapVisibility" => {
                    self.mini_map_visibility = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "miniMapVisibility attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "resetLength" => {
                    self.reset_length = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "resetLength attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "resetOffset" => {
                    self.reset_offset = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "resetOffset attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "scaleOnMapWithZoom" => {
                    self.scale_on_map_with_zoom = Some(match attr_value {
                        "1" | "true" => 1,
                        "0" | "false" => 0,
                        _others => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "scaleOnMapWithZoom attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "texture" => {
                    self.texture = Some(attr_value.to_string());
                }
                "toggleCategory" => {
                    self.toggle_cateogry = Some(attr_value.to_string());
                }
                "trailData" => {
                    self.trail_data_file = Some(attr_value.to_string());
                }
                "trailScale" => {
                    self.trail_scale = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "trailScale attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                "triggerRange" | "triggerrange" => {
                    self.trigger_range = Some(match attr_value.parse() {
                        Ok(v) => v,
                        Err(_e) => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "triggerRange attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    });
                }
                rest => match rest {
                    "DisplayName" | "name" | "xpos" | "ypos" | "zpos" | "IsSeparator" | "type"
                    | "GUID" | "MapID" | "copy" | "tip-name" | "tip-description" | "festival"
                    | "copy-message" | "schedule" | "schedule-duration" => {}
                    rest => {
                        if rest.starts_with("bh-") {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "attributes starting with bh- are ignored".to_string(),
                                ),
                            })
                        } else {
                            warnings.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(rest.to_string()),
                            });
                        }
                    }
                },
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_markers_trails(
    ele: &Element,
    fullnames_to_catid: &mut BTreeMap<String, u16>,
    catid_templates: &mut BTreeMap<u16, MarkerTemplate>,
    image_path_id: &mut BTreeMap<String, u16>,
    trail_path_id: &mut BTreeMap<String, (u16, [f32; 3], u32)>,
    markers: &mut BTreeMap<u32, Marker>,
    trails: &mut BTreeMap<u32, Trail>,
    entry_path: Arc<PathBuf>,
    warnings: &mut Vec<ErrorWithLocation>,
    errors: &mut Vec<ErrorWithLocation>,
) {
    if ele.tag().name() == "POIs" {
        for (_, mt) in ele.children().enumerate() {
            match mt.tag().name() {
                "POI" => {
                    if let Some(guid) = mt.get_attr("GUID") {
                        match base64::decode(guid) {
                            Ok(uuid_bytes) => {
                                if let Err(e) = uuid::Uuid::from_slice(&uuid_bytes) {
                                    warnings.push(ErrorWithLocation {
                                        file_path: entry_path.clone(),
                                        tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                        error: XMLPackError::AttributeParseError(format!("decoded uuid bytes are {uuid_bytes:?}. and uuid error: {e:?}")),
                                    })
                                }
                            }
                            Err(e) => {
                                warnings.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::AttributeParseError(format!(
                                        "GUID attribute is not valid base64. {e:?}"
                                    )),
                                });
                            }
                        }
                    } else {
                        warnings.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError(
                                "GUID attribute missing".to_string(),
                            ),
                        });
                    }
                    let mut position = [0_f32; 3];
                    if let Some(xpos) = mt.get_attr("xpos") {
                        match xpos.trim().parse() {
                            Ok(x) => position[0] = x,
                            Err(_e) => {
                                errors.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::AttributeParseError(
                                        "xpos attribute".to_string(),
                                    ),
                                });
                                continue;
                            }
                        }
                    } else {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError("xpos attribute".to_string()),
                        });
                        continue;
                    }
                    if let Some(ypos) = mt.get_attr("ypos") {
                        match ypos.trim().parse() {
                            Ok(y) => position[1] = y,
                            Err(_e) => {
                                errors.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::AttributeParseError(
                                        "ypos attribute".to_string(),
                                    ),
                                });
                                continue;
                            }
                        }
                    } else {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError("ypos attribute".to_string()),
                        });
                        continue;
                    }

                    if let Some(zpos) = mt.get_attr("zpos") {
                        match zpos.trim().parse() {
                            Ok(z) => position[2] = z,
                            Err(_e) => {
                                errors.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::AttributeParseError(
                                        "zpos attribute".to_string(),
                                    ),
                                });
                                continue;
                            }
                        }
                    } else {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError("zpos attribute".to_string()),
                        });
                        continue;
                    }

                    position
                        .iter_mut()
                        .for_each(|meter| *meter *= INCHES_PER_METER);

                    let map_id: u16 = if let Some(map_id) = mt.get_attr("MapID") {
                        match map_id.trim().parse() {
                            Ok(map_id) => map_id,
                            Err(_e) => {
                                errors.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::AttributeParseError(
                                        "MapID attribute".to_string(),
                                    ),
                                });
                                continue;
                            }
                        }
                    } else {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError("MapID attribute".to_string()),
                        });
                        continue;
                    };

                    let cat = if let Some(fullname) = mt.get_attr("type") {
                        if let Some(catid) = fullnames_to_catid.get(&fullname.to_lowercase()) {
                            *catid
                        } else {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(mt.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::CategoryNotFound {
                                    name: fullname.to_string(),
                                },
                            });
                            continue;
                        }
                    } else {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(mt.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError("type attribute".to_string()),
                        });
                        continue;
                    };

                    let mut m = Marker {
                        position,
                        cat,
                        ..Default::default()
                    };
                    let mut template = catid_templates
                        .get(&cat)
                        .expect("missing cat template")
                        .clone();
                    template.override_from_element(mt, warnings, errors, entry_path.clone());

                    m.alpha = template.alpha.map(|a| (a * 255.0) as u8);
                    m.color = template.color;
                    if template.fade_far.is_some() || template.fade_near.is_some() {
                        m.fade_range = Some([
                            template
                                .fade_near
                                .map(|m| m as f32 * INCHES_PER_METER)
                                .unwrap_or(0.0),
                            template
                                .fade_far
                                .map(|m| m as f32 * INCHES_PER_METER)
                                .unwrap_or(f32::MAX),
                        ]);
                    }
                    if template.min_size.is_some() || template.max_size.is_some() {}
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
                        let info = Info {
                            text: info,
                            range: template.info_range.map(|meter| meter * INCHES_PER_METER),
                        };
                        m.dynamic_props.info = Some(info);
                    }
                    if let Some(aid) = template.achievement_id {
                        let achievement = Achievement {
                            id: aid,
                            bit: template.achievement_bit.unwrap_or(u8::MAX),
                        };
                        m.dynamic_props.achievement = Some(achievement);
                    }
                    let range = template.trigger_range.map(|r| r * INCHES_PER_METER);
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
                        m.dynamic_props.trigger = Some(Trigger {
                            range,
                            behavior,
                            toggle_cat,
                        });
                    }
                    if let Some(tex) = template.icon_file {
                        m.texture = image_path_id.get(&tex.to_lowercase()).copied();
                        if m.texture.is_none() {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(mt.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::ImageNotFound { image_path: tex },
                            });
                        }
                    }

                    for marker_id in 0..u16::MAX {
                        let id: u32 = ((map_id as u32) << 16) | (marker_id as u32);
                        if let std::collections::btree_map::Entry::Vacant(e) = markers.entry(id) {
                            e.insert(m);
                            break;
                        }
                        if marker_id == u16::MAX {
                            panic!("markerid ran out of u16 range")
                        }
                    }
                }
                "Trail" => {
                    if let Some(guid) = mt.get_attr("GUID") {
                        match base64::decode(guid) {
                            Ok(uuid_bytes) => {
                                if let Err(e) = uuid::Uuid::from_slice(&uuid_bytes) {
                                    warnings.push(ErrorWithLocation {
                                        file_path: entry_path.clone(),
                                        tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                        error: XMLPackError::AttributeParseError(format!("decoded uuid bytes are {uuid_bytes:?}. and uuid error: {e:?}")),
                                    })
                                }
                            }
                            Err(e) => {
                                warnings.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::AttributeParseError(format!(
                                        "GUID attribute is not valid base64. {e:?}"
                                    )),
                                });
                            }
                        }
                    } else {
                        warnings.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError(
                                "GUID attribute missing".to_string(),
                            ),
                        });
                        continue;
                    }
                    let tdfile = match mt.get_attr("trailData") {
                        Some(e) => e,
                        None => {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::AttributeParseError(
                                    "TrailData attribute".to_string(),
                                ),
                            });
                            continue;
                        }
                    };
                    let (tbin_id, mut tposition, map_id) =
                        match trail_path_id.get(&tdfile.to_lowercase()).copied() {
                            Some(x) => x,
                            None => {
                                errors.push(ErrorWithLocation {
                                    file_path: entry_path.clone(),
                                    tag: Some(ele.attrs().map(|(_, a)| a).join("\n")),
                                    error: XMLPackError::TrlNotFound {
                                        trl_path: tdfile.to_string(),
                                    },
                                });
                                continue;
                            }
                        };
                    let cat = if let Some(fullname) = mt.get_attr("type") {
                        if let Some(catid) = fullnames_to_catid.get(&fullname.to_lowercase()) {
                            *catid
                        } else {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(mt.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::CategoryNotFound {
                                    name: fullname.to_string(),
                                },
                            });
                            continue;
                        }
                    } else {
                        errors.push(ErrorWithLocation {
                            file_path: entry_path.clone(),
                            tag: Some(mt.attrs().map(|(_, a)| a).join("\n")),
                            error: XMLPackError::AttributeParseError("type attribute".to_string()),
                        });
                        continue;
                    };
                    tposition
                        .iter_mut()
                        .for_each(|meter| *meter *= INCHES_PER_METER);
                    let mut m = Trail {
                        pos: tposition,
                        tbin: tbin_id,
                        cat,
                        ..Default::default()
                    };
                    let mut template = catid_templates
                        .get(&cat)
                        .expect("missing cat template")
                        .clone();

                    template.override_from_element(mt, warnings, errors, entry_path.clone());

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

                    if let Some(tex) = template.texture {
                        m.texture = image_path_id.get(&tex.to_lowercase()).copied();
                        if m.texture.is_none() {
                            errors.push(ErrorWithLocation {
                                file_path: entry_path.clone(),
                                tag: Some(mt.attrs().map(|(_, a)| a).join("\n")),
                                error: XMLPackError::ImageNotFound { image_path: tex },
                            });
                        }
                    }

                    for trail_id in 0..u16::MAX {
                        let id: u32 = (map_id << 16) | (trail_id as u32);
                        if let std::collections::btree_map::Entry::Vacant(e) = trails.entry(id) {
                            e.insert(m);
                            break;
                        }
                        if trail_id == u16::MAX {
                            panic!("trailid ran out of u16 range")
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct ErrorWithLocation {
    pub file_path: Arc<PathBuf>,
    pub tag: Option<String>,
    pub error: XMLPackError,
}

#[derive(Debug, thiserror::Error)]
pub enum XMLPackError {
    #[error("io errors")]
    IOError(#[from] std::io::Error),
    #[error("io errors")]
    DirEntryError(#[from] walkdir::Error),
    #[error("failed to convert filename to utf-8 str. ")]
    FileNameError,
    #[error("file with no extension.")]
    UnrecognisedFile,
    #[error("strip prefix error.")]
    StripPrefixError,
    #[error("file stem error ")]
    FileStemError,
    #[error("invalid png image")]
    InvalidPngImage(#[from] image::ImageError),
    #[error("too large png image")]
    ImageTooLarge,
    #[error("invalid trl binary")]
    TrailBinaryError,
    #[error("file does not contain valid utf-8")]
    UTF8Error(#[from] std::str::Utf8Error),
    #[error("file does not contain valid xml")]
    XMLParseError(#[from] elementtree::Error),
    #[error("referenced category not found")]
    CategoryNotFound { name: String },
    #[error("referenced image not found")]
    ImageNotFound { image_path: String },
    #[error("referenced trl binary not found")]
    TrlNotFound { trl_path: String },
    #[error("unknown attribute {0}")]
    UnknownAttribute(String),
    #[error("unknown tag {0}")]
    UnknownTag(String),
    #[error("Error occured when parsing attribute: {0}")]
    AttributeParseError(String),
}
