use std::io::Write;

use jmf::json::Pack;

fn main() {
    {
        {
            use simplelog::*;
            let config = ConfigBuilder::default()
                .set_location_level(LevelFilter::Error)
                .add_filter_ignore_str("oxipng")
                .build();

            CombinedLogger::init(vec![
                TermLogger::new(
                    log::LevelFilter::Debug,
                    config.clone(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    log::LevelFilter::Trace,
                    config,
                    std::fs::File::create("./jmf.log").unwrap(),
                ),
            ])
            .unwrap();
        }

        let (pack, errors) =
            jmf::xmlpack::load::xml_to_json_pack(std::path::Path::new("./assets/packs/tw"));
        std::thread::sleep(std::time::Duration::from_secs(5));

        log::warn!("{:#?}", &errors);
        std::fs::File::create("./assets/packs/pack.json")
            .unwrap()
            .write_all(serde_json::to_string(&pack.unwrap()).unwrap().as_bytes())
            .unwrap();
        // let pack_file = std::io::BufReader::new( std::fs::File::open("./assets/packs/pack.json").unwrap());
        // let pack: Pack = serde_json::from_reader(pack_file).unwrap();
        // std::thread::sleep(std::time::Duration::from_secs(30));
    }
}
// pub fn main() {

// }