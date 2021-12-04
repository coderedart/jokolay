#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use std::time::Instant;
    use std::{io::Write, path::PathBuf};
    // use std::io::BufWriter;
    use jokolink::mlink::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE};
    use jokolink::win::{create_link_shared_mem, get_xid};
    use log::debug;
    use std::io::{Seek, SeekFrom};
    // get all the cmd line args and initialize logs.
    let (mumble_link_key, mumble_refresh_interval, gw2_check_interval) =
        jokolink::cli::create_app();
    let key: &str = &mumble_link_key;

    // create shared memory using the mumble link key
    let link = create_link_shared_mem(key);
    debug!("created shared memory. pointer: {:?}", link);

    // check that we created shared memory successfully or panic. get ptr to shared memory
    let link_ptr = link.map_err(|e| {
        log::error!(
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
        log::error!(
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
                            log::error!("could not get xid. error: {:?}", &e);
                            e
                        })
                        .ok();
                    // successfully got xid
                    if let Some(id) = xid {
                        debug!("mumble link is initialized. got xid");
                        debug!("Mumble Link data: {:?}", unsafe { *link_ptr });
                        buffer[USEFUL_C_MUMBLE_LINK_SIZE..].copy_from_slice(&id.to_ne_bytes());
                        log::debug!("xid of gw2 window: {:?}", xid);
                    }
                }
            } else {
                log::debug!("the MumbleLink is not init yet. ");
            }
        }

        // write buffer to the file
        shm.write(&buffer).map_err(|e| {
            log::error!(
                "could not write to shared memory file due to error: {:?}",
                &e
            );
            e
        })?;
        // seek back so that we will write to file again from start
        shm.seek(SeekFrom::Start(0)).map_err(|e| {
            log::error!(
                "could not seek to start of shared memory file due to error: {:?}",
                &e
            );
            e
        })?;
    }
}

#[cfg(target_os = "linux")]
fn main() {}
