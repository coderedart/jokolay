// use std::{
//     ffi::OsString,
//     path::{Path, PathBuf},
//     sync::Arc, collections::BTreeMap,
// };

// use image::GenericImageView;
// use serde::{Deserialize, Serialize};
// use serde_with::{serde_as, skip_serializing_none};
// use tokio_stream::StreamExt;
// use uuid::Uuid;

// use crate::{
//     json::{
//         category::{Cat, CatTree},
//         marker::{Achievement, Behavior, Dynamic, Info, Marker, Trigger},
//         trail::Trail,
//         *,
//     },
//     xmlpack::{
//         xml_category::{parse_join_mc, XMLMarkerCategory},
//         xml_file::{OverlayData, XmlFile},
//         xml_trail::{TrailData, TrailDataDeserializeError},
//         MarkerTemplate,
//     },
// };

// /// The struct to represent a deserialized XML Marker Pack. We can use this intermediate representation to validate the pack.
// /// NOTE: relative paths will be converted to lowercase because windows is case-insensitive.
// #[derive(Debug, Clone)]
// pub struct XmlPackEntries {
//     /// images with their relative path as keys, and the values are width, height, full path, name, hash
//     pub images: BTreeMap<String, XmlPackImageEntry>,
//     /// xml file contents that have the OverlayData tag as root
//     pub xml_files: BTreeMap<PathBuf, XmlFile>,
//     /// Trl files
//     pub trl_files: BTreeMap<String, TrailData>,
//     pub unrecognized_files: Vec<PathBuf>,
// }
// #[derive(Debug, Clone)]
// pub struct XmlPackImageEntry {
//     pub width: u16,
//     pub height: u16,
//     pub path: PathBuf,
//     pub name: String,
// }
// impl XmlPackEntries {
//     /// creates a new pack from the folder given. on hard errors, it will just return the error. fix all errors and then try again.
//     pub async fn new(pack_folder: &Path) -> (XmlPackEntries, Vec<XmlPackLoadError>) {
//         let mut contents = async_walkdir::WalkDir::new(pack_folder);
//         let mut images = BTreeMap::default();
//         let mut xml_files = BTreeMap::default();
//         let mut trl_files = BTreeMap::default();
//         let mut unrecognized_files = vec![];
//         let mut errors = vec![];
//         // for each entry in the marker pack folder, check the file type and put the contents into a respective containers
//         while let Some(entry) = contents.next().await {
//             let entry = entry.expect("could not get dir entry");
//             let entry_type = entry.file_type().await.expect("could not get file type");
//             if entry_type.is_file() {
//                 let ext = match entry
//                     .path()
//                     .extension()
//                     .unwrap_or_default() // give empty extension so that it goes into a
//                     .to_os_string()
//                     .into_string()
//                 {
//                     Ok(ext) => ext,
//                     Err(e) => {
//                         errors.push(XmlPackLoadError::FileNameError(e));
//                         continue;
//                     }
//                 };
//                 let entry_path = entry.path();
//                 let relative_entry_path = match {
//                     match entry_path.strip_prefix(pack_folder) {
//                         Ok(p) => p,
//                         Err(_) => {
//                             errors.push(XmlPackLoadError::StripPrefixError(
//                                 entry_path,
//                                 pack_folder.to_path_buf(),
//                             ));
//                             continue;
//                         }
//                     }
//                     .as_os_str()
//                     .to_os_string()
//                     .into_string()
//                 } {
//                     Ok(rep) => rep,
//                     Err(e) => {
//                         errors.push(XmlPackLoadError::FileNameError(e));
//                         continue;
//                     }
//                 }
//                 .to_lowercase();
//                 match ext.as_str() {
//                     "xml" => {
//                         let xml_string = tokio::fs::read_to_string(&entry_path)
//                             .await
//                             .expect("failed to read file to string");
//                         match roxmltree::Document::parse(&xml_string) {
//                             Ok(_) => {}
//                             Err(e) => {
//                                 errors.push(XmlPackLoadError::XmlErrors {
//                                     roxml_error: e,
//                                     file_path: entry_path.clone(),
//                                 });
//                                 continue;
//                             }
//                         };
//                         let reader = std::io::Cursor::new(&xml_string);
//                         let deserializer = &mut quick_xml::de::Deserializer::from_reader(reader);

