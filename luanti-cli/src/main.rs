//! This crate provides some command line tools for administering a Luanti server.
//!
//! The current implementation is an incomplete stub at the moment.

mod config_file;

use std::{
    env,
    path::{Path, PathBuf},
};

use log::{LevelFilter, debug, error};

const CONFIG_FILE_NAME: &str = "minetest.conf";
const GAME_CONFIG_FILE_NAME: &str = "game.conf";
const GAMES_DIR_NAME: &str = "games";
const WORLDS_DIR_NAME: &str = "worlds";

// further reading:
// Look into `subgames.cpp/findSubgame` for the search algorithm used by Luanti to find the game.

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Trace)
        .init();

    // minetest.conf
    // server.conf
    // mod.conf
    // game.conf
    // modpack.conf

    let Some(install_location) = find_install_config() else {
        error!("no installation (minetest.conf) found in current directory or parent directories");
        return;
    };
}

fn find_install_config() -> Option<PathBuf> {
    debug!("searching for installation in current directory …");
    match env::current_dir().as_deref() {
        Ok(mut current_dir) => loop {
            debug!("searching for installation in {}", current_dir.display());
            let local_config = current_dir.join(CONFIG_FILE_NAME);
            if local_config.is_file() {
                debug!("found installation at {}", local_config.display());
                return Some(local_config);
            }
            let Some(parent) = current_dir.parent() else {
                debug!("no installation found in local directory");
                return None;
            };
            current_dir = parent;
        },
        Err(error) => {
            debug!("could not determine current working directory: {error}");
            None
        }
    }
}

fn check_installation_root(path: &Path) -> Result<PathBuf, String> {
    let config_path = path.join(CONFIG_FILE_NAME);
    if !config_path.is_file() {
        return Err(format!(
            "{CONFIG_FILE_NAME} not found in {path}",
            path = path.display()
        ));
    }

    // games sometimes contain a minetest.conf file, which needs to be ignored for searching an installation's root
    let game_config_path = path.join(GAME_CONFIG_FILE_NAME);
    if game_config_path.is_file() {
        return Err(format!(
            "{GAME_CONFIG_FILE_NAME} found in {path}",
            path = path.display()
        ));
    }

    // TODO this might not be the case for newly created servers
    // usually a `games` subdirectory needs to exist next to a `minetest.conf` file
    let games_path = path.join(GAMES_DIR_NAME);
    if !games_path.is_dir() {
        return Err(format!(
            "{GAMES_DIR_NAME} not found in {path}",
            path = path.display()
        ));
    }

    // TODO this might not be the case for newly created servers
    // usually a `worlds` subdirectory needs to exist next to a `minetest.conf` file
    let worlds_path = path.join(WORLDS_DIR_NAME);
    if !worlds_path.is_dir() {
        return Err(format!(
            "{WORLDS_DIR_NAME} not found in {path}",
            path = path.display()
        ));
    }

    debug!("found installation root at {}", path.display());
    Ok(config_path)
}
