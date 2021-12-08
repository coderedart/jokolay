use std::{io::Write, path::PathBuf, str::FromStr};

use jmf::xmlpack::xml_pack_entry::XmlPackEntries;
use log::{trace, warn};

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

        // let pack_dir = rfd::AsyncFileDialog::new()
        //     .pick_folder()
        //     .await
        //     .unwrap()
        //     .path()
        //     .to_path_buf();
        // let quit_signal = Arc::new(AtomicBool::new(true));
        // std::thread::spawn(|| {
        //     let meter = self_meter::Meter::new(std::time::Duration::from_secs(1)).unwrap();
        //     while quit_signal.load(std::sync::atomic::Ordering::Relaxed) {
        //         meter.scan().unwrap();
        //         if let Some(report) = meter.report() {

        //         }
        //     }
        // });
        let pack_dir = PathBuf::from_str("./assets/tw").unwrap();
        let time = std::time::Instant::now();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        trace!("starting deserializing: {:?}", &pack_dir);
        let (mut pack, errors) = rt.block_on(XmlPackEntries::new(&pack_dir));
        trace!("{:?}", errors);
        trace!("deserialized: {:?}", time.elapsed());

        let time = std::time::Instant::now();
        let mut validation_errors = pack.validate_pack();
        trace!("validated {:?}", time.elapsed());
        validation_errors.sort();
        warn!("{:#?}", &validation_errors);
        trace!("validation errors count: {}", validation_errors.len());
        trace!(
            "starting json_pack conversion {}",
            time::OffsetDateTime::now_utc().time()
        );

        let (status_sender, status_receiver) = flume::unbounded();
        let handle = std::thread::spawn(move || pack.to_json_pack(status_sender, false));
        let mut jpack = None;
        for status in status_receiver.iter() {
            match status {
                jmf::xmlpack::xml_pack_entry::ToJsonPackStatus::Started => {
                    log::trace!("started");
                }
                jmf::xmlpack::xml_pack_entry::ToJsonPackStatus::ProcessingCategories(
                    current,
                    total,
                ) => {
                    log::trace!("categories: current: {},total: {}", current, total);
                }
                jmf::xmlpack::xml_pack_entry::ToJsonPackStatus::ProcessingMarkers(
                    current,
                    total,
                ) => {
                    log::trace!("markers: current: {},total: {}", current, total);
                }
                jmf::xmlpack::xml_pack_entry::ToJsonPackStatus::ProcessingImages(
                    current,
                    total,
                ) => {
                    log::trace!("images: current: {},total: {}", current, total);
                }
                jmf::xmlpack::xml_pack_entry::ToJsonPackStatus::Completed(p) => jpack = Some(p),
            }
        }
        handle.join().unwrap();
        if let Some(jpack) = jpack {
            let pack = *jpack;

            // rt.block_on(pack.save_to_folder(&save_folder)).unwrap();
            let spack = serde_json::to_string(&pack).unwrap();
            std::fs::File::create("./assets/tw_data.json")
                .unwrap()
                .write_all(spack.as_bytes())
                .unwrap();
            // warn!("serialisation done: {:?}", time::OffsetDateTime::now_utc().time());
            // let _dspack: SinglePack = serde_json::from_str(&spack).unwrap();
            // let save_path = std::path::Path::new("./assets/reactif_repeat.zip");
            // rt.block_on(json_to_xml_zip(pack, save_path));
        }
    }
}