//                         let od: OverlayData = match serde_path_to_error::deserialize(deserializer) {
//                             Ok(od) => od,
//                             Err(e) => {
//                                 errors.push(XmlPackLoadError::DeError {
//                                     file_path: entry_path.clone(),
//                                     de_err: e,
//                                 });
//                                 continue;
//                             }
//                         };

//                         xml_files.insert(
//                             entry_path.clone(),
//                             XmlFile {
//                                 path: entry_path.clone(),
//                                 od,
//                             },
//                         );
//                     }
//                     "png" => {
//                         let image_file = tokio::fs::read(entry_path.clone())
//                             .await
//                             .expect("failed to read file");
//                         let img = match image::load_from_memory_with_format(
//                             &image_file,
//                             image::ImageFormat::Png,
//                         ) {
//                             Ok(i) => i,
//                             Err(e) => {
//                                 errors.push(XmlPackLoadError::InvalidPngImage {
//                                     file_path: entry_path.clone(),
//                                     image_err: e,
//                                 });
//                                 continue;
//                             }
//                         };
//                         // let hash =
//                         let name: String = match entry_path.file_stem() {
//                             Some(stem) => stem,
//                             None => {
//                                 errors.push(XmlPackLoadError::FileStemError(entry_path));
//                                 continue;
//                             }
//                         }
//                         .to_string_lossy()
//                         .to_string();
//                         let path = entry_path.to_path_buf();
//                         images.insert(
//                             relative_entry_path,
//                             XmlPackImageEntry {
//                                 width: img.width() as u16,
//                                 height: img.height() as u16,
//                                 path,
//                                 name,
//                             },
//                         );
//                     }
//                     "trl" => {
//                         let t_buffer = tokio::fs::read(entry_path.clone())
//                             .await
//                             .expect("failed to read file");
//                         let trail_data = match TrailData::parse_from_bytes(&t_buffer) {
//                             Ok(td) => td,
//                             Err(e) => {
//                                 errors.push(XmlPackLoadError::TrailDeError {
//                                     file_path: entry_path.clone(),
//                                     t_err: e,
//                                 });
//                                 continue;
//                             }
//                         };
//                         trl_files.insert(relative_entry_path, trail_data);
//                     }
//                     _ => unrecognized_files.push(entry_path),
//                 }
//             }
//         }
//         (
//             XmlPackEntries {
//                 images,
//                 xml_files,
//                 trl_files,
//                 unrecognized_files,
//             },
//             errors,
//         )
//     }
//     /// This will validate stuff like categories not found, images/trlfiles not found, or duplicate UUIDs.
//     /// this will collect all the errors as these are mostly soft errors and then return all of them at once.
//     /// if this succeeds, we can probably convert it into a json pack.
//     pub fn validate_pack(&mut self) -> Vec<XmlPackValidationErrors> {
//         let mut validation_errors: Vec<XmlPackValidationErrors> = vec![];
//         let mut cats: BTreeMap<String, XMLMarkerCategory> = BTreeMap::default();
//         let mut id_set: BTreeMap<Uuid, Arc<PathBuf>> = BTreeMap::default();

