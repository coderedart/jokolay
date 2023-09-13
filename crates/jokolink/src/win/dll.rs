#![allow(non_snake_case)]

arcdps::arcdps_export! {
    name: "jokolink",
    // This is just "joko" as hex bytes
    sig: 0x6a6f6b6f,
    init: init,
    release: release,
}

fn init() -> ::core::result::Result<(), Box<dyn std::error::Error>> {
    println!("jokolink init called by arcdps. spawning background thread for jokolink");
    unsafe { spawn_jokolink_thread() };
    Ok(())
}
/// If no other thread has been spawned, this will spawn a new thread where jokolink will run
unsafe fn spawn_jokolink_thread() {
    if d3d11::JOKOLINK_THREAD_HANDLE.is_none() {
        let (quit_request_sender, quit_request_receiver) = std::sync::mpsc::sync_channel(0);
        let (quit_response_sender, quit_response_receiver) = std::sync::mpsc::sync_channel(1);

        d3d11::JOKOLINK_QUIT_REQUESTER = Some(quit_request_sender);
        d3d11::JOKOLINK_QUIT_RESPONDER = Some(quit_response_receiver);

        match std::thread::Builder::new()
            .name("jokolink thread".to_string())
            .spawn(move || {
                d3d11::wine::wine_main(quit_request_receiver, quit_response_sender);
                "jokolink thread quit"
            }) {
            Ok(handle) => {
                println!("spawned jokolink thread. handle: {handle:?}");
                d3d11::JOKOLINK_THREAD_HANDLE = Some(handle);
            }
            Err(e) => {
                eprintln!("failed to spawn jokolink thread due to error {e:#?}");
            }
        }
    } else {
        println!("jokolink thread has already been initialized, so skipping initialization.");
    }
}
/// This is really unsafe, so we have to be careful
/// We cannot directly terminate thread because it might lead to some syncronization issues and cause a crash/deadlock
/// we HAVE to terminate the thread because otherwise, it will crash gw2 too.
/// So, we use channels to send a signal to jokolink thread to quit.
/// Then, we use another channel to wait and receive a signal that will be sent by jokolink thread when it terminates.
///
/// We can't call `join` on the thread handle because.. like i said, it can lead to a deadlock/crash.
/// This applies whether we are loaded by game as d3d11.dll or by arcdps as an addon.
unsafe fn terminate_jokolink_thread() {
    if let Some(sender) = d3d11::JOKOLINK_QUIT_REQUESTER.take() {
        if let Err(e) = sender.send(()) {
            eprintln!("failed to send quit signal due to error {e:#?}");
        } else {
            println!("successfully sent the quit signal to the jokolink thread");
        }
    }
    if let Some(receiver) = d3d11::JOKOLINK_QUIT_RESPONDER.take() {
        match receiver.recv() {
            Ok(_) => {
                println!("received quit response from jokolink thread");
            }
            Err(e) => {
                eprintln!("failed to receive quit response from jokolink thread. {e:#?}");
            }
        }
    }
    if let Some(handle) = d3d11::JOKOLINK_THREAD_HANDLE.take() {
        if handle.is_finished() {
            println!("jokolink thread is finished");
        } else {
            println!("jokolink thread is not yet finished, so waiting for it by joining the handle :((((");
            match handle.join() {
                Ok(o) => {
                    println!("joined jokolink thread with return value: {o}");
                }
                Err(e) => {
                    eprintln!("jokolink thread panic: {e:?}");
                }
            }
        }
    } else {
        println!("jokolink thread was never started. So, nothing to terminate");
    }
}
fn release() {
    println!("jokolink release called by arcdps.");
    unsafe {
        terminate_jokolink_thread();
    }
}

pub mod d3d11 {
    use std::{
        sync::mpsc::{Receiver, SyncSender},
        thread::JoinHandle,
    };

    use windows::{
        core::*,
        Win32::Foundation::*,
        Win32::System::{
            LibraryLoader::{GetProcAddress, LoadLibraryA},
            SystemInformation::GetSystemDirectoryA,
            // Threading::{CreateThread, TerminateThread, THREAD_CREATION_FLAGS},
        },
    };

