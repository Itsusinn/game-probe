#![feature(test, let_chains)]

extern crate test;

use anyhow::{Context, Result};
use manifest::MANIFIEST;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use vdf::Entry;
extern crate steamy_vdf as vdf;

pub mod manifest;

struct Game {
    appid: u64,
    name: String,
    install_dir: PathBuf,
}

fn main() -> Result<()> {
    let steam_path = get_steam_path()?;
    println!("Found Steam installation at: {}", steam_path.display());

    let libraries = get_steam_libraries(&steam_path)?;
    let games = get_installed_games(&libraries)?;

    for game in games {
        println!(
            "{}\n\tAppID: {}\n\tInstall Localation: {}",
            game.name,
            game.appid,
            game.install_dir.display()
        );
        if let Some(profile) = MANIFIEST.get(&game.appid)
            && let Some(files) = &profile.files
        {
            for (path, detail) in files.iter() {
                if let Some(tags) = &detail.tags
                    && tags.contains(&"save".to_string())
                {
                    if let Some(condition) = &detail.when {
                        let condition = &condition[0];
                        if let Some(store) = &condition.store
                            && !store.eq("steam")
                        {
                            continue;
                        }
                        println!(
                            "\tSave Locations: {} OS: {}",
                            path,
                            condition.os.clone().unwrap_or("ALL".to_string())
                        )
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
fn get_steam_path() -> Result<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Valve\\Steam")
        .context("Failed to open Steam registry key")?;

    let path: String = key
        .get_value("SteamPath")
        .context("Failed to read SteamPath from registry")?;

    Ok(PathBuf::from(path.replace("/", "\\")))
}

#[cfg(not(windows))]
fn get_steam_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let paths = [home.join(".steam/steam"), home.join(".local/share/Steam")];

    paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .context("Could not find Steam installation")
}

fn get_steam_libraries(steam_path: &Path) -> Result<Vec<PathBuf>> {
    let mut libraries = vec![steam_path.to_path_buf()];
    let library_file = steam_path.join("steamapps/libraryfolders.vdf");

    if let Ok(root) = vdf::load(&library_file) {
        if let Some(Entry::Table(folders)) = root.lookup("libraryfolders") {
            for (key, value) in folders.iter() {
                if key.parse::<u32>().is_ok() {
                    if let Some(Entry::Value(path)) = value.lookup("path") {
                        libraries.push(PathBuf::from(path.deref()));
                    }
                }
            }
        }
    }

    Ok(libraries)
}

fn get_installed_games(libraries: &[PathBuf]) -> Result<Vec<Game>> {
    let mut games = Vec::new();

    for path in libraries {
        let steamapps = path.join("steamapps");
        if !steamapps.exists() {
            continue;
        }

        for entry in fs::read_dir(&steamapps)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "acf")
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("appmanifest")
            {
                if let Ok(data) = vdf::load(&path) {
                    if let Some(appstate) = data.get("AppState") {
                        let appid = appstate
                            .get("appid")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let name = appstate
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let install_dir = appstate
                            .get("installdir")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();

                        games.push(Game {
                            appid: str::parse(appid)?,
                            name: name.to_string(),
                            install_dir: steamapps.join("common").join(install_dir),
                        });
                    }
                }
            }
        }
    }

    Ok(games)
}