//         for (_p, xf) in self.xml_files.iter() {
//             if let Some(ref mc) = xf.od.categories {
//                 parse_join_mc(mc.clone(), "", &mut cats);
//             }
//         }
//         for (_p, xf) in self.xml_files.iter_mut() {
//             let xf_path = Arc::new(xf.path.clone());
//             if let Some(ref mut pois) = xf.od.pois {
//                 if let Some(ref mut tags) = pois.tags {
//                     for pt in tags {
//                         match pt {
//                             poi @ crate::xmlpack::xml_marker::PoiOrTrail::POI { .. } => {
//                                 let p: super::xml_marker::POI = (&*poi).into();
//                                 if !cats.contains_key(&p.category) {
//                                     validation_errors.push(
//                                         XmlPackValidationErrors::CategoryNotFound {
//                                             file_path: xf_path.clone(),
//                                             id: p.guid.map(|id| base64::encode(id.as_bytes())),
//                                             category_name: p.category.clone(),
//                                         },
//                                     );
//                                 }
//                                 if let Some(ref icon_path) = p.icon_file {
//                                     if !self.images.contains_key(&icon_path.to_lowercase()) {
//                                         validation_errors.push(
//                                             XmlPackValidationErrors::ImageNotFound {
//                                                 file_path: xf_path.clone(),
//                                                 id: p.guid,
//                                                 image_path: icon_path.clone(),
//                                             },
//                                         )
//                                     }
//                                 }
//                                 if let Some(id) = p.guid {
//                                     if let Some(previous_file) = id_set.insert(id, xf_path.clone())
//                                     {
//                                         if let crate::xmlpack::xml_marker::PoiOrTrail::POI {
//                                             guid,
//                                             ..
//                                         } = poi
//                                         {
//                                             *guid = Some(Uuid::new_v4());
//                                         }
//                                         let id = base64::encode(id.as_bytes());
//                                         validation_errors.push(
//                                             XmlPackValidationErrors::DuplicateUUID {
//                                                 id,
//                                                 first_file_path: previous_file,
//                                                 second_file_path: xf_path.clone(),
//                                             },
//                                         )
//                                     }
//                                 }
//                             }
//                             trail @ crate::xmlpack::xml_marker::PoiOrTrail::Trail { .. } => {
//                                 let t: super::xml_trail::Trail = (&*trail).into();
//                                 if !cats.contains_key(&t.category) {
//                                     validation_errors.push(
//                                         XmlPackValidationErrors::CategoryNotFound {
//                                             file_path: xf_path.clone(),
//                                             id: t.guid.map(|id| base64::encode(id.as_bytes())),
//                                             category_name: t.category.clone(),
//                                         },
//                                     );
//                                 }
//                                 if let Some(ref icon_path) = t.texture {
//                                     if !self.images.contains_key(&icon_path.to_lowercase()) {
//                                         validation_errors.push(
//                                             XmlPackValidationErrors::ImageNotFound {
//                                                 file_path: xf_path.clone(),
//                                                 id: t.guid,
//                                                 image_path: icon_path.clone(),
//                                             },
//                                         )
//                                     }
//                                 }
//                                 if !self
//                                     .trl_files
//                                     .contains_key(&t.trail_data_file.to_lowercase())
//                                 {
//                                     validation_errors.push(XmlPackValidationErrors::TrlNotFound {
//                                         file_path: xf_path.clone(),
//                                         id: t.guid,
//                                         trl_path: t.trail_data_file.clone(),
//                                     })
//                                 }

//                                 if let Some(id) = t.guid {
//                                     if let Some(previous_path) = id_set.insert(id, xf_path.clone())
//                                     {
//                                         if let crate::xmlpack::xml_marker::PoiOrTrail::Trail {
//                                             guid,
//                                             ..
//                                         } = trail
//                                         {
//                                             *guid = Some(Uuid::new_v4());
//                                         }
//                                         let id = base64::encode(id.as_bytes());
//                                         validation_errors.push(
//                                             XmlPackValidationErrors::DuplicateUUID {
//                                                 id,
//                                                 first_file_path: previous_path,
//                                                 second_file_path: xf_path.clone(),
//                                             },
//                                         )
//                                     }
//                                 }
//                             }
//                             route @ crate::xmlpack::xml_marker::PoiOrTrail::Route { .. } => {
//                                 log::warn!("ignoring a route. {:?} in file: {:?}", route, &xf_path);
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         validation_errors
//     }
//     // pub fn to_json_pack(
//     //     mut self,
//     //     status_sender: flume::Sender<ToJsonPackStatus>,
//     //     enabled_image_compression: bool,
//     // ) {
//     //     if !self.validate_pack().is_empty() {

