use std::{io::Write, path::Path};

use super::{xml_category::XMLMarkerCategory, xml_marker::POI, xml_trail::Trail};
use crate::jsonpack::json_cat::{CatSelectionTree, JsonCat};
use jokotypes::*;

pub async fn json_to_xml_zip(
    mut jpack: crate::jsonpack::single_json::SinglePack,
    save_path: &Path,
) {
    let tdir = tempfile::tempdir().expect("failed to create temp dir");
    let td_path = tdir.path().to_path_buf();
    let images_dir = td_path.join("images");
    std::fs::create_dir(&images_dir).expect("failed to create images folder in temp dir");
    let images = jpack.pack_data.images;
    for (hash, ibytes) in images {
        let ipath = images_dir.join(format!("{}.png", hash));
        std::fs::File::create(ipath)
            .expect("failed to create png file in temp dir")
            .write_all(&ibytes)
            .expect("failed to write to png file in temp dir");
    }
    let trails_dir = td_path.join("trails");
    std::fs::create_dir(&trails_dir).expect("failed to create trails folder in temp dir");
    for (hash, tbytes) in jpack.pack_data.tbins.into_iter() {
        let tpath = trails_dir.join(format!("{}.trl", hash));

        let mut tdf = std::fs::File::create(tpath).expect("failed to create trl file in temp dir");
        tdf.write_all(bytemuck::cast_slice(&[
            0_u32,
            u16::from(
                jpack
                    .pack
                    .tbins_descriptions
                    .get(&hash)
                    .expect("failed to get trail_description")
                    .map_id,
            ) as u32,
        ]))
        .expect("failed to write to trail file in temp dir");
        tdf.write_all(bytemuck::cast_slice(&tbytes))
            .expect("failed to write to trl file in temp dir");
    }
    let mut markers = vec![];
    let mut trails = vec![];
    let mut mc_tree = vec![];
    let mut id_names_map = UOMap::new();
    fill_id_names(&jpack.pack.cattree, &jpack.pack.cats, "", &mut id_names_map);
    sc_tree_to_mc_tree(
        &jpack.pack.cattree,
        &mut jpack.pack.cats,
        &mut markers,
        &mut trails,
        &mut mc_tree,
        "",
        &id_names_map,
    );
    let od = super::xml_file::OverlayData {
        categories: if !mc_tree.is_empty() {
            Some(mc_tree)
        } else {
            None
        },
        pois: if !markers.is_empty() || !trails.is_empty() {
            Some(super::xml_marker::POIs {
                tags: Some({
                    let mut poi_or_trail_vec = vec![];
                    poi_or_trail_vec.extend(markers.into_iter().map(|m| m.into()));
                    poi_or_trail_vec.extend(trails.into_iter().map(|t| t.into()));
                    poi_or_trail_vec
                }),
            })
        } else {
            None
        },
    };
    let xml_file = std::fs::File::create(td_path.join("pack.xml"))
        .expect("failed to create xml file in temp dir");
    let writer = std::io::BufWriter::new(xml_file);
    quick_xml::se::to_writer(writer, &od).expect("failed to write to xml file in temp dir");
    zip_extensions::zip_create_from_directory(&save_path.to_path_buf(), &td_path)
        .expect("failed to create zip file from temp dir");
}

pub fn fill_id_names(
    sc_tree: &[CatSelectionTree],
    sc_map: &UOMap<CategoryID, JsonCat>,
    prefix: &str,
    id_names_map: &mut UOMap<CategoryID, String>,
) {
    for cst in sc_tree {
        let id = cst.id;
        if let Some(cat) = sc_map.get(&id) {
            let full_name = if prefix.is_empty() {
                cat.cat_description.name.clone()
            } else {
                let mut current = prefix.to_string();
                current.push('.');
                current.push_str(&cat.cat_description.name);
                current
            };
            fill_id_names(&cst.children, sc_map, &full_name, id_names_map);
            id_names_map.insert(id, full_name.to_string());
        }
    }
}

pub fn sc_tree_to_mc_tree(
    sc_tree: &[CatSelectionTree],
    sc_map: &mut UOMap<CategoryID, JsonCat>,
    markers: &mut Vec<POI>,
    trails: &mut Vec<Trail>,
    mc_tree: &mut Vec<XMLMarkerCategory>,
    prefix: &str,
    id_names_map: &UOMap<CategoryID, String>,
) {
    let images_dir_name = "images/";
    let trails_dir_name = "trails/";
    for cst in sc_tree {
        if let Some(sc) = sc_map.remove(&cst.id) {
            let mut mc = XMLMarkerCategory {
                name: sc.cat_description.name,
                is_separator: sc.cat_description.is_separator,
                display_name: sc.cat_description.display_name,
                ..Default::default()
            };

            let cat_name = id_names_map
                .get(&sc.cat_description.id)
                .expect("failed to find full name of Category");

            for (map_id, map_markers) in sc.map_markers {
                for (_, m) in map_markers.markers.into_iter() {
                    let xp = POI::from_json_marker(
                        m,
                        map_id.into(),
                        cat_name.to_string(),
                        images_dir_name,
                        id_names_map,
                    );
                    markers.push(xp);
                }
                for (_, t) in map_markers.trails.into_iter() {
                    let xt = Trail::from_json_trail(
                        t,
                        cat_name.clone(),
                        images_dir_name,
                        trails_dir_name,
                    );
                    trails.push(xt);
                }
            }
            if !cst.children.is_empty() {
                let mut children = vec![];
                sc_tree_to_mc_tree(
                    &cst.children,
                    sc_map,
                    markers,
                    trails,
                    &mut children,
                    prefix,
                    id_names_map,
                );
                mc.children = Some(children);
            }
            mc_tree.push(mc);
        }
    }
}
