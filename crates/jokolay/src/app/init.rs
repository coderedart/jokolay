use cap_std::{ambient_authority, fs_utf8::camino::Utf8PathBuf, fs_utf8::Dir};
use miette::{Context, IntoDiagnostic, Result};
/// Jokolay Configuration
/// We will read a path from env `JOKOLAY_DATA_DIR` or create a folder at data_local_dir/jokolay, where data_local_dir is platform specific
/// Inside this directory, we will store all of jokolay's data like configuration files, themes, logs etc..
pub fn get_jokolay_dir() -> Result<cap_std::fs_utf8::Dir> {
    let authoratah = ambient_authority();
    let jdir = if let Ok(env_dir) = std::env::var("JOKOLAY_DATA_DIR") {
        let jkl_path = Utf8PathBuf::try_from(&env_dir)
            .into_diagnostic()
            .wrap_err(env_dir)
            .wrap_err("failed to parse JOKOLAY_DATA_DIR")?;

        cap_std::fs_utf8::Dir::create_ambient_dir_all(&jkl_path, authoratah)
            .into_diagnostic()
            .wrap_err(jkl_path.clone())
            .wrap_err("failed to create jokolay directory")?;
        Dir::open_ambient_dir(&jkl_path, authoratah)
            .into_diagnostic()
            .wrap_err(jkl_path)
            .wrap_err("failed to open jokolay data dir")?
    } else {
        let dir = cap_directories::ProjectDirs::from("com.jokolay", "", "jokolay", authoratah)
            .ok_or(miette::miette!(
                "getting project dirs failed for some reason"
            ))?
            .data_local_dir()
            .into_diagnostic()
            .wrap_err("failed ot get data local dir using capstd")?;
        Dir::from_cap_std(dir) // into utf-8 dir
    };
    Ok(jdir)
}
