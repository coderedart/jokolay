#[cfg(windows)]
mod win_main {

    use miette::eyre::bail;
    use miette::eyre::WrapErr;
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use std::time::Instant;
    use std::{io::Write, str::FromStr};
    use tracing::metadata::LevelFilter;
    use tracing::*;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Memory::UnmapViewOfFile;
    // use std::io::BufWriter;
    use jokolink::{create_link_shared_mem, get_xid};
    use jokolink::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE};
    use std::io::{Seek, SeekFrom};

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
    pub struct JokolinkConfig {
        pub loglevel: String,
        pub logdir: PathBuf,
        pub mumble_link_names: Vec<String>,
        pub interval: u32,
        pub copy_dest_dir: PathBuf,
    }

    impl Default for JokolinkConfig {
        fn default() -> Self {
            Self {
                loglevel: "info".to_string(),
                logdir: PathBuf::from("."),
                mumble_link_names: vec!["MumbleLink".to_string()],
                interval: 5,
                copy_dest_dir: PathBuf::from("z:\\dev\\shm"),
            }
        }
    }

    pub fn win_main() {
        std::panic::set_hook(Box::new(move |info| {
            tracing::error!("{:#?}", info);
        }));
        // get all the cmd line args and initialize logs.

        let config = std::env::args()
            .nth(1)
            .expect("failed to get second argument. \nUsage: jokolink path/to/config.json");
        let config = std::path::PathBuf::from(config);
        if !config.exists() {
            std::fs::File::create(&config)
                .wrap_err("failed to create config file")
                .unwrap()
                .write_all(
                    serde_json::to_string_pretty(&JokolinkConfig::default())
                        .expect("failed to serialize default config")
                        .as_bytes(),
                )
                .expect("failed to write default config file");
        }
        let config: JokolinkConfig = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(&config).expect("failed to open config file"),
        ))
        .expect("failed to deserialize config file");

        let _guard = log_init(
            tracing::level_filters::LevelFilter::from_str(&config.loglevel)
                .expect("failed to deserialize log level"),
            &config.logdir,
            Path::new("jokolink.log"),
        )
        .expect("failed to init log");
        fake_main(config).unwrap();
    }
    const LIVE_MUMBLE_DATA_BUFFER_SIZE: usize =
        USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<u32>() + std::mem::size_of::<u32>();
    struct LiveMumbleData {
        pub key: String,
        pub mfile: std::fs::File,
        pub previous_tick: u32,
        pub previous_pid: u32,
        pub xid: u32,
        pub xid_tries: u32,
        pub handle: windows::Win32::Foundation::HANDLE,
        pub link_ptr: *const CMumbleLink,
        /// buffer to hold mumble link + xid of gw2 window data + jokolink counter
        pub buffer: [u8; LIVE_MUMBLE_DATA_BUFFER_SIZE],
    }
    impl Drop for LiveMumbleData {
        fn drop(&mut self) {
            unsafe {
                UnmapViewOfFile(self.link_ptr as *const std::ffi::c_void);
                CloseHandle(self.handle);
            }
        }
    }
    fn fake_main(config: JokolinkConfig) -> miette::Result<()> {
        let refresh_inverval = Duration::from_millis(config.interval as u64);

        info!("Application Name: {}", env!("CARGO_PKG_NAME"));
        info!("Application Version: {}", env!("CARGO_PKG_VERSION"));
        info!("Application Authors: {}", env!("CARGO_PKG_AUTHORS"));
        info!(
            "Application Repository Link: {}",
            env!("CARGO_PKG_REPOSITORY")
        );
        info!("Application License: {}", env!("CARGO_PKG_LICENSE"));

        // info!("git version details: {}", git_version::git_version!());

        info!(
            "the file log lvl: {:?}, the logfile directory: {:?}",
            &config.loglevel, &config.logdir
        );
        info!("created app and initialized logging");
        info!("the mumble link names: {:#?}", &config.mumble_link_names);
        info!(
            "the mumble refresh interval in milliseconds: {:#?}",
            refresh_inverval
        );

        info!(
            "the path to which we write mumble data: {:#?}",
            &config.copy_dest_dir
        );
        let mut live_data = vec![];
        for mumble_key in config.mumble_link_names.into_iter() {
            // create shared memory using the mumble link key
            let link = create_link_shared_mem(&mumble_key);
            info!("created shared memory. pointer: {:?}", link);
            let dest_path = config.copy_dest_dir.join(&mumble_key);
            // check that we created shared memory successfully or panic. get ptr to shared memory
            // as we don't really create more than one ptr for the whole lifetime of the program, we will just leak instead of cleaning up handle/link-ptr
            let (handle, link_ptr) =
                link.wrap_err("unabled to create mumble link shared memory ")?;

            // create a shared memory file in /dev/shm/mumble_link_key_name so that jokolay can mumble stuff from there.
            info!(
                "creating the path to destination shm file: {:?}",
                &dest_path
            );

            let shm = std::fs::File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(&dest_path)
                .with_context(|| {
                    format!("failed to create shm file with path {:#?}", &dest_path)
                })?;

            // variable to hold the xid.
            let xid = 0u32;
            // no point in us getting xid for a stale mumble link. so, we make sure to set the previous uitick so that
            // only if tick changes (gw2 is live) then we go check pid / xid
            let previous_tick = unsafe { (*link_ptr).ui_tick };
            let previous_pid = 0u32;

            let xid_tries = 0u32;
            let buffer = [0u8; LIVE_MUMBLE_DATA_BUFFER_SIZE];
            live_data.push(LiveMumbleData {
                key: mumble_key,
                mfile: shm,
                previous_tick,
                previous_pid,
                xid,
                xid_tries,
                handle,
                link_ptr,
                buffer,
            })
        }
        // use a timer to check how long has it been since last timer reset
        let mut timer = Instant::now();
        let mut counter = 0u32;
        loop {
            counter += 1;
            if timer.elapsed() > Duration::from_secs(30) {
                timer = Instant::now();
                warn!("none of the mumble links uitick has not updated in over 30 seconds");
            }
            for data in live_data.iter_mut() {
                let link_ptr = data.link_ptr;
                // copy the bytes from mumble link into shared memory file
                let present_tick = CMumbleLink::get_ui_tick(link_ptr);
                let present_pid = CMumbleLink::get_pid(link_ptr);
                let previous_tick = data.previous_tick;
                if present_tick != previous_tick {
                    data.previous_tick = present_tick;
                    timer = Instant::now();
                    if CMumbleLink::is_valid(link_ptr) {
                        if data.previous_pid != present_pid {
                            let previous_pid = data.previous_pid;
                            data.previous_pid = present_pid;
                            warn!("link_name: {}. pid of gw2 has changed from {previous_pid} to {present_pid}. going to get xid now", &data.key);
                            data.xid = 0;
                        }
                        // if xid is zero, there's no point in writing mumble data to buffer
                        if data.xid == 0 {
                            data.xid = match get_xid(link_ptr) {
                                Ok(present_xid) => {
                                    info!("link_name: {}. xid for gw2 process_id {present_pid} is {present_xid}.", &data.key);
                                    // write xid
                                    data.buffer[USEFUL_C_MUMBLE_LINK_SIZE
                                        ..(USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<u32>())]
                                        .copy_from_slice(&present_xid.to_ne_bytes());
                                    data.xid_tries = 0;
                                    present_xid
                                }
                                Err(e) => {
                                    error!("xid try {}: link_name: {}.failed to get xid for gw2 process with pid {present_pid} due to error {:#?}", data.xid_tries, &data.key, e);
                                    data.xid_tries += 1;
                                    if data.xid_tries > 1000 {
                                        bail!(
                                        "link_name: {}. failed to get xid after too many tries. so, we just quit", &data.key
                                    );
                                    }
                                    0
                                }
                            };
                        } else {
                            // xid is not zero, so we write to buffer now
                            CMumbleLink::copy_raw_bytes_into(
                                link_ptr,
                                &mut data.buffer[..USEFUL_C_MUMBLE_LINK_SIZE],
                            );
                        }
                    }
                }

                // write jokolink counter after xid
                data.buffer[(USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<u32>())..]
                    .copy_from_slice(&counter.to_ne_bytes());
                // write buffer to the file
                data.mfile
                    .write(&data.buffer)
                    .wrap_err("could not write to shared memory file due to error")?;
                // seek back so that we will write to file again from start
                data.mfile
                    .seek(SeekFrom::Start(0))
                    .wrap_err("could not seek to start of shared memory file due to error")?;
            }
            // we sleep for a few milliseconds to avoid reading mumblelink too many times. we will read it around 100 to 200 times per second
            std::thread::sleep(refresh_inverval);
        }
    }

    /// initializes global logging backend that is used by log macros
    /// Takes in a filter for stdout/stderr, a filter for logfile and finally the path to logfile
    pub fn log_init(
        file_filter: LevelFilter,
        log_directory: &Path,
        log_file_name: &Path,
    ) -> miette::Result<tracing_appender::non_blocking::WorkerGuard> {
        // let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
        let file_path = log_directory.join(log_file_name);
        let writer =
            std::io::BufWriter::new(std::fs::File::create(&file_path).wrap_err_with(|| {
                format!("failed to create logfile at path: {:#?}", &file_path)
            })?);
        let (nb, guard) = tracing_appender::non_blocking(writer);
        tracing_subscriber::fmt()
            .with_writer(nb)
            .with_max_level(file_filter)
            .init();

        Ok(guard)
    }
}
#[cfg(windows)]
fn main() {
    win_main::win_main();
}

#[cfg(not(windows))]
fn main() {
    panic!("no binary for non-windows platforms");
}
