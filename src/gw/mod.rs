pub mod category;
pub mod marker;
pub mod trail;
pub mod xmltypes;
use std::collections::BTreeMap;
use std::ffi::OsStr;

use crate::gw::{category::MarkerCategory, marker::Marker};
use category::OverlayData;
use trail::Trail;

pub fn load_markers() -> (
    BTreeMap<String, MarkerCategory>,
    BTreeMap<u32, Vec<Marker>>,
    Vec<Trail>,
) {
    use std::fs;
    let mut marker_cats: BTreeMap<String, MarkerCategory> = BTreeMap::new();
    let mut markers_mapid: BTreeMap<u32, Vec<Marker>> = BTreeMap::new();
    let mut trails: Vec<Trail> = vec![];
    for f in fs::read_dir("/home/red/extra/projects/programming/gw2_addons/jokolay/res/tw/")
        .expect("couldn't open directory ./res/tw")
    {
        let entry = f.expect("f to e");

        if entry.path().extension() == Some(OsStr::new("xml")) {
            let testxml = std::fs::File::open(&entry.path()).unwrap();
            let readxml = std::io::BufReader::new(testxml);

            match quick_xml::de::from_reader(readxml) {
                Ok(mfile) => {
                    let mfile: OverlayData = mfile;
                    MarkerCategory::build_categories(
                        "".to_owned(),
                        mfile.categories,
                        &mut marker_cats,
                    );
                    match mfile.pois {
                        Some(pois) => {
                            match pois.poi {
                                Some(poi) => {
                                    for m in poi.into_iter() {
                                        if m.map_id.is_some() {
                                            markers_mapid
                                                .entry(m.map_id.unwrap())
                                                .or_insert(Vec::new())
                                                .push(m);
                                        }
                                    }
                                }
                                None => (),
                            }
                            match pois.trail {
                                Some(trail) => trails.extend(trail),
                                None => (),
                            }
                        }
                        None => (),
                    }
                }
                Err(e) => {
                    eprint!(
                        "failed to deserialize file {:?} due to error {}\n",
                        entry.path(),
                        e
                    )
                }
            }
        }
    }
    (marker_cats, markers_mapid, trails)
}