    /// Dll injection basics:
    /// 1. You write a custom dll library exposing functions that match the names/signatures of the actual winapi functions
    /// 2. Then, you place your custom dll library in gw2's executable directory.
    /// 3. gw2 loads your dll and calls your functions thinking it is calling winapi functions.
    /// 4. You will use this chance to do whatever you want, before forwarding the calls to the actual winapi functions
    /// 5. So, we will load the dll from `system32` directory once. store it in [DLL_PTR]
    /// 6. When a function is called, we check if the fn pointer is already loaded. If it is not, we get it from the dll pointer
    static mut DLL_PTR: HMODULE = HMODULE(0);
    static mut CREATE_DEVICE_FNPTR: Option<
        unsafe extern "system" fn(
            padapter: *mut ::core::ffi::c_void,
            drivertype: i32,
            software: HMODULE,
            flags: u32,
            pfeaturelevels: *const i32,
            featurelevels: u32,
            sdkversion: u32,
            ppdevice: *mut *mut ::core::ffi::c_void,
            pfeaturelevel: *mut i32,
            ppimmediatecontext: *mut *mut ::core::ffi::c_void,
        ) -> HRESULT,
    > = None;
    pub static mut JOKOLINK_THREAD_HANDLE: Option<JoinHandle<&'static str>> = None;

    /// This is used to tell wine_main fn thread to quit.
    pub static mut JOKOLINK_QUIT_REQUESTER: Option<SyncSender<()>> = None;
    /// This is used to wait for wine_main fn thread to quit and send us a signal
    pub static mut JOKOLINK_QUIT_RESPONDER: Option<Receiver<()>> = None;
    /// This function is called whenever the dll is loaded into process or thread, and whenever the dll is unloaded out of process/thread.
    /// # Safety
    /// Don't do *anything* complicated at all. It can easily lead to a deadlock
    /// https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices
    /// Improper synchronization within DllMain can cause an application to deadlock or access data or code in an uninitialized DLL.
    #[no_mangle]
    pub unsafe extern "system" fn DllMain(
        _dll_module: HINSTANCE,
        call_reason: u32,
        _: *mut (),
    ) -> bool {
        match call_reason {
            // process detach
            0 => {
                // unlike attach
                println!("jokolink dll is being detached. WINE_MAIN_THREAD_HANDLE is {JOKOLINK_THREAD_HANDLE:?}.");
                super::terminate_jokolink_thread();
            }
            // process attach
            1 => {
                // Sometimes, our dll might be attached/detached multiple times. And we don't want to start jokolink_thread everything time
                // Instead, we only launch our jokolink thread when the D3D11CreateDevice is called
                println!("jokolink dll has been attached. WINE_MAIN_THREAD_HANDLE is {JOKOLINK_THREAD_HANDLE:?}");
            }
            // thread attach and detach
            2 | 3 => {
                // no need to do anything for thread attach and thread detach
            }
            // invalid values
            rest => {
                eprintln!("unrecognized dll main call reason: {rest}");
            }
        }
        true
    }
    /// This is the function we will "hook" into.
    /// GW2 will call this function right after the "login window" when creating the main window
    /// This is where we initialize our jokolink thread.
    /// # Safety
    /// Just need to load d3d11.dll from windows/system32 equivalent directory and call that function for gw2
    #[no_mangle]
    pub unsafe extern "system" fn D3D11CreateDevice(
        padapter: *mut ::core::ffi::c_void,
        drivertype: i32,
        software: HMODULE,
        flags: u32,
        pfeaturelevels: *const i32,
        featurelevels: u32,
        sdkversion: u32,
        ppdevice: *mut *mut ::core::ffi::c_void,
        pfeaturelevel: *mut i32,
        ppimmediatecontext: *mut *mut ::core::ffi::c_void,
    ) -> HRESULT {
        if DLL_PTR.is_invalid() {
            let mut path = [0u8; MAX_PATH as _];
            let len = GetSystemDirectoryA(Some(&mut path)) as usize;
            // we make sure that len is not zero. It means that GetSystemDirectoryA fn didn't fail.
            // we also check if length is above 200, because then we might be reaching the limit of maximum path length supported by windows.
            if len == 0 || len > 200 {
                eprintln!("the system directory path size is: {len}. So, i am quitting");
                return HRESULT::default();
            }
            const D3D11_DLL_PATH: &str = "\\d3d11.dll\0";
            path[len..(len + D3D11_DLL_PATH.len())].copy_from_slice(D3D11_DLL_PATH.as_bytes());

            match LoadLibraryA(PCSTR::from_raw(path.as_ptr())) {
                Ok(p) => {
                    println!("successfully loaded library d3d11.dll ");
                    DLL_PTR = p;
                }
                Err(e) => {
                    eprintln!("could not load d3d11.dll from system path due to error: {e:#?}");
                    return HRESULT::default();
                }
            }
        } else {
            println!("d3d11.dll library is already loaded. So, skipping that");
        }
        if CREATE_DEVICE_FNPTR.is_none() {
            if let Some(p) = GetProcAddress(DLL_PTR, PCSTR("D3D11CreateDevice\0".as_ptr())) {
                println!("successfully got proc address of D3D11CreateDevice");
                let _ = CREATE_DEVICE_FNPTR.insert(std::mem::transmute(p));
            } else {
                eprintln!("could not load address of D3D11CreateDevice");
            }
        } else {
            println!("D3D11CreateDevice fn ptr is already loaded, so skipped that");
        }
        if JOKOLINK_THREAD_HANDLE.is_none() {
            println!("starting jokolink's wine_main on another thrad");

            super::spawn_jokolink_thread();
        }
        println!("calling D3D11CreateDevice fn");
        if let Some(p) = CREATE_DEVICE_FNPTR {
            p(
                padapter,
                drivertype,
                software,
                flags,
                pfeaturelevels,
                featurelevels,
                sdkversion,
                ppdevice,
                pfeaturelevel,
                ppimmediatecontext,
            )
        } else {
            HRESULT::default()
        }
    }

