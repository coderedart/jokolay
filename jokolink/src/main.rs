use tracing::*;
use tracing_appender::non_blocking::WorkerGuard;
#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use std::time::Instant;
    use std::{io::Write, path::PathBuf};
    // use std::io::BufWriter;
    use jokolink::mlink::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE};
    use jokolink::win::{create_link_shared_mem, get_xid};
    use std::io::{Seek, SeekFrom};
    // get all the cmd line args and initialize logs.
    let (mumble_link_key, mumble_refresh_interval, gw2_check_interval, guard) =
        jokolink::cli::create_app();
    let key: &str = &mumble_link_key;

    // create shared memory using the mumble link key
    let link = create_link_shared_mem(key);
    debug!("created shared memory. pointer: {:?}", link);

    // check that we created shared memory successfully or panic. get ptr to shared memory
    let link_ptr = link.map_err(|e| {
        error!(
            "unabled to create mumble link shared memory due to error: {:?}",
            &e
        );
        e
    })?;

    // create a shared memory file in /dev/shm/mumble_link_key_name so that jokolay can mumble stuff from there.
    let shmpath: PathBuf = ["z:\\", "dev", "shm", key].iter().collect();
    debug!("the path to destination shm file: {:?}", &shmpath);

    let shm = std::fs::File::create(&shmpath);
    debug!("shm file created. File: {:?}", &shm);
    let mut shm = shm.map_err(|e| {
        error!(
            "unable to create the shared memory file in /dev/shm due to error: {:?}",
            &e
        );
        e
    })?;

    // variable to hold the xid.
    let mut xid = None;

    // buffer to hold mumble link and xid of gw2 window data.
    let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()];

    // use a timer to check how long has it been since last timer reset
    let mut timer = Instant::now();

    loop {
        // copy the bytes from mumble link into shared memory file
        CMumbleLink::copy_raw_bytes_into(link_ptr, &mut buffer[..USEFUL_C_MUMBLE_LINK_SIZE]);
        // we sleep for 10 milliseconds to avoid reading mumblelink too many times. we will read it around 100 times per second
        std::thread::sleep(mumble_refresh_interval);

        // every 5 seconds
        if timer.elapsed() > gw2_check_interval {
            // reset the timer
            timer = Instant::now();
            // check if mumble is initialized
            if CMumbleLink::is_valid(link_ptr) {
                if xid.is_none() {
                    // then get the window id of the gw2 window and write it to the buffer
                    xid = get_xid(link_ptr)
                        .map_err(|e| {
                            error!("could not get xid. error: {:?}", &e);
                            e
                        })
                        .ok();
                    // successfully got xid
                    if let Some(id) = xid {
                        debug!("mumble link is initialized. got xid");
                        debug!("Mumble Link data: {:?}", unsafe { *link_ptr });
                        buffer[USEFUL_C_MUMBLE_LINK_SIZE..].copy_from_slice(&id.to_ne_bytes());
                        debug!("xid of gw2 window: {:?}", xid);
                    }
                }
            } else {
                debug!("the MumbleLink is not init yet. ");
            }
        }

        // write buffer to the file
        shm.write(&buffer).map_err(|e| {
            error!(
                "could not write to shared memory file due to error: {:?}",
                &e
            );
            e
        })?;
        // seek back so that we will write to file again from start
        shm.seek(SeekFrom::Start(0)).map_err(|e| {
            error!(
                "could not seek to start of shared memory file due to error: {:?}",
                &e
            );
            e
        })?;
    }
}

#[cfg(not(windows))]
fn main() {
    panic!("no binary for non-windows platforms");
}

use std::path::Path;
use tracing::metadata::LevelFilter;
/// initializes global logging backend that is used by log macros
/// Takes in a filter for stdout/stderr, a filter for logfile and finally the path to logfile
pub fn log_init(
    file_filter: LevelFilter,
    log_directory: &Path,
    log_file_name: &Path,
) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
    let (nb, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(nb)
        .with_max_level(file_filter)
        .init();

    Ok(guard)
}

use clap::App;
use std::{path::PathBuf, str::FromStr, time::Duration};

pub fn create_app() -> (String, Duration, Duration, WorkerGuard) {
    let yml = clap::load_yaml!("app.yml");
    let m = App::from_yaml(yml).get_matches();
    let log_level = LevelFilter::from_str(m.value_of("log_level").unwrap_or("debug"))
        .expect("could not parse log_level option");
    let logfile_dir = PathBuf::from_str(m.value_of("logfile_dir").unwrap_or("."))
        .expect("could not parse logfile_dir option");
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
        "the file log lvl: {:?}, the logfile directory: {:?}",
        log_level, &logfile_dir
    );
    let guard =
        log_init(log_level, &logfile_dir, Path::new("jokolink.log")).expect("failed to init log");
    debug!("created app and initialized logging");
    let key = m.value_of("mumble").unwrap_or("MumbleLink").to_string();
    debug!("the mumble link name: {}", &key);
    (key, refresh_inverval, gw2_check_interval, guard)
}
const MUMBLE_REFRESH_INTERVAL: u64 = 5;
const GW2_EXIT_CHECK_INTERVAL: u64 = 5;
