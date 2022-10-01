use std::io::Write;

use indextree::Arena;
use jokolay::jmf::pack;
use pack::{xml::get_zpack_from_taco, ZPack};
use semver::Version;

fn main() {
    // {use tracing::info;

    //     tracing_subscriber::fmt().init();
    //     color_eyre::install()?;
    //     info!("Application Name: {}", env!("CARGO_PKG_NAME"));
    //     info!("Application Version: {}", env!("CARGO_PKG_VERSION"));
    //     info!("Application Authors: {}", env!("CARGO_PKG_AUTHORS"));
    //     info!(
    //         "Application Repository Link: {}",
    //         env!("CARGO_PKG_REPOSITORY")
    //     );
    //     info!("Application License: {}", env!("CARGO_PKG_LICENSE"));

    //     info!("git version details: {}", jmf::build::SHORT_COMMIT);

    //     info!("created app and initialized logging");
    //     Ok(())
    // }
    let tw_zip = std::fs::read("./assets/tw.zip").expect("failed ot read tw.zip");
    let timer = std::time::Instant::now();

    let (zpack, cats_toggle_status, failures) =
        get_zpack_from_taco(&tw_zip, Version::new(0, 0, 0)).expect("failed ot get pack from taco");
    assert_eq!(zpack.cats.len(), cats_toggle_status.len());
    for fw in failures.warnings {
        println!("{}", fw);
    }
    for fe in failures.errors {
        println!("{}", fe);
    }
    dbg!(timer.elapsed());
    let zkyv = rkyv::to_bytes::<_, 100000>(&zpack).expect("failed to serialize data");
    println!("pack into bytes: {}", timer.elapsed().as_secs_f32());
    std::fs::write("./assets/tekkit.rkyv", &zkyv).expect("failed to write to tekkit rkyv");
    // let zkyv = std::fs::read("./assets/tekkit.rkyv").expect("failed to read tekkit rkyv");
    use mmarinus::{perms, Map, Private};
    let zkyv = Map::load("./assets/tekkit.rkyv", Private, perms::Read).unwrap();
    println!("archive mmaped: {}", timer.elapsed().as_secs_f32());
    let pack =
        rkyv::check_archived_root::<ZPack>(&zkyv).expect("failed to deserialize tekkit rkyv");
    println!("archive checked: {}", timer.elapsed().as_secs_f32());

    // std::thread::sleep(std::time::Duration::from_secs(10));
    {
        let mut f = std::fs::File::create("./assets/tekkit_text.txt")
            .expect("failed to create tekkit text");
        f.write_all(
            format!(
                "cats: {}\ntbins: {}\ntext: {}\ntextures: {}\nmaps: {}\n",
                pack.cats.len(),
                pack.tbins.len(),
                pack.text.len(),
                pack.textures.len(),
                pack.maps.len()
            )
            .as_bytes(),
        )
        .unwrap();

        for (map, markers) in pack.maps.iter() {
            f.write_all(
                format!(
                    "map: {}, markers: {}, trails: {}\n",
                    map,
                    markers.markers.len(),
                    markers.trails.len()
                )
                .as_bytes(),
            )
            .unwrap();
        }
        let mut arena = Arena::new();
        let mut nodes = vec![];
        println!("starting arena : {}", timer.elapsed().as_secs_f32());
        for (cat_index, cat) in pack.cats.iter().copied().enumerate() {
            let n = arena.new_node((
                cat_index,
                pack.text[cat.display_name as usize].as_str(),
                cat.display_name,
            ));
            nodes.push(n);
            if cat_index != 0 {
                nodes[cat.parent_id as usize].append(n, &mut arena);
            }
        }
        println!("arena populated: {}", timer.elapsed().as_secs_f32());
        dbg!(arena.count());

        f.write_all(format!("{:?}", nodes[0].debug_pretty_print(&arena)).as_bytes())
            .expect("failed ot write to tekkit text");
        f.flush().unwrap();
        println!("arena printed to file: {}", timer.elapsed().as_secs_f32());
    }
}