    // unsafe extern "system" fn wine_main(_: *mut ::core::ffi::c_void) -> u32 {
    //     super::spawn_jokolink_thread();
    //     0
    // }
    pub mod wine {
        use crate::mumble::ctypes::*;
        use crate::win::MumbleWinImpl;
        use crate::DEFAULT_MUMBLELINK_NAME;
        use miette::{Context, IntoDiagnostic, Result};
        use serde::{Deserialize, Serialize};
        use std::io::Write;
        use std::io::{Seek, SeekFrom};
        use std::path::{Path, PathBuf};
        use std::str::FromStr;
        use std::sync::mpsc::{Receiver, SyncSender};
        use std::time::Duration;
        use tracing::{error, info};
        use tracing_subscriber::filter::LevelFilter;
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(default)]
        pub struct JokolinkConfig {
            pub loglevel: String,
            pub logdir: PathBuf,
            pub mumble_link_name: String,
            pub interval: u32,
            pub copy_dest_dir: PathBuf,
        }

        impl Default for JokolinkConfig {
            fn default() -> Self {
                Self {
                    loglevel: "info".to_string(),
                    logdir: PathBuf::from("."),
                    mumble_link_name: DEFAULT_MUMBLELINK_NAME.to_string(),
                    interval: 5,
                    copy_dest_dir: PathBuf::from("z:\\dev\\shm"),
                }
            }
        }

