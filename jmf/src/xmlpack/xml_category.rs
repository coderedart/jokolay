// Marker Category tag in xml files
// acts as a template for markers to inherit from when there's a common property to all the markers under that category/subcatagories.
// #[serde_as]
// #[skip_serializing_none]
// #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
// pub struct XMLMarkerCategory {
//      /// this is what will be shown in the user facing menu when selecting to enable/disable this Category of markers to draw.
//      #[serde(rename = "DisplayName")]
//      pub display_name: String,
//      /// If it's value is 1, the category will act as a separator in the marker category filter and won't have an [x] toggle
//     #[serde(rename = "IsSeparator")]
//     pub is_separator: Option<u8>,
//     /// name will be how we merge/check for consistencies when they are declared in multiple Marker Files and we try to merge them all into a Category Selection Tree.
//     pub name: String,
//     /// These are all the direct sub categories
//     #[serde(rename = "MarkerCategory")]
//     pub children: Vec<XMLMarkerCategory>,
// }

// impl XMLMarkerCategory {
//     // pub fn inherit_if_none(&mut self, other: &XMLMarkerCategory) {
//     //     self.name = other.name.clone() + "." + &self.name;
//     //     if self.map_display_size.is_none() {
//     //         self.map_display_size = other.map_display_size;
//     //     }
//     //     if self.icon_file.is_none() {
//     //         self.icon_file = other.icon_file.clone();
//     //     }
//     //     if self.icon_size.is_none() {
//     //         self.icon_size = other.icon_size;
//     //     }
//     //     if self.alpha.is_none() {
//     //         self.alpha = other.alpha;
//     //     }
//     //     if self.behavior.is_none() {
//     //         self.behavior = other.behavior;
//     //     }
//     //     if self.height_offset.is_none() {
//     //         self.height_offset = other.height_offset;
//     //     }
//     //     if self.fade_near.is_none() {
//     //         self.fade_near = other.fade_near;
//     //     }
//     //     if self.fade_far.is_none() {
//     //         self.fade_far = other.fade_far;
//     //     }
//     //     if self.min_size.is_none() {
//     //         self.min_size = other.min_size;
//     //     }
//     //     if self.max_size.is_none() {
//     //         self.max_size = other.max_size;
//     //     }
//     //     if self.reset_length.is_none() {
//     //         self.reset_length = other.reset_length;
//     //     }
//     //     if self.color.is_none() {
//     //         self.color = other.color;
//     //     }
//     //     if self.auto_trigger.is_none() {
//     //         self.auto_trigger = other.auto_trigger;
//     //     }
//     //     if self.has_countdown.is_none() {
//     //         self.has_countdown = other.has_countdown;
//     //     }
//     //     if self.trigger_range.is_none() {
//     //         self.trigger_range = other.trigger_range;
//     //     }
//     //     if self.achievement_id.is_none() {
//     //         self.achievement_id = other.achievement_id;
//     //     }
//     //     if self.achievement_bit.is_none() {
//     //         self.achievement_bit = other.achievement_bit;
//     //     }
//     //     if self.info.is_none() {
//     //         self.info = other.info.clone();
//     //     }
//     //     if self.info_range.is_none() {
//     //         self.info_range = other.info_range;
//     //     }
//     //     if self.map_visibility.is_none() {
//     //         self.map_visibility = other.map_visibility;
//     //     }
//     //     if self.mini_map_visibility.is_none() {
//     //         self.mini_map_visibility = other.mini_map_visibility;
//     //     }
//     // }
// }

// This just recursively goes through the marker categories and their children to insert them into the global_mc map with its fullname as would be
// referred in the category attribute of a POI/Trail tag. this doesn't validate anything at all.
// pub fn parse_join_mc(
//     cats: Vec<XMLMarkerCategory>,
//     prefix: &str,
//     global_mc: &mut BTreeMap<String, XMLMarkerCategory>,
// ) {
//     for mut mc in cats {
//         // this checks if this is the root/top of the category tree.
//         let name = if !prefix.is_empty() {
//             // if this is a child, take the parent's full name and attach its own name after the pullstop to make its fullname
//             prefix.to_string() + "." + &mc.name
//         } else {
//             // if this is the root, its own name is the fullname
//             mc.name.clone()
//         };
//         // take the children of this MC node to parse them
//         let mc_children = mc.children.take();
//         // if this cat name has not been seen in the global name_id_map, then we add it
//         if !global_mc.contains_key(&name) {
//             global_mc.insert(name.clone(), mc);
//         }
//         // recurse if we have children mc
//         if let Some(children) = mc_children {
//             parse_join_mc(children, &name, global_mc)
//         }
//     }
// }
