use std::str::FromStr;

use enumflags2::{bitflags, BitFlags};
use glam::Vec3;
use itertools::Itertools;
use tracing::info;
use xot::Element;

use crate::io::XotAttributeNameIDs;

use super::RelativePath;
use jokoapi::end_point::mounts::Mount;
use jokoapi::end_point::races::Race;
use smol_str::SmolStr;
/// This is a onetime macro to reduce code duplication
/// It basically takes the CommmonAttributes struct, adds the active_attributes and bool_attributes fields to it.
/// Then, it creates a method call `inherit_if_attr_none`, which will clone fields from other struct, if its own fields are not active (set)
/// Finally, it derives a getter and setter for all of the fields.
///
/// Once we are close to releasing a 1.0 version of this crate, we should just expand all these macros to raw code as its never going to change again.
macro_rules! common_attributes_struct_macro {
    (
      $( #[$attr:meta] )*
      $vis:vis struct $name:ident {
        $( $( #[$field_attr:meta] )* $field_vis:vis $field:ident : $ty:ty ),* $(,)?
      }
    ) => {
        $( #[$attr] )*
        $vis struct $name {
            active_attributes: BitFlags<ActiveAttributes>,
            bool_attributes: BitFlags<BoolAttributes>,
            $( $( #[$field_attr] )* $field : $ty ),*
        }
        impl $name {
            $vis fn inherit_if_attr_none(&mut self, other: &$name) {
                $(if !self.active_attributes.contains(ActiveAttributes::$field)
                    && other.active_attributes.contains(ActiveAttributes::$field) {
                self.active_attributes.insert(ActiveAttributes::$field);
                self.$field = other.$field.clone();
            })+
            }
            $(
                paste::paste!(
                    /// This gets the value IF the attribute is set. Otherwise returns None.
                    #[allow(unused)]
                    $vis fn [<get_  $field>](&self) -> Option<&$ty> {
                        self.active_attributes.contains(ActiveAttributes::$field).then_some(&self.$field)
                    }
                    /// This directly sets the field to value IF the value is Some. Otherwise deactivates the attribute.
                    ///
                    /// Warning: This simply overwrites the value of the existing field.
                    /// So, if you wanted to combine them (an array or bitflags), then do get -> combine it smh -> set.
                    #[allow(unused)]
                    $vis fn [<set_  $field>](&mut self, value: Option<$ty>)  {
                        if let Some(value) = value {
                            self.active_attributes.insert(ActiveAttributes::$field);
                            self.$field = value;
                        } else {
                            self.active_attributes.remove(ActiveAttributes::$field);
                        }
                    }
                );
            )+
        }
    }
}
/// uses the [ToString] impl of attributes to serialize them (only if the relevant active attribute flag is set)
///
/// #### Args:
/// - ca: &[CommonAttributes] (ref to the struct that we are serializing)
/// - ele: &[xot::Element] (xot Element to which we are serializing our fields to)
/// - names: &[XotAttributeNameIDs] (which contains the name ids of our fields)
/// - [f1, f2, f3...]: an array of field identifiers which will be serialized.
/// ```rust
/// set_attribute_to_ele!(ca, ele, names, [field1, field2, field3]);
/// ```
///
/// The expansion for each field is like this
/// ```rust
/// if ca.active_attributes.contains(ActiveAttributes::field1) {
///     ele.set_attribute(names.field1, ca.field1.to_string());
/// }
/// ```
macro_rules! set_attribute_to_ele {
    ($ca: ident, $ele: ident,$names: ident, [$($field: ident),+]) => {
        $(if $ca.active_attributes.contains(ActiveAttributes::$field) {
            $ele.set_attribute($names.$field, $ca.$field.to_string());
        })+
    };
}
/// true -> 1 and 0 -> false. (only if the relevant active attribute flag is set)
///
/// #### Args:
/// - ca: &[CommonAttributes] (ref to the struct that we are serializing)
/// - ele: &[xot::Element] (xot Element to which we are serializing our fields to)
/// - names: &[XotAttributeNameIDs] (which contains the name ids of our fields)
/// - [f1, f2, f3...]: an array of field identifiers which will be serialized.
/// ```rust
/// set_attribute_bool_to_ele!(ca, ele, names, [field1, field2, field3]);
/// ```
///
/// The expansion for each field is like this
/// ```rust
/// if ca.active_attributes.contains(ActiveAttributes::field1) {
///     ele.set_attribute(names.field1,
///         ca
///             .bool_attributes
///             .contains(BoolAttributes::field1)
///             .then_some(1)
///             .unwrap_or(0u8)
///             .to_string()
///     );
/// }
/// ```
macro_rules! set_attribute_bool_to_ele {
    ($ca: ident, $ele: ident,$names: ident, [$($field: ident),+]) => {
        $(if $ca.active_attributes.contains(ActiveAttributes::$field) {
            $ele.set_attribute(
                $names.can_fade,
                $ca.bool_attributes
                    .contains(BoolAttributes::$field)
                    .then_some(1)
                    .unwrap_or(0u8)
                    .to_string(),
            );
        })+
    };
}
/// iterates over a bitflags field and joins the enabled flags (as str) with comma. (only if the relevant active attribute flag is set)
///
/// #### Args:
/// - ca: &[CommonAttributes] (ref to the struct that we are serializing)
/// - ele: &[xot::Element] (xot Element to which we are serializing our fields to)
/// - names: &[XotAttributeNameIDs] (which contains the name ids of our fields)
/// - [f1, f2, f3...]: an array of field identifiers which will be serialized.
/// ```rust
/// set_attribute_bitflags_as_array_to_ele!(ca, ele, names, [field1, field2, field3]);
/// ```
///
/// The expansion for each field is like this
/// ```rust
/// if ca.active_attributes.contains(ActiveAttributes::field1) {
///     ele.set_attribute(
///         names.field1,
///         ca.field1.iter().map(|s| s.as_ref()).join(","),
///     );
/// }
/// ```
macro_rules! set_attribute_bitflags_as_array_to_ele {
    ($ca: ident, $ele: ident,$names: ident, [$($field: ident),+]) => {
        $(if $ca.active_attributes.contains(ActiveAttributes::$field) {
            $ele.set_attribute(
                $names.$field,
                $ca.$field.iter().map(|s| s.to_string()).join(","),
            );
        })+
    };
}
/// uses the [FromStr] impl of attributes to deserialize them (and set the relevant active attribute flag if successful)
///
/// #### Args:
/// - ca: &[CommonAttributes] (ref to the struct that we are serializing)
/// - ele: &[xot::Element] (xot Element to which we are serializing our fields to)
/// - names: &[XotAttributeNameIDs] (which contains the name ids of our fields)
/// - [f1, f2, f3...]: an array of field identifiers which will be serialized.
/// ```rust
/// update_attribute_from_ele!(ca, ele, names, [field1, field2, field3]);
/// ```
///
/// The expansion for each field is like this
/// ```rust
/// if let Some(value) = ele.get_attribute(names.field1) {
///     match value.trim().parse() {
///         Ok(value) => {
///             ca
///                 .active_attributes
///                 .insert(ActiveAttributes::fiel1);
///             ca.field1 = value;
///         }
///         Err(e) => {
///             tracing::info!(?e, value, "failed to parse {}", "field1");
///         }
///     }
/// }
/// ```
macro_rules! update_attribute_from_ele {
    ($ca: ident, $ele: ident,$names: ident, [$($field: ident),+]) => {
        $(if let Some(value) = $ele.get_attribute($names.$field) {
            match value.trim().parse() {
                Ok(value) => {
                    $ca
                        .active_attributes
                        .insert(ActiveAttributes::$field);
                    $ca.$field = value;
                }
                Err(e) => {
                    tracing::info!(?e, value, "failed to parse {}", stringify!($field));
                }
            }
        })+
    };
}

/// deserializes an [i8] and matches that as 1 -> true and 0 -> false.
/// On success, set the relevant active attribute flag.
///
/// #### Args:
/// - ca: &[CommonAttributes] (ref to the struct that we are serializing)
/// - ele: &[xot::Element] (xot Element to which we are serializing our fields to)
/// - names: &[XotAttributeNameIDs] (which contains the name ids of our fields)
/// - [f1, f2, f3...]: an array of field identifiers which will be serialized.
/// ```rust
/// update_attribute_bool_from_ele!(ca, ele, names, [field1, field2, field3]);
/// ```
///
/// The expansion for each field is like this
/// ```rust
/// if let Some(value) = ele.get_attribute(names.field1) {
///     match value.trim().parse::<i8>() {
///         Ok(value) => {
///             match value {
///                 0 | 1 => {
///                     ca
///                     .active_attributes
///                     .insert(ActiveAttributes::field1);
///                     ca.bool_attributes.set(
///                         BoolAttributes::field1,
///                         if value == 0 { false } else { true },
///                     );
///                 }
///                 _ => {
///                     info!(value, "failed to parse {}", "field1");
///                 }
///             }
///         }
///         Err(e) => {
///             tracing::info!(?e, value, "failed to parse {}", "field1");
///         }
///     }
/// }
/// ```
macro_rules! update_attribute_bool_from_ele {
    ($common_attributes: ident, $ele: ident,$names: ident, [$($field: ident),+]) => {
        $(if let Some(value) = $ele.get_attribute($names.$field) {
            match value.trim().parse::<i8>() {
                Ok(value) => {
                    match value {
                        0 | 1 => {
                            $common_attributes
                            .active_attributes
                            .insert(ActiveAttributes::$field);
                            $common_attributes.bool_attributes.set(
                                BoolAttributes::$field,
                                if value == 0 { false } else { true },
                            );
                        }
                        _ => {
                            info!(value, "failed to parse {}", stringify!($field));
                        }
                    }
                }
                Err(e) => {
                    tracing::info!(?e, value, "failed to parse {}", stringify!($field));
                }
            }
        })+
    };
}
/// deserializes an [i8] and matches that as 1 -> true and 0 -> false.
/// On success, set the relevant active attribute flag.
///
/// #### Args:
/// - ca: &[CommonAttributes] (ref to the struct that we are serializing)
/// - ele: &[xot::Element] (xot Element to which we are serializing our fields to)
/// - names: &[XotAttributeNameIDs] (which contains the name ids of our fields)
/// - [f1,t1; f2,t2;...]: an array of field identifiers which will be serialized and their enum type.
/// ```rust
/// update_attribute_bitflags_array_from_ele!(ca, ele, names, [f1, t1; f2, t2]);
/// ```
///
/// The expansion for each field is like this
/// ```rust
/// if let Some(field1_str) = ele.get_attribute(names.field1) {
///     for value in field1_str.split(',') {
///         match value.trim().parse::<t1>() {
///             Ok(flag) => {
///                 ca
///                 .active_attribus
///                 .insert(ActiveAttributes::field1);
///                 ca.field1.set(flag);
///             }
///             Err(e) => {
///                 tracing::info!(value, e);
///             }
///         }
///     }
/// }
/// ```
macro_rules! update_attribute_bitflags_array_from_ele {
    ($ca: ident, $ele: ident,$xot_names: ident, [$($field: ident, $ty: ty);+]) => {
        $(if let Some(value) = $ele.get_attribute($xot_names.$field) {
            for item in value.trim().split(',') {
                match item.trim().parse::<$ty>() {
                    Ok(flag) => {
                        $ca.active_attributes.insert(ActiveAttributes::$field);
                        $ca.$field.insert(flag);
                    }
                    Err(e) => {
                        info!(item, e);
                    }
                }
            }
        })+
    };
}
/// generates getters for bool attributes
/// ```rust
/// getters_for_bool_attributes!([field1, field2, field3]);
/// ```
///
/// This generates a `fn get_field1(&self) -> Option<bool>`
/// if attribute is not active, we return None. Otherwise, the value of the boolean attribute
macro_rules! getters_for_bool_attributes {
    ([$($field: ident),+]) => {
        paste::paste!{
        $(
            /// If the attribute is not set, then we return None.
            /// Otherwise, we return the boolean value of the attribute.
            #[allow(unused)]
            fn [<get_ $field>](&self) -> Option<bool> {
                self.active_attributes.contains(ActiveAttributes::$field).then_some(
                    self.bool_attributes.contains(BoolAttributes::$field)
                )
            }
        )+
        }
    };
}
/// generates setters for bool attributes
/// ```rust
/// setters_for_bool_attributes!([field1, field2, field3]);
/// ```
///
/// This generates a `fn set_field1(&mut self, value: Option<bool>)`
/// if attribute is not active, we return None. Otherwise, the value of the boolean attribute
macro_rules! setters_for_bool_attributes {
    ([$($field: ident),+]) => {
        paste::paste!{
        $(
            /// If the attribute is not set, then we return None.
            /// Otherwise, we return the boolean value of the attribute.
            #[allow(unused)]
            fn [<set_ $field>](&mut self, value: Option<bool>) {
                if let Some(value) = value {
                    self.active_attributes.insert(ActiveAttributes::$field);
                    self.bool_attributes.set(BoolAttributes::$field, value);
                } else {
                    self.active_attributes.remove(ActiveAttributes::$field);
                }
            }
        )+
        }
    };
}
common_attributes_struct_macro!(
    /// the struct we use for inheritance from category/other markers.
    #[derive(Debug, Clone, Default)]
    pub(crate) struct CommonAttributes {
        /// An ID for an achievement from the GW2 API. Markers with the corresponding achievement ID will be hidden if the ID is marked as "done" for the API key that's entered in TacO.
        achievement_id: u32,
        /// This is similar to achievementId, but works for partially completed achievements as well, if the achievement has "bits", they can be individually referenced with this.
        achievement_bit: u32,
        /// How opaque the displayed icon should be. The default is 1.0
        alpha: f32,
        anim_speed: f32,
        /// it describes the way the marker will behave when a player presses 'F' over it.
        behavior: Behavior,
        bounce: SmolStr,
        bounce_delay: f32,
        bounce_duration: f32,
        bounce_height: f32,
        /// hex value. The color tint of the marker. sRGBA8
        color: [u8; 4],
        copy: SmolStr,
        copy_message: SmolStr,
        cull: Cull,
        /// Determines how far the marker will completely disappear. If below 0, the marker won't disappear at any distance. Default is -1. FadeFar needs to be higher than fadeNear for sane results. This value is in game units (inches).
        // #[serde(rename = "fadeFar")]
        fade_far: f32,
        /// Determines how far the marker will start to fade out. If below 0, the marker won't disappear at any distance. Default is -1. This value is in game units (inches).
        // #[serde(rename = "fadeNear")]
        fade_near: f32,
        festival: BitFlags<Festival>,
        /// Specifies how high above the ground the marker is displayed. Default value is 1.5. in meters
        height_offset: f32,
        hide: SmolStr,
        /// The icon to be displayed for the marker. If not given, this defaults to the image shown at the start of this article. This should point to a .png file. The overlay looks for the image files both starting from the root directory and the POIs directory for convenience. Make sure you don't use too high resolution (above 128x128) images because the texture atlas used for these is limited in size and it's a needless waste of resources to fill it quickly.Default value: 20
        icon_file: RelativePath,
        /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
        icon_size: f32,
        /// his can be a multiline string, it will show up on screen as a text when the player is inside of infoRange of the marker
        info: SmolStr,
        /// This determines how far away from the marker the info string will be visible. in meters.
        info_range: f32,
        /// The size of the marker at normal UI scale, at zoom level 1 on the miniMap, in Pixels. For trails this value can be used to tweak the width
        // #[serde(rename = "mapDisplaySize")]
        map_display_size: f32,
        map_fade_out_scale_level: f32,
        map_type: BitFlags<MapType>,
        /// Determines the maximum size of a marker on the screen, in pixels.
        // #[serde(rename = "maxSize")]
        max_size: f32,
        /// Determines the minimum size of a marker on the screen, in pixels.
        // #[serde(rename = "minSize")]
        min_size: f32,
        mount: BitFlags<Mount>,
        profession: BitFlags<Profession>,
        race: BitFlags<Race>,
        /// For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is. in seconds.
        // #[serde(rename = "resetLength")]
        reset_length: f32,
        /// this will supply data for behavior 5. The data will be given in seconds.
        // #[serde(rename = "resetOffset")]
        reset_offset: f32,
        rotate: Vec3,
        rotate_x: f32,
        rotate_y: f32,
        rotate_z: f32,
        show: SmolStr,
        specialization: Vec<Specialization>,
        text: SmolStr,
        texture: RelativePath,
        tip_name: SmolStr,
        tip_description: SmolStr,
        title: SmolStr,
        title_color: [u8; 4],
        /// will toggle the specified category on or off when triggered with the action key. or with auto_trigger/trigger_range
        // #[serde(rename = "toggleCategory")]
        toggle_category: SmolStr,
        trail_data: RelativePath,
        trail_scale: f32,
        /// Determines the range from where the marker is triggered. in meters.
        trigger_range: f32,
    }
);

impl CommonAttributes {
    getters_for_bool_attributes!([
        auto_trigger,
        can_fade,
        has_countdown,
        in_game_visibility,
        invert_behavior,
        is_wall,
        keep_on_map_edge,
        map_visibility,
        mini_map_visibility,
        scale_on_map_with_zoom
    ]);
    setters_for_bool_attributes!([
        auto_trigger,
        can_fade,
        has_countdown,
        in_game_visibility,
        invert_behavior,
        is_wall,
        keep_on_map_edge,
        map_visibility,
        mini_map_visibility,
        scale_on_map_with_zoom
    ]);
    pub(crate) fn update_common_attributes_from_element(
        &mut self,
        ele: &Element,
        names: &XotAttributeNameIDs,
    ) {
        if let Some(input_str) = ele.get_attribute(names.color) {
            use data_encoding::HEXLOWER_PERMISSIVE;
            let mut output = [0u8; 4];
            match HEXLOWER_PERMISSIVE.decode_len(input_str.len()) {
                Ok(len) => {
                    match HEXLOWER_PERMISSIVE.decode_mut(input_str.as_bytes(), &mut output[0..len])
                    {
                        Ok(_) => {
                            self.active_attributes.insert(ActiveAttributes::color);
                            self.color = output;
                        }
                        Err(e) => {
                            info!(?e, input_str, "failed to decode hex bytes of the attribute");
                        }
                    }
                }
                Err(e) => {
                    info!(?e, input_str, "failed to get decode len for hex attribute");
                }
            }
        }
        if let Some(input_str) = ele.get_attribute(names.title_color) {
            use data_encoding::HEXLOWER_PERMISSIVE;
            let mut output = [0u8; 4];
            match HEXLOWER_PERMISSIVE.decode_len(input_str.len()) {
                Ok(len) => {
                    match HEXLOWER_PERMISSIVE.decode_mut(input_str.as_bytes(), &mut output[0..len])
                    {
                        Ok(_) => {
                            self.active_attributes.insert(ActiveAttributes::title_color);
                            self.title_color = output;
                        }
                        Err(e) => {
                            info!(?e, input_str, "failed to decode hex bytes of the attribute");
                        }
                    }
                }
                Err(e) => {
                    info!(?e, input_str, "failed to get decode len for hex attribute");
                }
            }
        }
        if let Some(rotate_str) = ele.get_attribute(names.rotate) {
            let mut array = [0f32; 3];
            for (index, value) in rotate_str.trim().split(',').enumerate() {
                match value.parse::<f32>() {
                    Ok(f) => {
                        if let Some(x) = array.get_mut(index) {
                            *x = f;
                            self.rotate = array.into();
                            self.active_attributes.insert(ActiveAttributes::rotate);
                        }
                    }
                    Err(e) => {
                        info!(?e, rotate_str, value, "failed to parse rotate attribute");
                    }
                }
            }
        }
        if let Some(specs) = ele.get_attribute(names.specialization) {
            for spec in specs.trim().split(',') {
                match spec.parse() {
                    Ok(s) => {
                        self.active_attributes
                            .insert(ActiveAttributes::specialization);
                        self.specialization.push(s);
                    }
                    Err(e) => {
                        info!(specs, spec, e);
                    }
                }
            }
        }
        // bitflags with multiple elements
        update_attribute_bitflags_array_from_ele!(self, ele, names, [
            festival, Festival;
            map_type, MapType;
            mount, Mount;
            profession, Profession;
            race, Race
        ]);

        // bools
        update_attribute_bool_from_ele!(
            self,
            ele,
            names,
            [
                auto_trigger,
                can_fade,
                has_countdown,
                in_game_visibility,
                invert_behavior,
                is_wall,
                keep_on_map_edge,
                map_visibility,
                mini_map_visibility,
                scale_on_map_with_zoom
            ]
        );
        update_attribute_from_ele!(
            self,
            ele,
            names,
            [
                icon_file,
                texture,
                trail_data,
                achievement_id,
                achievement_bit,
                bounce,
                copy,
                hide,
                info,
                copy_message,
                show,
                text,
                tip_name,
                tip_description,
                title,
                toggle_category,
                alpha,
                anim_speed,
                bounce_delay,
                bounce_duration,
                bounce_height,
                fade_near,
                fade_far,
                height_offset,
                icon_size,
                info_range,
                map_display_size,
                map_fade_out_scale_level,
                max_size,
                min_size,
                reset_length,
                reset_offset,
                rotate_x,
                rotate_y,
                rotate_z,
                trail_scale,
                trigger_range,
                cull,
                behavior
            ]
        );
    }

    pub(crate) fn serialize_to_element(&self, ele: &mut Element, names: &XotAttributeNameIDs) {
        // color arrays
        if self.active_attributes.contains(ActiveAttributes::color) {
            ele.set_attribute(names.color, data_encoding::HEXLOWER.encode(&self.color));
        }
        if self
            .active_attributes
            .contains(ActiveAttributes::title_color)
        {
            ele.set_attribute(
                names.title_color,
                data_encoding::HEXLOWER.encode(&self.title_color),
            );
        }
        // rotate array
        if self.active_attributes.contains(ActiveAttributes::rotate) {
            ele.set_attribute(
                names.rotate,
                format!("{},{},{}", self.rotate.x, self.rotate.y, self.rotate.z),
            );
        }
        // spec vector
        if self
            .active_attributes
            .contains(ActiveAttributes::specialization)
        {
            ele.set_attribute(
                names.specialization,
                self.specialization
                    .iter()
                    .copied()
                    .map(|s| s as u8)
                    .join(","),
            );
        }
        // bitflags arrays
        set_attribute_bitflags_as_array_to_ele!(
            self,
            ele,
            names,
            [festival, map_type, mount, profession, race]
        );
        // bools
        set_attribute_bool_to_ele!(
            self,
            ele,
            names,
            [
                auto_trigger,
                can_fade,
                has_countdown,
                in_game_visibility,
                invert_behavior,
                is_wall,
                keep_on_map_edge,
                map_visibility,
                mini_map_visibility,
                scale_on_map_with_zoom
            ]
        );
        // tostrings
        set_attribute_to_ele!(
            self,
            ele,
            names,
            [
                icon_file,
                texture,
                trail_data,
                achievement_id,
                achievement_bit,
                bounce,
                copy,
                hide,
                info,
                copy_message,
                show,
                text,
                tip_name,
                tip_description,
                title,
                toggle_category,
                alpha,
                anim_speed,
                bounce_delay,
                bounce_duration,
                bounce_height,
                fade_near,
                fade_far,
                height_offset,
                icon_size,
                info_range,
                map_display_size,
                map_fade_out_scale_level,
                max_size,
                min_size,
                reset_length,
                reset_offset,
                rotate_x,
                rotate_y,
                rotate_z,
                trail_scale,
                trigger_range
            ]
        );
    }
    /*

    TF32 height = 1.5f;
    TF32 triggerRange = 2.0f;
    TF32 animSpeed = 1;
    TS32 miniMapSize = 20;
    TF32 miniMapFadeOutLevel = 100.0f;
    TF32 infoRange = 2.0f;
    CColor color = CColor( 0xffffffff );

    TS16 resetLength = 0;
    TS16 minSize = 5;
    TS16 maxSize = 2048;

    */
}

#[allow(non_camel_case_types)]
#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum BoolAttributes {
    /// should the trigger activate when within trigger range
    auto_trigger = 1,
    can_fade = 1 << 1,
    /// should we show the countdown timers for markers that are sleeping
    has_countdown = 1 << 2,
    /// whether the marker is drawn ingame
    in_game_visibility = 1 << 3,
    invert_behavior = 1 << 4,
    is_wall = 1 << 5,
    keep_on_map_edge = 1 << 6,
    /// whether draw on map
    map_visibility = 1 << 7,
    /// draw on minimap
    mini_map_visibility = 1 << 8,
    /// scaling of marker on 2d map (or minimap)
    scale_on_map_with_zoom = 1 << 9,
}
#[allow(non_camel_case_types)]
#[bitflags]
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum ActiveAttributes {
    achievement_id = 1,
    achievement_bit = 1 << 1,
    alpha = 1 << 2,
    anim_speed = 1 << 3,
    auto_trigger = 1 << 4,
    behavior = 1 << 5,
    bounce = 1 << 6,
    bounce_delay = 1 << 7,
    bounce_duration = 1 << 8,
    bounce_height = 1 << 9,
    can_fade = 1 << 10,
    color = 1 << 11,
    copy = 1 << 12,
    copy_message = 1 << 13,
    cull = 1 << 14,
    fade_far = 1 << 15,
    fade_near = 1 << 16,
    festival = 1 << 17,
    has_countdown = 1 << 18,
    height_offset = 1 << 19,
    hide = 1 << 20,
    icon_file = 1 << 21,
    icon_size = 1 << 22,
    in_game_visibility = 1 << 23,
    info = 1 << 24,
    info_range = 1 << 25,
    invert_behavior = 1 << 26,
    is_wall = 1 << 27,
    keep_on_map_edge = 1 << 28,
    map_display_size = 1 << 29,
    map_fade_out_scale_level = 1 << 30,
    map_type = 1 << 31,
    map_visibility = 1 << 32,
    max_size = 1 << 33,
    min_size = 1 << 34,
    mini_map_visibility = 1 << 35,
    mount = 1 << 36,
    profession = 1 << 37,
    race = 1 << 38,
    reset_length = 1 << 39,
    reset_offset = 1 << 40,
    rotate = 1 << 41,
    rotate_x = 1 << 42,
    rotate_y = 1 << 43,
    rotate_z = 1 << 44,
    scale_on_map_with_zoom = 1 << 45,
    show = 1 << 46,
    specialization = 1 << 47,
    text = 1 << 48,
    texture = 1 << 49,
    tip_name = 1 << 50,
    tip_description = 1 << 51,
    title = 1 << 52,
    title_color = 1 << 53,
    toggle_category = 1 << 54,
    trail_data = 1 << 55,
    trail_scale = 1 << 56,
    trigger_range = 1 << 57,
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Behavior {
    #[default]
    AlwaysVisible,
    /// live. marker_id
    ReappearOnMapChange,
    /// store. marker_id + next reset timestamp
    ReappearOnDailyReset,
    /// store. marker_id
    OnlyVisibleBeforeActivation,
    /// store. marker_id + timestamp of when to wakeup
    ReappearAfterTimer,
    /// store. marker_id + timestamp of next reset of map
    ReappearOnMapReset,
    /// live. marker_id + instance ip / shard id
    OncePerInstance,
    /// store. marker_id + next reset. character data
    DailyPerChar,
    /// live. marker_id + instance_id + character_name
    OncePerInstancePerChar,
    /// I have no idea.
    WvWObjective,
    WeeklyReset = 101,
}
impl FromStr for Behavior {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "0" => Self::AlwaysVisible,
            "1" => Self::ReappearOnMapChange,
            "2" => Self::ReappearOnDailyReset,
            "3" => Self::OnlyVisibleBeforeActivation,
            "4" => Self::ReappearAfterTimer,
            "5" => Self::ReappearOnMapReset,
            "6" => Self::OncePerInstance,
            "7" => Self::DailyPerChar,
            "8" => Self::OncePerInstancePerChar,
            "9" => Self::WvWObjective,
            "101" => Self::WeeklyReset,
            _ => return Err("invalid behavior value"),
        })
    }
}
/// Filter which professions the marker should be active for. if its null, its available for all professions
#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Profession {
    Elementalist = 1 << 0,
    Engineer = 1 << 1,
    Guardian = 1 << 2,
    Mesmer = 1 << 3,
    Necromancer = 1 << 4,
    Ranger = 1 << 5,
    Revenant = 1 << 6,
    Thief = 1 << 7,
    Warrior = 1 << 8,
}
impl FromStr for Profession {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "guardian" => Profession::Guardian,
            "warrior" => Profession::Warrior,
            "engineer" => Profession::Engineer,
            "ranger" => Profession::Ranger,
            "thief" => Profession::Thief,
            "elementalist" => Profession::Elementalist,
            "mesmer" => Profession::Mesmer,
            "necromancer" => Profession::Necromancer,
            "revenant" => Profession::Revenant,
            _ => return Err("invalid profession"),
        })
    }
}
impl AsRef<str> for Profession {
    fn as_ref(&self) -> &str {
        match self {
            Profession::Guardian => "guardian",
            Profession::Warrior => "warrior",
            Profession::Engineer => "engineer",
            Profession::Ranger => "ranger",
            Profession::Thief => "thief",
            Profession::Elementalist => "elementalist",
            Profession::Mesmer => "mesmer",
            Profession::Necromancer => "necromancer",
            Profession::Revenant => "revenant",
        }
    }
}
impl ToString for Profession {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub enum Cull {
    #[default]
    None,
    ClockWise,
    CounterClockWise,
}
impl FromStr for Cull {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "None" => Cull::None,
            "Clockwise" => Cull::ClockWise,
            "CounterClockwise" => Cull::CounterClockWise,
            _ => {
                return Err("invalid value for cull attribute");
            }
        })
    }
}
impl AsRef<str> for Cull {
    fn as_ref(&self) -> &'static str {
        match self {
            Cull::None => "None",
            Cull::ClockWise => "Clockwise",
            Cull::CounterClockWise => "CounterClockwise",
        }
    }
}
impl ToString for Cull {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
/// Filter for which festivals will the marker be active for
#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Festival {
    DragonBash = 1 << 0,
    #[allow(clippy::enum_variant_names)]
    FestivalOfTheFourWinds = 1 << 1,
    Halloween = 1 << 2,
    LunarNewYear = 1 << 3,
    SuperAdventureBox = 1 << 4,
    Wintersday = 1 << 5,
}
impl FromStr for Festival {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "halloween" => Festival::Halloween,
            "wintersday" => Festival::Wintersday,
            "superadventurefestival" => Festival::SuperAdventureBox,
            "lunarnewyear" => Festival::LunarNewYear,
            "festivalofthefourwinds" => Festival::FestivalOfTheFourWinds,
            "dragonbash" => Festival::DragonBash,
            _ => return Err("unrecognized festival"),
        })
    }
}
impl AsRef<str> for Festival {
    fn as_ref(&self) -> &'static str {
        match self {
            Festival::Halloween => "halloween",
            Festival::Wintersday => "wintersday",
            Festival::SuperAdventureBox => "superadventurefestival",
            Festival::LunarNewYear => "lunarnewyear",
            Festival::FestivalOfTheFourWinds => "festivalofthefourwinds",
            Festival::DragonBash => "dragonbash",
        }
    }
}
impl ToString for Festival {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
/// Filter for which specializations (the third traitline) will the marker be active for
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Specialization {
    Dueling = 0,
    DeathMagic = 1,
    Invocation = 2,
    Strength = 3,
    Druid = 4,
    Explosives = 5,
    Daredevil = 6,
    Marksmanship = 7,
    Retribution = 8,
    Domination = 9,
    Tactics = 10,
    Salvation = 11,
    Valor = 12,
    Corruption = 13,
    Devastation = 14,
    Radiance = 15,
    Water = 16,
    Berserker = 17,
    BloodMagic = 18,
    ShadowArts = 19,
    Tools = 20,
    Defense = 21,
    Inspiration = 22,
    Illusions = 23,
    NatureMagic = 24,
    Earth = 25,
    Dragonhunter = 26,
    DeadlyArts = 27,
    Alchemy = 28,
    Skirmishing = 29,
    Fire = 30,
    BeastMastery = 31,
    WildernessSurvival = 32,
    Reaper = 33,
    CriticalStrikes = 34,
    Arms = 35,
    Arcane = 36,
    Firearms = 37,
    Curses = 38,
    Chronomancer = 39,
    Air = 40,
    Zeal = 41,
    Scrapper = 42,
    Trickery = 43,
    Chaos = 44,
    Virtues = 45,
    Inventions = 46,
    Tempest = 47,
    Honor = 48,
    SoulReaping = 49,
    Discipline = 50,
    Herald = 51,
    Spite = 52,
    Acrobatics = 53,
    Soulbeast = 54,
    Weaver = 55,
    Holosmith = 56,
    Deadeye = 57,
    Mirage = 58,
    Scourge = 59,
    Spellbreaker = 60,
    Firebrand = 61,
    Renegade = 62,
    Harbinger = 63,
    Willbender = 64,
    Virtuoso = 65,
    Catalyst = 66,
    Bladesworn = 67,
    Vindicator = 68,
    Mechanist = 69,
    Specter = 70,
    Untamed = 71,
}

impl FromStr for Specialization {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dueling" => Self::Dueling,
            "deathmagic" => Self::DeathMagic,
            "invocation" => Self::Invocation,
            "strength" => Self::Strength,
            "druid" => Self::Druid,
            "explosives" => Self::Explosives,
            "daredevil" => Self::Daredevil,
            "marksmanship" => Self::Marksmanship,
            "retribution" => Self::Retribution,
            "domination" => Self::Domination,
            "tactics" => Self::Tactics,
            "salvation" => Self::Salvation,
            "valor" => Self::Valor,
            "corruption" => Self::Corruption,
            "devastation" => Self::Devastation,
            "radiance" => Self::Radiance,
            "water" => Self::Water,
            "berserker" => Self::Berserker,
            "bloodmagic" => Self::BloodMagic,
            "shadowarts" => Self::ShadowArts,
            "tools" => Self::Tools,
            "defense" => Self::Defense,
            "inspiration" => Self::Inspiration,
            "illusions" => Self::Illusions,
            "naturemagic" => Self::NatureMagic,
            "earth" => Self::Earth,
            "dragonhunter" => Self::Dragonhunter,
            "deadlyarts" => Self::DeadlyArts,
            "alchemy" => Self::Alchemy,
            "skirmishing" => Self::Skirmishing,
            "fire" => Self::Fire,
            "beastmastery" => Self::BeastMastery,
            "wildernesssurvival" => Self::WildernessSurvival,
            "reaper" => Self::Reaper,
            "criticalstrikes" => Self::CriticalStrikes,
            "arms" => Self::Arms,
            "arcane" => Self::Arcane,
            "firearms" => Self::Firearms,
            "curses" => Self::Curses,
            "chronomancer" => Self::Chronomancer,
            "air" => Self::Air,
            "zeal" => Self::Zeal,
            "scrapper" => Self::Scrapper,
            "trickery" => Self::Trickery,
            "chaos" => Self::Chaos,
            "virtues" => Self::Virtues,
            "inventions" => Self::Inventions,
            "tempest" => Self::Tempest,
            "honor" => Self::Honor,
            "soulreaping" => Self::SoulReaping,
            "discipline" => Self::Discipline,
            "herald" => Self::Herald,
            "spite" => Self::Spite,
            "acrobatics" => Self::Acrobatics,
            "soulbeast" => Self::Soulbeast,
            "weaver" => Self::Weaver,
            "holosmith" => Self::Holosmith,
            "deadeye" => Self::Deadeye,
            "mirage" => Self::Mirage,
            "scourge" => Self::Scourge,
            "spellbreaker" => Self::Spellbreaker,
            "firebrand" => Self::Firebrand,
            "renegade" => Self::Renegade,
            "harbinger" => Self::Harbinger,
            "willbender" => Self::Willbender,
            "virtuoso" => Self::Virtuoso,
            "catalyst" => Self::Catalyst,
            "bladesworn" => Self::Bladesworn,
            "vindicator" => Self::Vindicator,
            "mechanist" => Self::Mechanist,
            "specter" => Self::Specter,
            "untamed" => Self::Untamed,
            _ => return Err("invalid specialization"),
        })
    }
}
impl AsRef<str> for Specialization {
    fn as_ref(&self) -> &str {
        match self {
            Self::Dueling => "dueling",
            Self::DeathMagic => "deathmagic",
            Self::Invocation => "invocation",
            Self::Strength => "strength",
            Self::Druid => "druid",
            Self::Explosives => "explosives",
            Self::Daredevil => "daredevil",
            Self::Marksmanship => "marksmanship",
            Self::Retribution => "retribution",
            Self::Domination => "domination",
            Self::Tactics => "tactics",
            Self::Salvation => "salvation",
            Self::Valor => "valor",
            Self::Corruption => "corruption",
            Self::Devastation => "devastation",
            Self::Radiance => "radiance",
            Self::Water => "water",
            Self::Berserker => "berserker",
            Self::BloodMagic => "bloodmagic",
            Self::ShadowArts => "shadowarts",
            Self::Tools => "tools",
            Self::Defense => "defense",
            Self::Inspiration => "inspiration",
            Self::Illusions => "illusions",
            Self::NatureMagic => "naturemagic",
            Self::Earth => "earth",
            Self::Dragonhunter => "dragonhunter",
            Self::DeadlyArts => "deadlyarts",
            Self::Alchemy => "alchemy",
            Self::Skirmishing => "skirmishing",
            Self::Fire => "fire",
            Self::BeastMastery => "beastmastery",
            Self::WildernessSurvival => "wildernesssurvival",
            Self::Reaper => "reaper",
            Self::CriticalStrikes => "criticalstrikes",
            Self::Arms => "arms",
            Self::Arcane => "arcane",
            Self::Firearms => "firearms",
            Self::Curses => "curses",
            Self::Chronomancer => "chronomancer",
            Self::Air => "air",
            Self::Zeal => "zeal",
            Self::Scrapper => "scrapper",
            Self::Trickery => "trickery",
            Self::Chaos => "chaos",
            Self::Virtues => "virtues",
            Self::Inventions => "inventions",
            Self::Tempest => "tempest",
            Self::Honor => "honor",
            Self::SoulReaping => "soulreaping",
            Self::Discipline => "discipline",
            Self::Herald => "herald",
            Self::Spite => "spite",
            Self::Acrobatics => "acrobatics",
            Self::Soulbeast => "soulbeast",
            Self::Weaver => "weaver",
            Self::Holosmith => "holosmith",
            Self::Deadeye => "deadeye",
            Self::Mirage => "mirage",
            Self::Scourge => "scourge",
            Self::Spellbreaker => "spellbreaker",
            Self::Firebrand => "firebrand",
            Self::Renegade => "renegade",
            Self::Harbinger => "harbinger",
            Self::Willbender => "willbender",
            Self::Virtuoso => "virtuoso",
            Self::Catalyst => "catalyst",
            Self::Bladesworn => "bladesworn",
            Self::Vindicator => "vindicator",
            Self::Mechanist => "mechanist",
            Self::Specter => "specter",
            Self::Untamed => "untamed",
        }
    }
}

