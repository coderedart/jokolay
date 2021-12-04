use clap::App;
use log::{debug, LevelFilter};
use std::{path::PathBuf, str::FromStr, time::Duration};

pub fn create_app() -> (String, Duration, Duration) {
    let yml = clap::load_yaml!("app.yml");
    let m = App::from_yaml(yml).get_matches();
    let termlog = LevelFilter::from_str(m.value_of("termlog").unwrap_or("info"))
        .expect("could not parse termlog option");
    let filelog = LevelFilter::from_str(m.value_of("filelog").unwrap_or("debug"))
        .expect("could not parse filelog option");
    let logfile = PathBuf::from_str(m.value_of("logfile").unwrap_or("jokolink.log"))
        .expect("could not parse logfile option");
    let refresh_inverval = Duration::from_millis(
        u64::from_str(
            m.value_of("interval")
                .unwrap_or(&MUMBLE_REFRESH_INTERVAL.to_string()),
        )
        .expect("could not parse refresh interval option"),
    );
    let gw2_check_interval = Duration::from_secs(
        u64::from_str(
            m.value_of("gwcheck")
                .unwrap_or(&GW2_EXIT_CHECK_INTERVAL.to_string()),
        )
        .expect("could not parse gw2 check alive option"),
    );

    debug!(
        "the terminal log level: {:?}, the file log lvl: {:?}, the logfile name: {:?}",
        termlog, filelog, &logfile
    );
    crate::log_init(termlog, filelog, logfile).expect("failed to init log");
    debug!("created app and initialized logging");
    let key = m.value_of("mumble").unwrap_or("MumbleLink").to_string();
    debug!("the mumble link name: {}", &key);
    (key, refresh_inverval, gw2_check_interval)
}
const MUMBLE_REFRESH_INTERVAL: u64 = 5;
const GW2_EXIT_CHECK_INTERVAL: u64 = 5;