//     //         // return None;
//     //     }
//     //     let _ = status_sender.send(ToJsonPackStatus::Started);
//     //     // let mut jpack = SinglePack::default();
//     //     // // give new id for a new pack
//     //     // jpack.pack_description.id = Uuid::new_v4();
//     //     let mut jpack_cats: BTreeMap<CategoryID, CatDescription> = BTreeMap::default();
//     //     let mut jpack_trls = BTreeMap::default();
//     //     let mut jpack_trl_descriptions = BTreeMap::default();
//     //     let mut jpack_image_descriptions = BTreeMap::default();
//     //     let mut jpack_images = BTreeMap::default();
//     //     // templates which will be used by markers to inherit from. templates will inherit attributes from parent cats.
//     //     let mut templates = BTreeMap::default();
//     //     // map which will have the id of the category referenced by its "full name"
//     //     let mut names_id_map = BTreeMap::default();
//     //     // A Cat Selection Tree which will be constructed from the hierarchial xml categories.
//     //     let mut cat_selection_tree = vec![];
//     //     // for progress
//     //     let total_files_count = self.xml_files.len();
//     //     // iterate through all xml files and convert all the categories
//     //     self.xml_files
//     //         .iter()
//     //         .filter_map(|(_, xf)| xf.od.categories.as_ref())
//     //         .enumerate()
//     //         .for_each(|(index, xcat)| {
//     //             let _ = status_sender.send(ToJsonPackStatus::ProcessingCategories(
//     //                 index as u32,
//     //                 total_files_count as u32,
//     //             ));
//     //             Self::insert_cat_recursive_json_pack(
//     //                 &mut jpack_cats,
//     //                 &xcat.clone(),
//     //                 "",
//     //                 &MarkerTemplate::default(),
//     //                 &mut templates,
//     //                 &mut names_id_map,
//     //                 &mut cat_selection_tree,
//     //             )
//     //         });
//     //     // iterate again, but this time, we fill up markers/trails into the categories that we prepared in the first iteration
//     //     self.xml_files
//     //         .iter()
//     //         .enumerate()
//     //         .filter_map(|(index, (_, xf))|  xf.od.pois.as_ref().map(|pois| (index, pois)))
//     //         .filter_map(|(index, pois)|  pois.tags.as_ref().map(|tags| (index, tags)))
//     //         .for_each(|(index, tags)| {
//     //             let _ = status_sender.send(ToJsonPackStatus::ProcessingMarkers(index as u32, total_files_count as u32));
//     //             for pt in tags {
//     //                 match pt {
//     //                     poi @ super::xml_marker::PoiOrTrail::POI { ..} => {
//     //                         let xp: super::xml_marker::POI = poi.into();
//     //                         let mut xp = xp.clone();
//     //                         if let Some(&cat_id) = names_id_map.get(&xp.category) {

//     //                             let template = templates.get(&cat_id).cloned().unwrap_or_default();
//     //                             xp.inherit_if_none(&template);
//     //                             let mut m = Marker{
//     //                                 id:  xp.guid.map(|guid| if guid.is_nil() {Uuid::new_v4()} else {guid}).unwrap_or_else(Uuid::new_v4).into(),
//     //                                 position: [xp.xpos, xp.ypos, xp.zpos],
//     //                             alpha : xp.alpha,
//     //                             color : xp.color,
//     //                             in_game_visibility: xp.in_game_visibility,
//     //                             keep_on_map_edge: xp.keep_on_map_edge,
//     //                             map_display_size: xp.map_display_size.map(|mds| (mds as u16)),
//     //                             map_fade_out_scale_level: xp.map_fade_out_scale_level,
//     //                             map_visibility: xp.map_visibility,
//     //                             max_size: xp.max_size.map(|ms| ms as u16),
//     //                             mini_map_visibility: xp.mini_map_visibility,
//     //                             scale : xp.icon_size,
//     //                             scale_on_map_with_zoom: xp.scale_on_map_with_zoom,
//     //                                 ..Default::default()
//     //                             };
//     //                             if let Some(a_id) = xp.achievement_id {
//     //                                 m.achievement = Some(Achievement {
//     //                                     id: a_id,
//     //                                     bit: xp.achievement_bit,
//     //                                 })
//     //                             }
//     //                             if let Some(fade_near) = xp.fade_near {
//     //                                 m.fade_range = Some([
//     //                                     fade_near as f32,
//     //                                     xp.fade_far.unwrap_or(fade_near + 100) as f32,
//     //                                 ]);
//     //                             }
//     //                             if let Some(icon_file) = xp.icon_file {
//     //                                 if let Some(image_entry) = self.images.get(&icon_file) {
//     //                                     m.image = Some(image_entry.hash);
//     //                                 }
//     //                             }

