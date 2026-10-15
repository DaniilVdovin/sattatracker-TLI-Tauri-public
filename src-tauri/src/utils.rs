use std::path::{Path, PathBuf};
use sysinfo::System;

const LOG_CANDIDATES: &[&str] = &[
    r"app\game\XDGame\Saved\Logs\XDGame.log",
    r"UE_game\TorchLight\Saved\Logs\UE_game.log",
    r"XDGame\Saved\Logs\XDGame.log",
    r"TorchLight\Saved\Logs\UE_game.log",
    r"Saved\Logs\XDGame.log",
    r"Saved\Logs\UE_game.log",
];

const DRIVES: &[char] = &[
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

const STEAM_LIB_NAMES: &[&str] = &[
    "SteamLibrary",
    "Steam",
    r"Games\Steam",
    "SteamGames",
    r"Program Files\Steam",
    r"Program Files (x86)\Steam",
];

pub fn get_game_log_path() -> Option<PathBuf> {
    for name in &["UE_game.log", "XDGame.log"] {
        if Path::new(name).exists() {
            if let Ok(abs) = std::fs::canonicalize(name) {
                return Some(abs);
            }
        }
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    for (_pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name.contains("torchlight") || name.contains("xdgame") {
            if let Some(exe) = process.exe() {
                println!("[utils] Found game process exe: {:?}", exe);
                let mut dir = exe.parent().map(|p| p.to_path_buf());
                for _ in 0..5 {
                    if let Some(base) = dir {
                        for rel in LOG_CANDIDATES {
                            let candidate = base.join(rel);
                            if candidate.exists() {
                                println!("[utils] Log found via process: {:?}", candidate);
                                return Some(candidate);
                            }
                        }
                        dir = base.parent().map(|p| p.to_path_buf());
                    } else {
                        break;
                    }
                }
                println!("[utils] Process found but log not located near it");
            }
        }
    }

    let steam_libs = collect_steam_libraries();
    println!("[utils] Steam libraries found: {:?}", steam_libs);

    for lib in &steam_libs {
        let game_root = lib
            .join("steamapps")
            .join("common")
            .join("Torchlight Infinite");
        for rel in LOG_CANDIDATES {
            let candidate = game_root.join(rel);
            if candidate.exists() {
                println!("[utils] Log found via Steam lib: {:?}", candidate);
                return Some(candidate);
            }
        }
    }

    for &drive in DRIVES {
        for &lib_name in STEAM_LIB_NAMES {
            let lib_root = PathBuf::from(format!("{}:\\{}", drive, lib_name));
            if !lib_root.exists() {
                continue;
            }
            let game_root = lib_root
                .join("steamapps")
                .join("common")
                .join("Torchlight Infinite");
            for rel in LOG_CANDIDATES {
                let candidate = game_root.join(rel);
                if candidate.exists() {
                    println!("[utils] Log found via drive scan: {:?}", candidate);
                    return Some(candidate);
                }
            }
        }
    }

    println!("[utils] Log file not found anywhere");
    None
}

fn collect_steam_libraries() -> Vec<PathBuf> {
    let mut libs: Vec<PathBuf> = Vec::new();

    let steam_install_names = [
        r"Program Files (x86)\Steam",
        r"Program Files\Steam",
        "Steam",
    ];

    for &drive in DRIVES {
        for &name in &steam_install_names {
            let steam_root = PathBuf::from(format!("{}:\\{}", drive, name));
            if !steam_root.exists() {
                continue;
            }

            libs.push(steam_root.clone());

            let vdf = steam_root.join("config").join("libraryfolders.vdf");
            if let Ok(content) = std::fs::read_to_string(&vdf) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("\"path\"") {
                        let mut parts = line.splitn(2, "\"path\"");
                        if let Some(rest) = parts.nth(1) {
                            let rest = rest.trim();
                            if rest.starts_with('"') {
                                let inner = &rest[1..];
                                if let Some(end) = inner.find('"') {
                                    let raw = &inner[..end];
                                    let path_str = raw.replace("\\\\", "\\");
                                    let p = PathBuf::from(&path_str);
                                    if p.exists() && !libs.contains(&p) {
                                        libs.push(p);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    libs
}