        pub fn wine_main(
            quit_request_receiver: Receiver<()>,
            quit_response_sender: SyncSender<()>,
        ) {
            if let Err(e) = std::panic::catch_unwind(move || {
                let config = "./jokolink_config.json".to_string();
                let config = std::path::PathBuf::from(config);
                if !config.exists() {
                    match std::fs::File::create(&config) {
                        Ok(mut f) => match serde_json::to_string_pretty(&JokolinkConfig::default())
                        {
                            Ok(config_string) => {
                                if let Err(e) = f.write_all(config_string.as_bytes()) {
                                    eprintln!(
                                        "failed to write default config file due to error {e:#?}"
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!("failed to serialize default config due to error {e:#?}");
                            }
                        },
                        Err(e) => eprintln!("failed to create config.json due to error {e:#?}"),
                    }
                }
                let config: JokolinkConfig = match std::fs::File::open(&config) {
                    Ok(f) => match serde_json::from_reader(std::io::BufReader::new(f)) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("failed to deserialize config file due to error {e:#?}");
                            return;
                        }
                    },
                    Err(e) => {
                        eprintln!("failed to open config file due to error {e:#?}");
                        return;
                    }
                };
                println!("successfully loaded configuration file");
                match miette::set_hook(Box::new(|_| {
                    Box::new(
                        miette::MietteHandlerOpts::new()
                            .unicode(true)
                            .context_lines(4)
                            .with_cause_chain()
                            .build(),
                    )
                })) {
                    Ok(_) => {
                        println!("miette hook set");
                    }
                    Err(e) => {
                        eprintln!("failed to set miette hook due to {e:#?}");
                    }
                }
                let guard = match log_init(
                    LevelFilter::from_str(&config.loglevel).unwrap_or(LevelFilter::INFO),
                    &config.logdir,
                    Path::new("jokolink.log"),
                ) {
                    Ok(g) => g,
                    Err(e) => {
                        eprintln!("failed to initiailize logging due to error {e:#?}");
                        return;
                    }
                };
                if let Err(e) = fake_main(config, quit_request_receiver) {
                    eprintln!("fake main exited due to error: {e:#?}");
                }
                std::mem::drop(guard);
                println!("dropped logfile guard");
            }) {
                eprintln!("There was a panic in jokolink thread: {e:?}");
            }
            println!("exiting wine_main function");
            match quit_response_sender.send(()) {
                Ok(_) => {
                    println!("successfully sent quit response");
                }
                Err(e) => {
                    eprintln!("failed to send quit response due to: {e:#?}");
                }
            }
        }

        fn fake_main(config: JokolinkConfig, quit_signal: Receiver<()>) -> Result<()> {
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
            info!("the mumble link names: {:#?}", &config.mumble_link_name);
            info!(
                "the mumble refresh interval in milliseconds: {:#?}",
                refresh_inverval
            );

            info!(
                "the path to which we write mumble data: {:#?}",
                &config.copy_dest_dir
            );
            let mumble_key = config.mumble_link_name.clone();

            let dest_path = config.copy_dest_dir.join(&mumble_key);

            // create a shared memory file in /dev/shm/mumble_link_key_name so that jokolay can mumble stuff from there.
            info!(
                "creating the path to destination shm file: {:?}",
                &dest_path
            );

            let mut mfile = std::fs::File::options()
                .write(true)
                .create(true)
                .open(&dest_path)
                .into_diagnostic()
                .wrap_err_with(|| {
                    format!("failed to create shm file with path {:#?}", &dest_path)
                })?;
            // create shared memory using the mumble link key
            let mut source = MumbleWinImpl::new(&mumble_key)?;

            loop {
                if let Err(e) = source.tick() {
                    error!(?e, "mumble tick error");
                }
                let link = source.get_cmumble_link();

                let buffer: [u8; C_MUMBLE_LINK_SIZE_FULL] =
                    unsafe { std::ptr::read_volatile(&link as *const CMumbleLink as *const _) };
                mfile
                    .seek(SeekFrom::Start(0))
                    .into_diagnostic()
                    .wrap_err("could not seek to start of shared memory file due to error")?;

                // write buffer to the file
                mfile
                    .write(&buffer)
                    .into_diagnostic()
                    .wrap_err("could not write to shared memory file due to error")?;
                match quit_signal.try_recv() {
                    Ok(_) => {
                        println!("received quit signal. returning from wine_main()");
                        error!("received quit signal. returning from wine_main()");
                        return Ok(());
                    }
                    Err(e) => match e {
                        std::sync::mpsc::TryRecvError::Empty => {}
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            eprintln!("why is the quit signaller sender disconnected????");
                        }
                    },
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
        ) -> Result<tracing_appender::non_blocking::WorkerGuard> {
            // let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
            let file_path = log_directory.join(log_file_name);
            let writer = std::io::BufWriter::new(
                std::fs::File::create(&file_path)
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        format!("failed to create logfile at path: {:#?}", &file_path)
                    })?,
            );
            let (nb, guard) = tracing_appender::non_blocking(writer);
            tracing_subscriber::fmt()
                .with_writer(nb)
                .with_max_level(file_filter)
                .pretty()
                .with_ansi(false)
                .init();

            Ok(guard)
        }
    }
}
