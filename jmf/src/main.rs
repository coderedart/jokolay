

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

        let _ = dbg!(jmf::xmlpack::load::xml_to_json_pack(std::path::Path::new("./assets/tw")));

        
        
    }
}
// pub fn main() {
    
// }