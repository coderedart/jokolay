use crate::prelude::*;
use cap_std::{ambient_authority, fs::Dir};
use std::path::PathBuf;

/// Jokolay Configuration
/// We will read a path from env `JOKOLAY_DATA_DIR` or create a folder at data_local_dir/jokolay, where data_local_dir is platform specific
/// Inside this directory, we will store all of jokolay's data like configuration files, themes, logs etc..
pub fn get_jokolay_dir() -> Result<(PathBuf, cap_std::fs::Dir)> {
    let authoratah = ambient_authority();
    let jokolay_data_local_dir_path = if let Some(env_dir) = std::env::var("JOKOLAY_DATA_DIR").ok()
    {
        match PathBuf::try_from(&env_dir) {
            Ok(jokolay_dir) => jokolay_dir,
            Err(e) => return Err(miette::miette!("failed to parse JOKOLAY_DATA_DIR: {e}")),
        }
    } else {
        match directories_next::ProjectDirs::from("com.jokolay", "", "jokolay") {
            Some(pd) => pd.data_local_dir().to_path_buf(),
            None => {
                return Err(miette::miette!(
                    "getting project dirs failed for some reason"
                ))
            }
        }
    };
    if jokolay_data_local_dir_path.to_str().is_none() {
        return Err(miette::miette!(
            "jokolay data dir is not utf-8: {jokolay_data_local_dir_path:?}"
        ));
    }
    if let Err(e) =
        cap_std::fs::Dir::create_ambient_dir_all(&jokolay_data_local_dir_path, authoratah)
    {
        return Err(miette::miette!(
            "failed to create jokolay directory at {jokolay_data_local_dir_path:?} due to error: {e}"
        ));
    }
    let jdir = match Dir::open_ambient_dir(&jokolay_data_local_dir_path, authoratah) {
        Ok(jdir) => jdir,
        Err(e) => {
            return Err(miette::miette!(
                "failed to open jokolay data dir at {jokolay_data_local_dir_path:?} due to {e}"
            ))
        }
    };

    Ok((jokolay_data_local_dir_path, jdir))
}
