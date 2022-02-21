use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct JokolinkCli {
    #[clap(short, long)]
    pub config: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JokolinkConfig {
    pub loglevel: String,
    pub logdir: PathBuf,
    pub mumble: String,
    pub interval: u64,
    pub gwcheck: u64,
    pub destpath: PathBuf,
}

impl Default for JokolinkConfig {
    fn default() -> Self {
        Self {
            loglevel: "debug".to_string(),
            logdir: PathBuf::from("."),
            mumble: "MumbleLink".to_string(),
            interval: 5,
            gwcheck: 1,
            destpath: PathBuf::from("z:\\dev\\shm\\MumbleLink"),
        }
    }
}

#[cfg(target_os = "windows")]
fn main() {
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("{:#?}", info);
    }));
    // get all the cmd line args and initialize logs.

    use std::{io::Write, str::FromStr};
    let app = JokolinkCli::parse();
    if !app.config.exists() {
        std::fs::File::create(&app.config)
            .context("failed to create config file")
            .unwrap()
            .write_all(
                serde_json::to_string_pretty(&JokolinkConfig::default())
                    .expect("failed to serialize default config")
                    .as_bytes(),
            )
            .expect("failed to write default config file");
    }
    let config: JokolinkConfig = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open(&app.config).expect("failed to open config file"),
    ))
    .expect("failed to deserialize config file");

    let _guard = log_init(
        tracing::level_filters::LevelFilter::from_str(&config.loglevel)
            .expect("failed to deserialize log level"),
        &config.logdir,
        Path::new("jokolink.log"),
    )
    .expect("failed to init log");
    if let Err(e) = fake_main(config) {
        tracing::error!("fake_main exited with error: {e:#?}");
    }
}
#[cfg(target_os = "windows")]
fn fake_main(config: JokolinkConfig) -> anyhow::Result<()> {
    use std::time::Duration;
    use tracing::*;

    use anyhow::{bail, Context as _};
    use std::io::Write;
    use std::time::Instant;
    // use std::io::BufWriter;
    use jokolink::mlink::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE};
    use jokolink::win::{create_link_shared_mem, get_process_handle, get_xid};
    use std::io::{Seek, SeekFrom};

    let mumble_key = config.mumble;
    let dest_path = config.destpath;
    let refresh_inverval = Duration::from_millis(config.interval);
    let gw2_check_interval = Duration::from_secs(config.gwcheck);

    info!("Application Name: {}", env!("CARGO_PKG_NAME"));
    info!("Application Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Application Authors: {}", env!("CARGO_PKG_AUTHORS"));
    info!(
        "Application Repository Link: {}",
        env!("CARGO_PKG_REPOSITORY")
    );
    info!("Application License: {}", env!("CARGO_PKG_LICENSE"));

    info!("git version details: {}", git_version::git_version!());

    info!(
        "the file log lvl: {:?}, the logfile directory: {:?}",
        &config.loglevel, &config.logdir
    );
    info!("created app and initialized logging");
    info!("the mumble link name: {}", &mumble_key);
    info!(
        "the mumble refresh interval in milliseconds: {:#?}",
        refresh_inverval
    );
    info!(
        "the gw2 exit check interval in seconds: {:#?}",
        gw2_check_interval
    );
    info!("the path to which we write mumble data: {:#?}", dest_path);

    // create shared memory using the mumble link key
    let link = create_link_shared_mem(&mumble_key);
    info!("created shared memory. pointer: {:?}", link);

    // check that we created shared memory successfully or panic. get ptr to shared memory
    // as we don't really create more than one ptr for the whole lifetime of the program, we will just leak instead of cleaning up handle/link-ptr
    let (_handle, link_ptr) = link.context("unabled to create mumble link shared memory ")?;

    // create a shared memory file in /dev/shm/mumble_link_key_name so that jokolay can mumble stuff from there.
    info!(
        "creating the path to destination shm file: {:?}",
        &dest_path
    );

    let mut shm = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&dest_path)
        .with_context(|| format!("failed to create shm file with path {:#?}", &dest_path))?;

    // variable to hold the xid.
    let mut xid = None;
    let mut process_handle = None;

    // buffer to hold mumble link and xid of gw2 window data + jokolink counter
    let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE
        + std::mem::size_of::<isize>()
        + std::mem::size_of::<usize>()];

    // use a timer to check how long has it been since last timer reset
    let mut timer = Instant::now();
    let mut counter = 0_usize;
    loop {
        // copy the bytes from mumble link into shared memory file
        CMumbleLink::copy_raw_bytes_into(link_ptr, &mut buffer[..USEFUL_C_MUMBLE_LINK_SIZE]);

        buffer[(USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<usize>())..]
            .copy_from_slice(&counter.to_ne_bytes());
        counter += 1;
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
                        info!("mumble link is initialized. got xid");
                        info!("Mumble Link data: {:?}", unsafe { *link_ptr });
                        buffer[USEFUL_C_MUMBLE_LINK_SIZE
                            ..(USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<usize>())]
                            .copy_from_slice(&id.to_ne_bytes());
                        info!("xid of gw2 window: {:?}", xid);
                    }
                }
                if let Some(ph) = process_handle {
                    if let Ok(alive) = jokolink::win::check_process_alive(ph) {
                        if !alive {
                            error!("gw2 is not running anymore. exiting...");
                            jokolink::win::close_process_handle(ph);
                            break;
                        }
                    } else {
                        bail!("failed to get gw2's alive status");
                    }
                } else {
                    info!("trying to get process handle");
                    process_handle = get_process_handle(jokolink::win::get_gw2_pid(link_ptr))
                        .map_err(|e| {
                            error!("failed to get process handle due to error: {e:#?}");
                        })
                        .ok();
                }
            } else {
                info!("the MumbleLink is not init yet. ");
            }
        }

        // write buffer to the file
        shm.write(&buffer)
            .context("could not write to shared memory file due to error")?;
        // seek back so that we will write to file again from start
        shm.seek(SeekFrom::Start(0))
            .context("could not seek to start of shared memory file due to error")?;

        // we sleep for a few milliseconds to avoid reading mumblelink too many times. we will read it around 100 to 200 times per second
        std::thread::sleep(refresh_inverval);
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    panic!("no binary for non-windows platforms");
}

use anyhow::Context;
use std::path::{Path, PathBuf};
use tracing::metadata::LevelFilter;
/// initializes global logging backend that is used by log macros
/// Takes in a filter for stdout/stderr, a filter for logfile and finally the path to logfile
pub fn log_init(
    file_filter: LevelFilter,
    log_directory: &Path,
    log_file_name: &Path,
) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    // let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
    let file_path = log_directory.join(log_file_name);
    let writer = std::io::BufWriter::new(
        std::fs::File::create(&file_path)
            .with_context(|| format!("failed to create logfile at path: {:#?}", &file_path))?,
    );
    let (nb, guard) = tracing_appender::non_blocking(writer);
    tracing_subscriber::fmt()
        .with_writer(nb)
        .with_max_level(file_filter)
        .init();

    Ok(guard)
}