//     //                             if let Some(min_size) = xp.min_size {
//     //                                 m.min_size = Some(min_size as u16);
//     //                             }
//     //                             // start dynamic part
//     //                             if xp.info.is_some() || xp.trigger_range.is_some() {
//     //                                 m.dynamic_props = Some(Dynamic {
//     //                                     trigger: {
//     //                                         if xp.trigger_range.is_some() || xp.auto_trigger.is_some() || xp.has_countdown.is_some() || xp.behavior.is_some() || xp.toggle_cateogry.is_some() {
//     //                                             Some(Trigger {
//     //                                                 auto_trigger: xp.auto_trigger,
//     //                                                 count_down: xp.has_countdown,
//     //                                                 range: xp.trigger_range.unwrap_or(10.0),
//     //                                                 behavior: xp.behavior.map(|b| {
//     //                                                     match b {
//     //                                                         crate::xmlpack::xml_marker::Behavior::AlwaysVisible => Behavior::AlwaysVisible,
//     //                                                         crate::xmlpack::xml_marker::Behavior::ReappearOnMapChange => Behavior::ReappearOnMapChange,
//     //                                                         crate::xmlpack::xml_marker::Behavior::ReappearOnDailyReset => Behavior::ReappearOnDailyReset,
//     //                                                         crate::xmlpack::xml_marker::Behavior::OnlyVisibleBeforeActivation => Behavior::OnlyVisibleBeforeActivation,
//     //                                                         crate::xmlpack::xml_marker::Behavior::ReappearAfterTimer => Behavior::ReappearAfterTimer {
//     //                                                             reset_length: xp.reset_length.unwrap_or(10),
//     //                                                         },
//     //                                                         crate::xmlpack::xml_marker::Behavior::ReappearOnMapReset => Behavior::ReappearOnMapReset {
//     //                                                             map_cycle_length: 3600,
//     //                                                             map_cycle_offset_after_reset: 0,
//     //                                                         },
//     //                                                         crate::xmlpack::xml_marker::Behavior::OncePerInstance => Behavior::OncePerInstance,
//     //                                                         crate::xmlpack::xml_marker::Behavior::DailyPerChar => Behavior::DailyPerChar,
//     //                                                         crate::xmlpack::xml_marker::Behavior::OncePerInstancePerChar => Behavior::OncePerInstancePerChar,
//     //                                                         crate::xmlpack::xml_marker::Behavior::WvWObjective => Behavior::WvWObjective,
//     //                                                     }
//     //                                                 }),
//     //                                                 toggle_cat: if let Some(c) = xp.toggle_cateogry {
//     //                                                     names_id_map.get(&c).copied()
//     //                                                 } else {
//     //                                                     None
//     //                                                 },
//     //                                             })
//     //                                         } else {
//     //                                             None
//     //                                         }
//     //                                     },
//     //                                     info: {
//     //                                         if xp.info.is_some() {
//     //                                             Some(Info {
//     //                                                 text: xp.info.unwrap_or_default(),
//     //                                                 range: xp.info_range.unwrap_or(100.0),
//     //                                             })
//     //                                         } else {
//     //                                             None
//     //                                         }
//     //                                     }
//     //                                 })
//     //                             }
//     //                             let cat =  jpack_cats.get_mut(&cat_id).unwrap();
//     //                             let map_id = xp.map_id.into();
//     //                             if !cat.map_markers.contains_key(&map_id) {
//     //                                 cat.map_markers.insert(map_id, Default::default());

//     //                             };
//     //                             let map_markers = &mut cat.map_markers.get_mut(&map_id).unwrap().markers;
//     //                             if let Some(prev_marker) = map_markers.get(&m.id) {
//     //                                 log::warn!("prev marker and current marker have same id: {:?} {:?}", prev_marker, &m);
//     //                                 m.id = Uuid::new_v4().into();
//     //                             }
//     //                             map_markers.insert(m.id, m);

//     //                         }
//     //                     }
//     //                     trail @ super::xml_marker::PoiOrTrail::Trail{..} => {