impl ToString for Specialization {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
/// Most of this data is stolen from BlishHUD.
#[bitflags]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MapType {
    Unknown = 1 << 0,
    /// Redirect map type, e.g. when logging in while in a PvP match.
    Redirect = 1 << 1,
    /// Character create map type.
    CharacterCreate = 1 << 2,
    /// PvP map type.    
    PvP = 1 << 3,
    /// GvG map type. Unused.
    /// Quote from lye: "lol unused ;_;".
    GvG = 1 << 4,
    /// Instance map type, e.g. dungeons and story content.
    Instance = 1 << 5,
    /// Public map type, e.g. open world.
    Public = 1 << 6,
    /// Tournament map type. Probably unused.
    Tournament = 1 << 7,
    /// Tutorial map type.    
    Tutorial = 1 << 8,
    /// User tournament map type. Probably unused.   
    UserTournament = 1 << 9,
    /// Eternal Battlegrounds (WvW) map type.    
    EternalBattlegrounds = 1 << 10,
    /// Blue Borderlands (WvW) map type.
    BlueBorderlands = 1 << 11,
    /// Green Borderlands (WvW) map type.
    GreenBorderlands = 1 << 12,
    /// Red Borderlands (WvW) map type.    
    RedBorderlands = 1 << 13,
    /// Fortune's Vale. Unused.    
    FortunesVale = 1 << 14,
    /// Obsidian Sanctum (WvW) map type.   
    ObsidianSanctum = 1 << 15,
    /// Edge of the Mists (WvW) map type.    
    EdgeOfTheMists = 1 << 16,
    /// Mini public map type, e.g. Dry Top, the Silverwastes and Mistlock Sanctuary.
    PublicMini = 1 << 17,
    /// WvW lounge map type, e.g. Armistice Bastion.    
    WvwLounge = 1 << 18,
}
impl FromStr for MapType {
    type Err = &'static str;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        unimplemented!("needs research to verify the map type values")
    }
}
impl AsRef<str> for MapType {
    fn as_ref(&self) -> &str {
        unimplemented!("needs research to verify the maptype values")
    }
}
impl ToString for MapType {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
/// made it using multi cursor (ctrl + shift + L) by copy-pasting json from api
#[allow(unused)]
pub static MAP_ID_TO_NAME: phf::OrderedMap<u16, &'static str> = phf::phf_ordered_map! {
    15u16 => "Queensdale",
    17u16 => "Harathi Hinterlands",
    18u16 => "Divinity's Reach",
    19u16 => "Plains of Ashford",
    20u16 => "Blazeridge Steppes",
    21u16 => "Fields of Ruin",
    22u16 => "Fireheart Rise",
    23u16 => "Kessex Hills",
    24u16 => "Gendarran Fields",
    25u16 => "Iron Marches",
    26u16 => "Dredgehaunt Cliffs",
    27u16 => "Lornar's Pass",
    28u16 => "Wayfarer Foothills",
    29u16 => "Timberline Falls",
    30u16 => "Frostgorge Sound",
    31u16 => "Snowden Drifts",
    32u16 => "Diessa Plateau",
    33u16 => "Ascalonian Catacombs",
    34u16 => "Caledon Forest",
    35u16 => "Metrica Province",
    36u16 => "Ascalonian Catacombs",
    37u16 => "Arson at the Orphanage",
    38u16 => "Eternal Battlegrounds",
    39u16 => "Mount Maelstrom",
    50u16 => "Lion's Arch",
    51u16 => "Straits of Devastation",
    53u16 => "Sparkfly Fen",
    54u16 => "Brisban Wildlands",
    55u16 => "The Hospital in Jeopardy",
    61u16 => "Infiltration",
    62u16 => "Cursed Shore",
    63u16 => "Sorrow's Embrace",
    64u16 => "Sorrow's Embrace",
    65u16 => "Malchor's Leap",
    66u16 => "Citadel of Flame",
    67u16 => "Twilight Arbor",
    68u16 => "Twilight Arbor",
    69u16 => "Citadel of Flame",
    70u16 => "Honor of the Waves",
    71u16 => "Honor of the Waves",
    73u16 => "Bloodtide Coast",
    75u16 => "Caudecus's Manor",
    76u16 => "Caudecus's Manor",
    77u16 => "Search the Premises",
    79u16 => "The Informant",
    80u16 => "A Society Function",
    81u16 => "Crucible of Eternity",
    82u16 => "Crucible of Eternity",
    89u16 => "Chasing the Culprits",
    91u16 => "The Grove",
    92u16 => "The Trial of Julius Zamon",
    95u16 => " Alpine Borderlands",
    96u16 => " Alpine Borderlands",
    97u16 => "Infiltration",
    110u16 => "The Perils of Friendship",
    111u16 => "Victory or Death",
    112u16 => "The Ruined City of Arah",
    113u16 => "Desperate Medicine",
    120u16 => "The Commander",
    138u16 => "Defense of Shaemoor",
    139u16 => "Rata Sum",
    140u16 => "The Apothecary",
    142u16 => "Going Undercover",
    143u16 => "Going Undercover",
    144u16 => "The Greater Good",
    145u16 => "The Rescue",
    147u16 => "Breaking the Blade",
    148u16 => "The Fall of Falcon Company",
    149u16 => "The Fall of Falcon Company",
    152u16 => "Confronting Captain Tervelan",
    153u16 => "Seek Logan's Aid",
    154u16 => "Seek Logan's Aid",
    157u16 => "Accusation",
    159u16 => "Accusation",
    161u16 => "Liberation",
    162u16 => "Voices From the Past",
    163u16 => "Voices From the Past",
    171u16 => "Rending the Mantle",
    172u16 => "Rending the Mantle",
    178u16 => "The Floating Grizwhirl",
    179u16 => "The Floating Grizwhirl",
    180u16 => "The Floating Grizwhirl",
    182u16 => "Clown College",
    184u16 => "The Artist's Workshop",
    185u16 => "Into the Woods",
    186u16 => "The Ringmaster",
    190u16 => "The Orders of Tyria",
    191u16 => "The Orders of Tyria",
    192u16 => "Brute Force",
    193u16 => "Mortus Virge",
    195u16 => "Triskell Quay",
    196u16 => "Track the Seraph",
    198u16 => "Speaker of the Dead",
    199u16 => "The Sad Tale of the \"Ravenous\"",
    201u16 => "Kellach's Attack",
    202u16 => "The Queen's Justice",
    203u16 => "The Trap",
    211u16 => "Best Laid Plans",
    212u16 => "Welcome Home",
    215u16 => "The Tribune's Call",
    216u16 => "The Tribune's Call",
    217u16 => "The Tribune's Call",
    218u16 => "Black Citadel",
    222u16 => "A Spy for a Spy",
    224u16 => "Scrapyard Dogs",
    225u16 => "A Spy for a Spy",
    226u16 => "On the Mend",
    232u16 => "Spilled Blood",
    234u16 => "Ghostbore Musket",
    237u16 => "Iron Grip of the Legion",
    238u16 => "The Flame Advances",
    239u16 => "The Flame Advances",
    242u16 => "Test Your Metal",
    244u16 => "Quick and Quiet",
    248u16 => "Salma District (Home)",
    249u16 => "An Unusual Inheritance",
    250u16 => "Windrock Maze",
    251u16 => "Mired Deep",
    252u16 => "Mired Deep",
    254u16 => "Deadly Force",
    255u16 => "Ghostbore Artillery",
    256u16 => "No Negotiations",
    257u16 => "Salvaging Scrap",
    258u16 => "Salvaging Scrap",
    259u16 => "In the Ruins",
    260u16 => "In the Ruins",
    262u16 => "Chain of Command",
    263u16 => "Chain of Command",
    264u16 => "Time for a Promotion",
    267u16 => "The End of the Line",
    269u16 => "Magic Users",
    271u16 => "Rage Suppression",
    272u16 => "Rage Suppression",
    274u16 => "Operation: Bulwark",
    275u16 => "AWOL",
    276u16 => "Human's Lament",
    282u16 => "Misplaced Faith",
    283u16 => "Thicker Than Water",
    284u16 => "Dishonorable Discharge",
    287u16 => "Searching for the Truth",
    288u16 => "Lighting the Beacons",
    290u16 => "Stoking the Flame",
    294u16 => "A Fork in the Road",
    295u16 => "Sins of the Father",
    297u16 => "Graveyard Ornaments",
    326u16 => "Hoelbrak",
    327u16 => "Desperate Medicine",
    330u16 => "Seraph Headquarters",
    334u16 => "Keg Brawl",
    335u16 => "Claw Island",
    336u16 => "Chantry of Secrets",
    350u16 => "Heart of the Mists",
    363u16 => "The Sting",
    364u16 => "Drawing Out the Cult",
    365u16 => "Ashes of the Past",
    371u16 => "Hero's Canton (Home)",
    372u16 => "Blood Tribune Quarters",
    373u16 => "The Command Core",
    374u16 => "Knut Whitebear's Loft",
    375u16 => "Hunter's Hearth (Home)",
    376u16 => "Stonewright's Steading",
    378u16 => "Queen's Throne Room",
    379u16 => "The Great Hunt",
    380u16 => "A Weapon of Legend",
    381u16 => "The Last of the Giant-Kings",
    382u16 => "Disciples of the Dragon",
    385u16 => "A Weapon of Legend",
    386u16 => "Echoes of Ages Past",
    387u16 => "Wild Spirits",
    388u16 => "Out of the Skies",
    389u16 => "Echoes of Ages Past",
    390u16 => "Twilight of the Wolf",
    391u16 => "Rage of the Minotaurs",
    392u16 => "A Pup's Illness",
    393u16 => "Through the Veil",
    394u16 => "A Trap Foiled",
    396u16 => "Raven's Revered",
    397u16 => "One Good Drink Deserves Another",
    399u16 => "Shape of the Spirit",
    400u16 => "Into the Mists",
    401u16 => "Through the Veil",
    405u16 => "Blessed of Bear",
    407u16 => "The Wolf Havroun",
    410u16 => "Minotaur Rampant",
    411u16 => "Minotaur Rampant",
    412u16 => "Unexpected Visitors",
    413u16 => "Rumors of Trouble",
    414u16 => "A New Challenger",
    415u16 => "Unexpected Visitors",
    416u16 => "Roadblock",
    417u16 => "Assault on Moledavia",
    418u16 => "Don't Leave Your Toys Out",
    419u16 => "A New Challenger",
    420u16 => "First Attack",
    421u16 => "The Finishing Blow",
    422u16 => "The Semifinals",
    423u16 => "The Championship Fight",
    424u16 => "The Championship Fight",
    425u16 => "The Machine in Action",
    427u16 => "Among the Kodan",
    428u16 => "Rumors of Trouble",
    429u16 => "Rage of the Minotaurs",
    430u16 => "Darkness at Drakentelt",
    432u16 => "Fighting the Nightmare",
    434u16 => "Preserving the Balance",
    435u16 => "Means to an End",
    436u16 => "Dredge Technology",
    439u16 => "Underground Scholar",
    440u16 => "Dredge Assault",
    441u16 => "The Dredge Hideout",
    444u16 => "Sabotage",
    447u16 => "Codebreaker",
    449u16 => "Armaments",
    453u16 => "Assault the Hill",
    454u16 => "Silent Warfare",
    455u16 => "Sever the Head",
    458u16 => "Fury of the Dead",
    459u16 => "A Fork in the Road",
    460u16 => "Citadel Stockade",
    464u16 => "Tribunes in Effigy",
    465u16 => "Sins of the Father",
    466u16 => "Misplaced Faith",
    470u16 => "Graveyard Ornaments",
    471u16 => "Undead Infestation",
    474u16 => "Whispers in the Dark",
    476u16 => "Dangerous Research",
    477u16 => "Digging Up Answers",
    480u16 => "Defending the Keep",
    481u16 => "Undead Detection",
    483u16 => "Ever Vigilant",
    485u16 => "Research and Destroy",
    487u16 => "Whispers of Vengeance",
    488u16 => "Killer Instinct",
    489u16 => "Meeting my Mentor",
    490u16 => "A Fragile Peace",
    492u16 => "Don't Shoot the Messenger",
    496u16 => "Meeting my Mentor",
    497u16 => "Dredging Up the Past",
    498u16 => "Dredging Up the Past",
    499u16 => "Scrapyard Dogs",
    502u16 => "Quaestor's Siege",
    503u16 => "Minister's Defense",
    504u16 => "Called to Service",
    505u16 => "Called to Service",
    507u16 => "Mockery of Death",
    509u16 => "Discovering Darkness",
    511u16 => "Hounds and the Hunted",
    512u16 => "Hounds and the Hunted",
    513u16 => "Loved and Lost",
    514u16 => "Saving the Stag",
    515u16 => "Hidden in Darkness",
    516u16 => "Good Work Spoiled",
    517u16 => "Black Night, White Stag",
    518u16 => "The Omphalos Chamber",
    519u16 => "Weakness of the Heart",
    520u16 => "Awakening",
    521u16 => "Holding Back the Darkness",
    522u16 => "A Sly Trick",
    523u16 => "Deep Tangled Roots",
    524u16 => "The Heart of Nightmare",
    525u16 => "Beneath a Cold Moon",
    527u16 => "The Knight's Duel",
    528u16 => "Hammer and Steel",
    529u16 => "Where Life Goes",
    532u16 => "After the Storm",
    533u16 => "After the Storm",
    534u16 => "Beneath the Waves",
    535u16 => "Mirror, Mirror",
    536u16 => "A Vision of Darkness",
    537u16 => "Shattered Light",
    538u16 => "An Unknown Soul",
    539u16 => "An Unknown Soul",
    540u16 => "Where Life Goes",
    542u16 => "Source of the Issue",
    543u16 => "Wild Growth",
    544u16 => "Wild Growth",
    545u16 => "Seeking the Zalisco",
    546u16 => "The Direct Approach",
    547u16 => "Trading Trickery",
    548u16 => "Eye of the Sun",
    549u16 => "Battle of Kyhlo",
    552u16 => "Seeking the Zalisco",
    554u16 => "Forest of Niflhel",
    556u16 => "A Different Dream",
    557u16 => "A Splinter in the Flesh",
    558u16 => "Shadow of the Tree",
    559u16 => "Eye of the Sun",
    560u16 => "Sharpened Thorns",
    561u16 => "Bramble Walls",
    563u16 => "Secrets in the Earth",
    564u16 => "The Blossom of Youth",
    566u16 => "The Bad Apple",
    567u16 => "Trouble at the Roots",
    569u16 => "Flower of Death",
    570u16 => "Dead of Winter",
    571u16 => "A Tangle of Weeds",
    573u16 => "Explosive Intellect",
    574u16 => "In Snaff's Footsteps",
    575u16 => "Golem Positioning System",
    576u16 => "Monkey Wrench",
    577u16 => "Defusing the Problem",
    578u16 => "The Things We Do For Love",
    579u16 => "The Snaff Prize",
    581u16 => "A Sparkling Rescue",
    582u16 => "High Maintenance",
    583u16 => "Snaff Would Be Proud",
    584u16 => "Taking Credit Back",
    586u16 => "Political Homicide",
    587u16 => "Here, There, Everywhere",
    588u16 => "Piece Negotiations",
    589u16 => "Readings On the Rise",
    590u16 => "Snaff Would Be Proud",
    591u16 => "Readings On the Rise",
    592u16 => "Unscheduled Delay",
    594u16 => "Stand By Your Krewe",
    595u16 => "Unwelcome Visitors",
    596u16 => "Where Credit Is Due",
    597u16 => "Where Credit Is Due",
    598u16 => "Short Fuse",
    599u16 => "Short Fuse",
    606u16 => "Salt in the Wound",
    607u16 => "Free Rein",
    608u16 => "Serving Up Trouble",
    609u16 => "Serving Up Trouble",
    610u16 => "Flash Flood",
    611u16 => "I Smell a Rat",
    613u16 => "Magnum Opus",
    614u16 => "Magnum Opus",
    617u16 => "Bad Business",
    618u16 => "Beta Test",
    619u16 => "Beta Test",
    620u16 => "Any Sufficiently Advanced Science",
    621u16 => "Any Sufficiently Advanced Science",
    622u16 => "Bad Forecast",
    623u16 => "Industrial Espionage",
    624u16 => "Split Second",
    625u16 => "Carry a Big Stick",
    627u16 => "Meeting my Mentor",
    628u16 => "Stealing Secrets",
    629u16 => "A Bold New Theory",
    630u16 => "Forging Permission",
    631u16 => "Forging Permission",
    633u16 => "Setting the Stage",
    634u16 => "Containment",
    635u16 => "Containment",
    636u16 => "Hazardous Environment",
    638u16 => "Down the Hatch",
    639u16 => "Down the Hatch",
    642u16 => "The Stone Sheath",
    643u16 => "Bad Blood",
    644u16 => "Test Subject",
    645u16 => "Field Test",
    646u16 => "The House of Caithe",
    647u16 => "Dreamer's Terrace (Home)",
    648u16 => "The Omphalos Chamber",
    649u16 => "Snaff Memorial Lab",
    650u16 => "Applied Development Lab (Home)",
    651u16 => "Council Level",
    652u16 => "A Meeting of the Minds",
    653u16 => "Mightier than the Sword",
    654u16 => "They Went Thataway",
    655u16 => "Lines of Communication",
    656u16 => "Untamed Wilds",
    657u16 => "An Apple a Day",
    658u16 => "Base of Operations",
    659u16 => "The Lost Chieftain's Return",
    660u16 => "Thrown Off Guard",
    662u16 => "Pets and Walls Make Stronger Kraals",
    663u16 => "Doubt",
    664u16 => "The False God's Lair",
    666u16 => "Bad Ice",
    667u16 => "Bad Ice",
    668u16 => "Pets and Walls Make Stronger Kraals",
    669u16 => "Attempted Deicide",
    670u16 => "Doubt",
    672u16 => "Rat-Tastrophe",
    673u16 => "Salvation Through Heresy",
    674u16 => "Enraged and Unashamed",
    675u16 => "Pastkeeper",
    676u16 => "Protest Too Much",
    677u16 => "Prying the Eye Open",
    678u16 => "The Hatchery",
    680u16 => "Convincing the Faithful",
    681u16 => "Evacuation",
    682u16 => "Untamed Wilds",
    683u16 => "Champion's Sacrifice",
    684u16 => "Thieving from Thieves",
    685u16 => "Crusader's Return",
    686u16 => "Unholy Grounds",
    687u16 => "Chosen of the Sun",
    691u16 => "Set to Blow",
    692u16 => "Gadd's Last Gizmo",
    693u16 => "Library Science",
    694u16 => "Rakt and Ruin",
    695u16 => "Suspicious Activity",
    696u16 => "Reconnaissance",
    697u16 => "Critical Blowback",
    698u16 => "The Battle of Claw Island",
    699u16 => "Suspicious Activity",
    700u16 => "Priory Library",
    701u16 => "On Red Alert",
    702u16 => "Forearmed Is Forewarned",
    703u16 => "The Oratory",
    704u16 => "Killing Fields",
    705u16 => "The Ghost Rite",
    706u16 => "The Good Fight",
    707u16 => "Defense Contract",
    708u16 => "Shards of Orr",
    709u16 => "The Sound of Psi-Lance",
    710u16 => "Early Parole",
    711u16 => "Magic Sucks",
    712u16 => "A Light in the Darkness",
    713u16 => "The Priory Assailed",
    714u16 => "Under Siege",
    715u16 => "Retribution",
    716u16 => "Retribution",
    719u16 => "The Sound of Psi-Lance",
    726u16 => "Wet Work",
    727u16 => "Shell Shock",
    728u16 => "Volcanic Extraction",
    729u16 => "Munition Acquisition",
    730u16 => "To the Core",
    731u16 => "The Battle of Fort Trinity",
    732u16 => "Tower Down",
    733u16 => "Forging the Pact",
    735u16 => "Willing Captives",
    736u16 => "Marshaling the Truth",
    737u16 => "Breaking the Bone Ship",
    738u16 => "Liberating Apatia",
    739u16 => "Liberating Apatia",
    743u16 => "Fixing the Blame",
    744u16 => "A Sad Duty",
    745u16 => "Striking off the Chains",
    746u16 => "Delivering Justice",
    747u16 => "Intercepting the Orb",
    750u16 => "Close the Eye",
    751u16 => "Through the Looking Glass",
    758u16 => "The Cathedral of Silence",
    760u16 => "Starving the Beast",
    761u16 => "Stealing Light",
    762u16 => "Hunters and Prey",
    763u16 => "Romke's Final Voyage",
    764u16 => "Marching Orders",
    766u16 => "Air Drop",
    767u16 => "Estate of Decay",
    768u16 => "What the Eye Beholds",
    769u16 => "Conscript the Dead Ships",
    772u16 => "Ossuary of Unquiet Dead",
    775u16 => "Temple of the Forgotten God",
    776u16 => "Temple of the Forgotten God",
    777u16 => "Temple of the Forgotten God",
    778u16 => "Through the Looking Glass",
    779u16 => "Starving the Beast",
    780u16 => "Against the Corruption",
    781u16 => "The Source of Orr",
    782u16 => "Armor Guard",
    783u16 => "Blast from the Past",
    784u16 => "The Steel Tide",
    785u16 => "Further Into Orr",
    786u16 => "Ships of the Line",
    787u16 => "Source of Orr",
    788u16 => "Victory or Death",
    789u16 => "A Grisly Shipment",
    790u16 => "Blast from the Past",
    792u16 => "A Pup's Illness",
    793u16 => "Hunters and Prey",
    795u16 => "Legacy of the Foefire",
    796u16 => "The Informant",
    797u16 => "A Traitor's Testimony",
    799u16 => "Follow the Trail",
    806u16 => "Awakening",
    807u16 => "Eye of the North",
    820u16 => "The Omphalos Chamber",
    821u16 => "The Omphalos Chamber",
    825u16 => "Codebreaker",
    827u16 => "Caer Aval",
    828u16 => "The Durmand Priory",
    830u16 => "Vigil Headquarters",
    833u16 => "Ash Tribune Quarters",
    845u16 => "Shattered Light",
    862u16 => "Reaper's Rumble",
    863u16 => "Ascent to Madness",
    864u16 => "Lunatic Inquisition",
    865u16 => "Mad King's Clock Tower",
    866u16 => "Mad King's Labyrinth",
    872u16 => "Fractals of the Mists",
    873u16 => "Southsun Cove",
    875u16 => "Temple of the Silent Storm",
    877u16 => "Snowball Mayhem",
    878u16 => "Tixx's Infinirarium",
    880u16 => "Toypocalypse",
    881u16 => "Bell Choir Ensemble",
    882u16 => "Winter Wonderland",
    894u16 => "Spirit Watch",
    895u16 => "Super Adventure Box",
    896u16 => "North Nolan Hatchery",
    897u16 => "Cragstead",
    899u16 => "Obsidian Sanctum",
    900u16 => "Skyhammer",
    901u16 => "Molten Furnace",
    905u16 => "Crab Toss",
    911u16 => "Dragon Ball Arena",
    912u16 => "Ceremony and Acrimony—Memorials on the Pyre",
    913u16 => "Hard Boiled—The Scene of the Crime",
    914u16 => "The Dead End",
    915u16 => "Aetherblade Retreat",
    917u16 => "No More Secrets—The Scene of the Crime",
    918u16 => "Aspect Arena",
    919u16 => "Sanctum Sprint",
    920u16 => "Southsun Survival",
    922u16 => "Labyrinthine Cliffs",
    924u16 => "Grandmaster of Om",
    929u16 => "The Crown Pavilion",
    930u16 => "Opening Ceremony",
    931u16 => "Scarlet's Playhouse",
    932u16 => "Closing Ceremony",
    934u16 => "Super Adventure Box",
    935u16 => "Super Adventure Box",
    937u16 => "Scarlet's End",
    943u16 => "The Tower of Nightmares (Public)",
    945u16 => "The Nightmare Ends",
    947u16 => "Fractals of the Mists",
    948u16 => "Fractals of the Mists",
    949u16 => "Fractals of the Mists",
    950u16 => "Fractals of the Mists",
    951u16 => "Fractals of the Mists",
    952u16 => "Fractals of the Mists",
    953u16 => "Fractals of the Mists",
    954u16 => "Fractals of the Mists",
    955u16 => "Fractals of the Mists",
    956u16 => "Fractals of the Mists",
    957u16 => "Fractals of the Mists",
    958u16 => "Fractals of the Mists",
    959u16 => "Fractals of the Mists",
    960u16 => "Fractals of the Mists",
    964u16 => "Scarlet's Secret Lair",
    965u16 => "The Origins of Madness: A Moment's Peace",
    968u16 => "Edge of the Mists",
    971u16 => "The Dead End: A Study in Scarlet",
    973u16 => "The Evacuation of Lion's Arch",
    980u16 => "The Dead End: Celebration",
    984u16 => "Courtyard",
    987u16 => "Lion's Arch: Honored Guests",
    988u16 => "Dry Top",
    989u16 => "Prosperity's Mystery",
    990u16 => "Cornered",
    991u16 => "Disturbance in Brisban Wildlands",
    992u16 => "Fallen Hopes",
    993u16 => "Scarlet's Secret Room",
    994u16 => "The Concordia Incident",
    997u16 => "Discovering Scarlet's Breakthrough",
    998u16 => "The Machine",
    999u16 => "Trouble at Fort Salma",
    1000u16 => "The Waypoint Conundrum",
    1001u16 => "Summit Invitations",
    1002u16 => "Mission Accomplished",
    1003u16 => "Rallying Call",
    1004u16 => "Plan of Attack",
    1005u16 => "Party Politics",
    1006u16 => "Foefire Cleansing",
    1007u16 => "Recalibrating the Waypoints",
    1008u16 => "The Ghosts of Fort Salma",
    1009u16 => "Taimi's Device",
    1010u16 => "The World Summit",
    1011u16 => "Battle of Champion's Dusk",
    1015u16 => "The Silverwastes",
    1016u16 => "Hidden Arcana",
    1017u16 => "Reunion with the Pact",
    1018u16 => "Caithe's Reconnaissance Squad",
    1019u16 => "Fort Trinity",
    1021u16 => "Into the Labyrinth",
    1022u16 => "Return to Camp Resolve",
    1023u16 => "Tracking the Aspect Masters",
    1024u16 => "No Refuge",
    1025u16 => "The Newly Awakened",
    1026u16 => "Meeting the Asura",
    1027u16 => "Pact Assaulted",
    1028u16 => "The Mystery Cave",
    1029u16 => "Arcana Obscura",
    1032u16 => "Prized Possessions",
    1033u16 => "Buried Insight",
    1037u16 => "The Jungle Provides",
    1040u16 => "Hearts and Minds",
    1041u16 => "Dragon's Stand",
    1042u16 => "Verdant Brink",
    1043u16 => "Auric Basin",
    1045u16 => "Tangled Depths",
    1046u16 => "Roots of Terror",
    1048u16 => "City of Hope",
    1050u16 => "Torn from the Sky",
    1051u16 => "Prisoners of the Dragon",
    1052u16 => "Verdant Brink",
    1054u16 => "Bitter Harvest",
    1057u16 => "Strange Observations",
    1058u16 => "Prologue: Rally to Maguuma",
    1062u16 => "Spirit Vale",
    1063u16 => "Southsun Crab Toss",
    1064u16 => "Claiming the Lost Precipice",
    1065u16 => "Angvar's Trove",
    1066u16 => "Claiming the Gilded Hollow",
    1067u16 => "Angvar's Trove",
    1068u16 => "Gilded Hollow",
    1069u16 => "Lost Precipice",
    1070u16 => "Claiming the Lost Precipice",
    1071u16 => "Lost Precipice",
    1072u16 => "Southsun Crab Toss",
    1073u16 => "Guild Initiative Office",
    1074u16 => "Blightwater Shatterstrike",
    1075u16 => "Proxemics Lab",
    1076u16 => "Lost Precipice",
    1078u16 => "Claiming the Gilded Hollow",
    1079u16 => "Deep Trouble",
    1080u16 => "Branded for Termination",
    1081u16 => "Langmar Estate",
    1082u16 => "Langmar Estate",
    1083u16 => "Deep Trouble",
    1084u16 => "Southsun Crab Toss",
    1086u16 => "Save Our Supplies",
    1087u16 => "Proxemics Lab",
    1088u16 => "Claiming the Gilded Hollow",
    1089u16 => "Angvar's Trove",
    1090u16 => "Langmar Estate",
    1091u16 => "Save Our Supplies",
    1092u16 => "Scratch Sentry Defense",
    1093u16 => "Angvar's Trove",
    1094u16 => "Save Our Supplies",
    1095u16 => "Dragon's Stand (Heart of Thorns)",
    1097u16 => "Proxemics Lab",
    1098u16 => "Claiming the Gilded Hollow",
    1099u16 => " Desert Borderlands",
    1100u16 => "Scratch Sentry Defense",
    1101u16 => "Gilded Hollow",
    1104u16 => "Lost Precipice",
    1105u16 => "Langmar Estate",
    1106u16 => "Deep Trouble",
    1107u16 => "Gilded Hollow",
    1108u16 => "Gilded Hollow",
    1109u16 => "Angvar's Trove",
    1110u16 => "Scrap Rifle Field Test",
    1111u16 => "Scratch Sentry Defense",
    1112u16 => "Branded for Termination",
    1113u16 => "Scratch Sentry Defense",
    1115u16 => "Haywire Punch-o-Matic Battle",
    1116u16 => "Deep Trouble",
    1117u16 => "Claiming the Lost Precipice",
    1118u16 => "Save Our Supplies",
    1121u16 => "Gilded Hollow",
    1122u16 => "Claiming the Gilded Hollow",
    1123u16 => "Blightwater Shatterstrike",
    1124u16 => "Lost Precipice",
    1126u16 => "Southsun Crab Toss",
    1128u16 => "Scratch Sentry Defense",
    1129u16 => "Langmar Estate",
    1130u16 => "Deep Trouble",
    1131u16 => "Blightwater Shatterstrike",
    1132u16 => "Claiming the Lost Precipice",
    1133u16 => "Branded for Termination",
    1134u16 => "Blightwater Shatterstrike",
    1135u16 => "Branded for Termination",
    1136u16 => "Proxemics Lab",
    1137u16 => "Proxemics Lab",
    1138u16 => "Save Our Supplies",
    1139u16 => "Southsun Crab Toss",
    1140u16 => "Claiming the Lost Precipice",
    1142u16 => "Blightwater Shatterstrike",
    1146u16 => "Branded for Termination",
    1147u16 => "Spirit Vale",
    1149u16 => "Salvation Pass",
    1153u16 => "Tiger Den",
    1154u16 => "Special Forces Training Area",
    1155u16 => "Lion's Arch Aerodrome",
    1156u16 => "Stronghold of the Faithful",
    1158u16 => "Noble's Folly",
    1159u16 => "Research in Rata Novus",
    1161u16 => "Eir's Homestead",
    1163u16 => "Revenge of the Capricorn",
    1164u16 => "Fractals of the Mists",
    1165u16 => "Bloodstone Fen",
    1166u16 => "Confessor's Stronghold",
    1167u16 => "A Shadow's Deeds",
    1169u16 => "Rata Novus",
    1170u16 => "Taimi's Game",
    1171u16 => "Eternal Coliseum",
    1172u16 => "Dragon Vigil",
    1173u16 => "Taimi's Game",
    1175u16 => "Ember Bay",
    1176u16 => "Taimi's Game",
    1177u16 => "Fractals of the Mists",
    1178u16 => "Bitterfrost Frontier",
    1180u16 => "The Bitter Cold",
    1181u16 => "Frozen Out",
    1182u16 => "Precocious Aurene",
    1185u16 => "Lake Doric",
    1188u16 => "Bastion of the Penitent",
    1189u16 => "Regrouping with the Queen",
    1190u16 => "A Meeting of Ministers",
    1191u16 => "Confessor's End",
    1192u16 => "The Second Vision",
    1193u16 => "The First Vision",
    1194u16 => "The Sword Regrown",
    1195u16 => "Draconis Mons",
    1196u16 => "Heart of the Volcano",
    1198u16 => "Taimi's Pet Project",
    1200u16 => "Hall of the Mists",
    1201u16 => "Asura Arena",
    1202u16 => "White Mantle Hideout",
    1203u16 => "Siren's Landing",
    1204u16 => "Palace Temple",
    1205u16 => "Fractals of the Mists",
    1206u16 => "Mistlock Sanctuary",
    1207u16 => "The Last Chance",
    1208u16 => "Shining Blade Headquarters",
    1209u16 => "The Sacrifice",
    1210u16 => "Crystal Oasis",
    1211u16 => "Desert Highlands",
    1212u16 => "Office of the Chief Councilor",
    1214u16 => "Windswept Haven",
    1215u16 => "Windswept Haven",
    1217u16 => "Sparking the Flame",
    1219u16 => "Enemy of My Enemy: The Beastmarshal",
    1220u16 => "Sparking the Flame (Prologue)",
    1221u16 => "The Way Forward",
    1222u16 => "Claiming Windswept Haven",
    1223u16 => "Small Victory (Epilogue)",
    1224u16 => "Windswept Haven",
    1226u16 => "The Desolation",
    1227u16 => "Hallowed Ground: Tomb of Primeval Kings",
    1228u16 => "Elon Riverlands",
    1230u16 => "Facing the Truth: The Sanctum",
    1231u16 => "Claiming Windswept Haven",
    1232u16 => "Windswept Haven",
    1234u16 => "To Kill a God",
    1236u16 => "Claiming Windswept Haven",
    1240u16 => "Blazing a Trail",
    1241u16 => "Night of Fires",
    1242u16 => "Zalambur's Office",
    1243u16 => "Windswept Haven",
    1244u16 => "Claiming Windswept Haven",
    1245u16 => "The Departing",
    1246u16 => "Captain Kiel's Office",
    1247u16 => "Enemy of My Enemy",
    1248u16 => "Domain of Vabbi",
    1250u16 => "Windswept Haven",
    1252u16 => "Crystalline Memories",
    1253u16 => "Beast of War",
    1255u16 => "Enemy of My Enemy: The Troopmarshal",
    1256u16 => "The Dark Library",
    1257u16 => "Spearmarshal's Lament",
    1260u16 => "Eye of the Brandstorm",
    1263u16 => "Domain of Istan",
    1264u16 => "Hall of Chains",
    1265u16 => "The Hero of Istan",
    1266u16 => "Cave of the Sunspear Champion",
    1267u16 => "Fractals of the Mists",
    1268u16 => "Fahranur, the First City",
    1270u16 => "Toypocalypse",
    1271u16 => "Sandswept Isles",
    1274u16 => "The Charge",
    1275u16 => "Courtyard",
    1276u16 => "The Test Subject",
    1277u16 => "The Charge",
    1278u16 => "???",
    1279u16 => "ERROR: SIGNAL LOST",
    1281u16 => "A Kindness Repaid",
    1282u16 => "Tracking the Scientist",
    1283u16 => "???",
    1285u16 => "???",
    1288u16 => "Domain of Kourna",
    1289u16 => "Seized",
    1290u16 => "Fractals of the Mists",
    1291u16 => "Forearmed Is Forewarned",
    1292u16 => "Be My Guest",
    1294u16 => "Sun's Refuge",
    1295u16 => "Legacy",
    1296u16 => "Storm Tracking",
    1297u16 => "A Shattered Nation",
    1299u16 => "Storm Tracking",
    1300u16 => "From the Ashes—The Deadeye",
    1301u16 => "Jahai Bluffs",
    1302u16 => "Storm Tracking",
    1303u16 => "Mythwright Gambit",
    1304u16 => "Mad King's Raceway",
    1305u16 => "Djinn's Dominion",
    1306u16 => "Secret Lair of the Snowmen (Squad)",
    1308u16 => "Scion & Champion",
    1309u16 => "Fractals of the Mists",
    1310u16 => "Thunderhead Peaks",
    1313u16 => "The Crystal Dragon",
    1314u16 => "The Crystal Blooms",
    1315u16 => "Armistice Bastion",
    1316u16 => "Mists Rift",
    1317u16 => "Dragonfall",
    1318u16 => "Dragonfall",
    1319u16 => "Descent",
    1320u16 => "The End",
    1321u16 => "Dragonflight",
    1322u16 => "Epilogue",
    1323u16 => "The Key of Ahdashim",
    1326u16 => "Dragon Bash Arena",
    1327u16 => "Dragon Arena Survival",
    1328u16 => "Auric Span",
    1329u16 => "Coming Home",
    1330u16 => "Grothmar Valley",
    1331u16 => "Strike Mission: Shiverpeaks Pass (Public)",
    1332u16 => "Strike Mission: Shiverpeaks Pass (Squad)",
    1334u16 => "Deeper and Deeper",
    1336u16 => "A Race to Arms",
    1338u16 => "Bad Blood",
    1339u16 => "Weekly Strike Mission: Boneskinner (Squad)",
    1340u16 => "Weekly Strike Mission: Voice of the Fallen and Claw of the Fallen (Public)",
    1341u16 => "Weekly Strike Mission: Fraenir of Jormag (Squad)",
    1342u16 => "The Invitation",
    1343u16 => "Bjora Marches",
    1344u16 => "Weekly Strike Mission: Fraenir of Jormag (Public)",
    1345u16 => "What's Left Behind",
    1346u16 => "Weekly Strike Mission: Voice of the Fallen and Claw of the Fallen (Squad)",
    1349u16 => "Silence",
    1351u16 => "Weekly Strike Mission: Boneskinner (Public)",
    1352u16 => "Secret Lair of the Snowmen (Public)",
    1353u16 => "Celestial Challenge",
    1355u16 => "Voice in the Deep",
    1356u16 => "Chasing Ghosts",
    1357u16 => "Strike Mission: Whisper of Jormag (Public)",
    1358u16 => "Eye of the North",
    1359u16 => "Strike Mission: Whisper of Jormag (Squad)",
    1361u16 => "The Nightmare Incarnate",
    1362u16 => "Forging Steel (Public)",
    1363u16 => "New Friends, New Enemies—North Nolan Hatchery",
    1364u16 => "The Battle for Cragstead",
    1366u16 => "Darkrime Delves",
    1368u16 => "Forging Steel (Squad)",
    1369u16 => "Canach's Lair",
    1370u16 => "Eye of the North",
    1371u16 => "Drizzlewood Coast",
    1372u16 => "Turnabout",
    1373u16 => "Pointed Parley",
    1374u16 => "Strike Mission: Cold War (Squad)",
    1375u16 => "Snapping Steel",
    1376u16 => "Strike Mission: Cold War (Public)",
    1378u16 => "Behind Enemy Lines",
    1379u16 => "One Charr, One Dragon, One Champion",
    1380u16 => "Epilogue",
    1382u16 => "Arena of the Wolverine",
    1383u16 => "A Simple Negotiation",
    1384u16 => "Fractals of the Mists",
    1385u16 => "Caledon Forest (Private)",
    1386u16 => "Thunderhead Peaks (Private)",
    1387u16 => "Bloodtide Coast (Public)",
    1388u16 => "Snowden Drifts (Private)",
    1389u16 => "Snowden Drifts (Public)",
    1390u16 => "Fireheart Rise (Public)",
    1391u16 => "Brisban Wildlands (Private)",
    1392u16 => "Primordus Rising",
    1393u16 => "Lake Doric (Public)",
    1394u16 => "Bloodtide Coast (Private)",
    1395u16 => "Thunderhead Peaks (Public)",
    1396u16 => "Gendarran Fields (Public)",
    1397u16 => "Metrica Province (Public)",
    1398u16 => "Fields of Ruin (Public)",
    1399u16 => "Brisban Wildlands (Public)",
    1400u16 => "Fields of Ruin (Private)",
    1401u16 => "Metrica Province (Private)",
    1402u16 => "Lake Doric (Private)",
    1403u16 => "Caledon Forest (Public)",
    1404u16 => "Fireheart Rise (Private)",
    1405u16 => "Gendarran Fields (Private)",
    1407u16 => "Council Level",
    1408u16 => "Wildfire",
    1409u16 => "Dragonstorm (Private Squad)",
    1410u16 => "Champion's End",
    1411u16 => "Dragonstorm (Public)",
    1412u16 => "Dragonstorm",
    1413u16 => "The Twisted Marionette (Public)",
    1414u16 => "The Twisted Marionette (Private Squad)",
    1415u16 => "The Future in Jade: Power Plant",
    1416u16 => "Deepest Secrets: Yong Reactor",
    1419u16 => "Isle of Reflection",
    1420u16 => "Fallout: Nika's Blade",
    1421u16 => "???",
    1422u16 => "Dragon's End",
    1426u16 => "Isle of Reflection",
    1427u16 => "Weight of the World: Lady Joon's Estate",
    1428u16 => "Arborstone",
    1429u16 => "The Cycle, Reborn: Arborstone",
    1430u16 => "Claiming the Isle of Reflection",
    1432u16 => "Strike Mission: Aetherblade Hideout",
    1433u16 => "Old Friends",
    1434u16 => "Empty",
    1435u16 => "Isle of Reflection",
    1436u16 => "Extraction Point: Command Quarters",
    1437u16 => "Strike Mission: Harvest Temple",
    1438u16 => "New Kaineng City",
    1439u16 => "The Only One",
    1440u16 => "Laying to Rest",
    1442u16 => "Seitung Province",
    1444u16 => "Isle of Reflection",
    1445u16 => "The Future in Jade: Nahpui Lab",
    1446u16 => "Aetherblade Armada",
    1448u16 => "The Cycle, Reborn: The Dead End Bar",
    1449u16 => "Aurene's Sanctuary",
    1450u16 => "Strike Mission: Xunlai Jade Junkyard",
    1451u16 => "Strike Mission: Kaineng Overlook",
    1452u16 => "The Echovald Wilds",
    1453u16 => "Ministry of Security: Main Office",
    1454u16 => "The Scenic Route: Kaineng Docks",
    1456u16 => "Claiming the Isle of Reflection",
    1457u16 => "Detention Facility",
    1458u16 => "Aurene's Sanctuary",
    1459u16 => "Claiming the Isle of Reflection",
    1460u16 => "Empress Ihn's Court",
    1461u16 => "Zen Daijun Hideaway",
    1462u16 => "Isle of Reflection",
    1463u16 => "Claiming the Isle of Reflection",
    1464u16 => "Fallout: Arborstone",
    1465u16 => "Thousand Seas Pavilion",
    1466u16 => "A Quiet Celebration—Knut Whitebear's Loft",
    1467u16 => "New Friends, New Enemies—The Command Core",
    1468u16 => "The Battle for Cragstead—Knut Whitebear's Loft",
    1469u16 => "New Friends, New Enemies—Blood Tribune Quarters",
    1470u16 => "A Quiet Celebration—Citadel Stockade",
    1471u16 => "Case Closed—The Dead End",
    1472u16 => "Hard Boiled—The Dead End",
    1474u16 => "Picking Up the Pieces",
    1477u16 => "The Tower of Nightmares (Private Squad)",
    1478u16 => "The Battle for Lion's Arch (Private Squad)",
    1480u16 => "The Twisted Marionette",
    1481u16 => "Battle on the Breachmaker",
    1482u16 => "The Battle For Lion's Arch (Public)",
    1483u16 => "Memory of Old Lion's Arch",
    1484u16 => "North Evacuation Camp",
    1485u16 => "Strike Mission: Old Lion's Court",
    1487u16 => "The Aether Escape",
    1488u16 => "On the Case: Excavation Yard",
    1489u16 => "A Raw Deal: Red Duck Tea House",
    1490u16 => "Gyala Delve",
    1491u16 => "Deep Trouble: Excavation Yard",
    1492u16 => "Deep Trouble: The Deep",
    1494u16 => "Entrapment: The Deep",
    1495u16 => "A Plan Emerges: Power Plant",
    1496u16 => "Emotional Release: Jade Pools",
    1497u16 => "Emotional Release: Command Quarters",
    1498u16 => "Full Circle: Red Duck Tea House",
    1499u16 => "Forward",
    1500u16 => "Fractals of the Mists",
};