//     //                         let xt: super::xml_trail::Trail = trail.into();
//     //                         let mut xt = xt.clone();
//     //                         if let Some(&cat_id) = names_id_map.get(&xt.category) {
//     //                             let template = templates.get(&cat_id).cloned().unwrap_or_default();
//     //                             xt.inherit_if_none(&template);
//     //                             let mut t = Trail {
//     //                                 id: xt.guid.map(|guid| if guid.is_nil() {Uuid::new_v4()} else {guid}).unwrap_or_else(Uuid::new_v4).into(),
//     //                                 alpha: xt.alpha,
//     //                                 anim_speed: xt.anim_speed,
//     //                                 color: xt.color,
//     //                                 scale : xt.trail_scale,
//     //                                 ..Default::default()
//     //                             };
//     //                             if let Some(fade_near) = xt.fade_near {
//     //                                 t.fade_range = Some([fade_near as f32, xt.fade_far.unwrap_or(fade_near + 30) as f32]);
//     //                             }
//     //                             if let Some(texture_name) = xt.texture {
//     //                                 if let Some( image_entry) = self.images.get(&texture_name) {
//     //                                     t.image = Some(image_entry.hash);
//     //                                 }
//     //                             }
//     //                             if let Some(data) = self.trl_files.get(&xt.trail_data_file) {
//     //                                 let hash = xxhash_rust::xxh3::xxh3_64(bytemuck::cast_slice(&data.nodes));
//     //                                 let td = crate::json::trail::TBinDescription {
//     //                                     name: std::path::Path::new(&xt.trail_data_file).file_stem().unwrap_or_default().to_string_lossy().to_string(),
//     //                                     map_id: (data.map_id as u16).into(),

//     //                                 };
//     //                                 let map_id = td.map_id;

//     //                                 t.tbin = hash.into();
//     //                                 let hash = t.tbin;
//     //                                 jpack_trls.insert(hash, data.nodes.clone());
//     //                                 jpack_trl_descriptions.insert(hash, td);
//     //                                 let cat =  jpack_cats.get_mut(&cat_id).unwrap();

//     //                                 if !cat.map_markers.contains_key(&map_id) {
//     //                                     cat.map_markers.insert(map_id, Default::default());
//     //                                 };
//     //                                 let map_markers = cat.map_markers.get_mut(&map_id).unwrap();
//     //                             if let Some(prev_trail) = map_markers.trails.get(&t.id) {
//     //                                 log::warn!("prev trail {:?} and current trail {:?} have the same id", prev_trail, &t);
//     //                                 t.id = Uuid::new_v4().into();
//     //                             }
//     //                             map_markers.trails.insert(t.id, t);
//     //                             }
//     //                         }
//     //                     }
//     //                     super::xml_marker::PoiOrTrail::Route { ..} => todo!(),
//     //                 }
//     //             }
//     //         });
//     //     let total = self.images.len();
//     //     for (index, (_, image_entry)) in self.images.iter().enumerate() {
//     //         let _ = status_sender.send(ToJsonPackStatus::ProcessingImages(
//     //             index as u32,
//     //             total as u32,
//     //         ));
//     //         let ibytes = std::fs::read(&image_entry.path).unwrap();
//     //         let opt = oxipng::Options::max_compression();
//     //         let comp_bytes = if enabled_image_compression {
//     //             oxipng::optimize_from_memory(&ibytes, &opt).unwrap()
//     //         } else {
//     //             ibytes
//     //         };

//     //         // let comp_bytes = ibytes;
//     //         jpack_image_descriptions.insert(
//     //             image_entry.hash,
//     //             super::super::json::pack::ImageDescription {
//     //                 name: image_entry.name.clone(),
//     //                 width: image_entry.width,
//     //                 height: image_entry.height,
//     //             },
//     //         );
//     //         jpack_images.insert(image_entry.hash, comp_bytes);
//     //     }
//     //     let _ = status_sender.send(ToJsonPackStatus::Completed(Box::new(SinglePack {
//     //         pack: JsonPack {
//     //             pack_description: PackDescription {
//     //                 id: Uuid::new_v4().into(),
//     //                 ..Default::default()
//     //             },
//     //             images_descriptions: jpack_image_descriptions,
//     //             tbins_descriptions: jpack_trl_descriptions,
//     //             cattree: cat_selection_tree,
//     //             cats: jpack_cats,
//     //         },
//     //         pack_data: PackData {
//     //             images: jpack_images,
//     //             tbins: jpack_trls,
//     //         },
//     //     })));
//     // }
//     // / The basic idea is that we first collect the categories into the jpack cats, give them an id and put their full name into id_names map,
//     // / then, we just insert the inherited template into the templates map. finally, as we recurse through the xml hierarchy, we will keep inserting
//     // / into the cat selection tree the ids of the categories.
//     // / jcats are the global categories of a json pack.
//     // / xcats are the categories at a certain lvl in the tree, and we recurse this function for each xc in xcat that has children xcats
//     // / prefix/parent_template for inheritance and names_id map so that we can use the map for finding which category a trail/poi goes into later
//     // / cat_selection_tree represents the `Tree` of category selection ui so to speak. and also serves as the representation of we will convert into xml categories
//     //
//     //     fn insert_cat_recursive_json_pack(
//     //         jcats: &mut BTreeMap<CategoryID, JsonCat>,
//     //         xcats: &[super::xml_category::XMLMarkerCategory],
//     //         xml_prefix: &str,
//     //         parent_template: &MarkerTemplate,
//     //         templates: &mut BTreeMap<CategoryID, MarkerTemplate>,
//     //         names_id_map: &mut BTreeMap<String, CategoryID>,
//     //         cat_selection_tree: &mut Vec<CatTree>,
//     //     ) {
//     //         for xc in xcats {
//     //             let id = cat_selection_tree
//     //                 .iter_mut()
//     //                 .find_map(|cst| (cst.name == xc.display_name).then(|| cst.id));
//     //             if let Some(id) = id {
//     //                 if !cat_selection_tree.iter().any(|cst| cst.id == id) {
//     //                     dbg!(jcats.get(&id), &cat_selection_tree);
//     //                 }
//     //             }
//     //             let id = id.unwrap_or_else(|| {
//     //                 let id = Uuid::new_v4().into();
//     //                 let mut cat = JsonCat::default();
//     //                 cat.cat_description.id = id;
//     //                 if let Some(prev) = jcats.insert(id, cat) {
//     //                     panic!("{:?}", prev);
//     //                 }
//     //                 cat_selection_tree.push(CatTree {
//     //                     name: xc.display_name.clone(),
//     //                     id,
//     //                     children: vec![],
//     //                 });
//     //                 id
//     //             });
//     //             let jc = jcats.get_mut(&id).unwrap();
//     //             jc.cat_description.name = xc.name.clone();
//     //             jc.cat_description.display_name = xc.display_name.clone();
//     //             jc.cat_description.is_separator = xc.is_separator;

//     //             let full_name = if xml_prefix.is_empty() {
//     //                 xc.name.clone()
//     //             } else {
//     //                 xml_prefix.to_string() + "." + &xc.name
//     //             };

//     //             let mut mt = MarkerTemplate::new(xc);
//     //             mt.inherit_from_template(parent_template);

//     //             templates.insert(id, mt.clone());
//     //             names_id_map.insert(full_name.clone(), id);
//     //             if let Some(children) = xc.children.as_ref() {
//     //                 let cst_children = match cat_selection_tree.iter_mut().find(|cst| cst.id == id) {
//     //                     Some(c) => &mut c.children,
//     //                     None => panic!(
//     //                         "{}, {:?}, {:?}, {:#?}, {:#?}",
//     //                         &full_name,
//     //                         cat_selection_tree,
//     //                         &id,
//     //                         &jcats.get(&id),
//     //                         &names_id_map.get(&full_name)
//     //                     ),
//     //                 };

//     //                 Self::insert_cat_recursive_json_pack(
//     //                     jcats,
//     //                     children,
//     //                     &full_name,
//     //                     &mt,
//     //                     templates,
//     //                     names_id_map,
//     //                     cst_children,
//     //                 );
//     //             }
//     //         }
//     //     }
// }

// #[serde_as]
// #[skip_serializing_none]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum ToJsonPackStatus {
//     Started,
//     ProcessingCategories(u32, u32),
//     ProcessingMarkers(u32, u32),
//     ProcessingImages(u32, u32),
//     Completed(Box<Pack>),
// }

// pub fn validate_path(relative_path: &str, pack_path: &Path) -> Result<bool, std::io::Error> {
//     let absolute_path = pack_path.join(relative_path);
//     let absolute_path = absolute_path.canonicalize()?;
//     Ok(absolute_path.starts_with(pack_path))
// }
