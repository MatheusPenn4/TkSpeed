//! Detecção de jogos instalados no Windows.
//! Somente leitura — não modifica nenhuma configuração.

use serde::Serialize;

/// Jogo instalado detectado via launcher ou filesystem.
#[derive(Debug, Clone, Serialize)]
pub struct InstalledGame {
    /// Chave única: "{launcher}:{id}"
    pub id: String,
    /// Nome exibido ao usuário.
    pub name: String,
    /// Nome do executável do processo (minúsculas, sem .exe).
    pub exe: String,
    /// Launcher: "steam" | "epic" | "riot" | "battlenet" | "ea" | "ubisoft" | "gog" | "rockstar"
    pub launcher: String,
    /// Caminho de instalação.
    pub install_path: String,
}

/// Detecta todos os jogos instalados. Somente leitura, sem elevação necessária.
pub fn detect_installed_games() -> Vec<InstalledGame> {
    #[cfg(windows)]
    return imp::detect_all();
    #[cfg(not(windows))]
    return vec![];
}

#[cfg(windows)]
mod imp {
    use super::InstalledGame;
    use crate::{registry, registry_hklm};
    use std::collections::HashSet;
    use std::path::Path;

    pub fn detect_all() -> Vec<InstalledGame> {
        let mut games: Vec<InstalledGame> = Vec::new();
        games.extend(steam());
        games.extend(epic());
        games.extend(riot());
        games.extend(battlenet());
        games.extend(ea_app());
        games.extend(ubisoft());
        games.extend(gog());
        games.extend(rockstar());

        let mut seen = HashSet::new();
        games.retain(|g| seen.insert(g.id.clone()));
        games.sort_by(|a, b| a.name.cmp(&b.name));
        games
    }

    // ── Steam ──────────────────────────────────────────────────────────────────

    const STEAM_KNOWN: &[(&str, u32, &str)] = &[
        ("Counter-Strike 2",        730,     "cs2"),
        ("Dota 2",                  570,     "dota2"),
        ("Apex Legends",            1172470, "r5apex"),
        ("PUBG: Battlegrounds",     578080,  "tslgame"),
        ("Rocket League",           252950,  "rocketleague"),
        ("Rainbow Six Siege",       359550,  "rainbowsix"),
        ("Cyberpunk 2077",          1091500, "cyberpunk2077"),
        ("Grand Theft Auto V",      271590,  "gta5"),
        ("Red Dead Redemption 2",   1174180, "rdr2"),
        ("Elden Ring",              1245620, "eldenring"),
        ("Team Fortress 2",         440,     "hl2"),
        ("Garry's Mod",             4000,    "gmod"),
        ("Rust",                    252490,  "rust"),
        ("Warframe",                230410,  "warframe"),
        ("Monster Hunter: World",   582010,  "monsterhunterworld"),
        ("Deep Rock Galactic",      548430,  "fsd"),
        ("Hades",                   1145360, "hades"),
    ];

    fn steam() -> Vec<InstalledGame> {
        let steam_path = match registry::read_string("Software\\Valve\\Steam", "SteamPath") {
            Ok(Some(p)) => p.replace('/', "\\"),
            _ => return vec![],
        };

        let mut libraries = vec![format!("{}\\steamapps", steam_path)];
        for vdf in [
            format!("{}\\config\\libraryfolders.vdf", steam_path),
            format!("{}\\steamapps\\libraryfolders.vdf", steam_path),
        ] {
            if let Ok(content) = std::fs::read_to_string(&vdf) {
                for path in parse_vdf_paths(&content) {
                    libraries.push(format!("{}\\steamapps", path));
                }
            }
        }

        let mut games = Vec::new();
        for (name, appid, exe) in STEAM_KNOWN {
            for lib in &libraries {
                let manifest = format!("{}\\appmanifest_{}.acf", lib, appid);
                if !Path::new(&manifest).exists() {
                    continue;
                }
                let install_path = acf_value(&manifest, "installdir")
                    .map(|d| format!("{}\\common\\{}", lib, d))
                    .unwrap_or_else(|| lib.clone());
                games.push(InstalledGame {
                    id: format!("steam:{}", appid),
                    name: name.to_string(),
                    exe: exe.to_string(),
                    launcher: "steam".to_string(),
                    install_path,
                });
                break;
            }
        }
        games
    }

    fn parse_vdf_paths(content: &str) -> Vec<String> {
        let mut paths = Vec::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.trim().splitn(5, '"').collect();
            if parts.get(1).map(|s| s.to_lowercase().as_str() == "path").unwrap_or(false) {
                if let Some(val) = parts.get(3) {
                    if !val.is_empty() {
                        paths.push(val.replace("\\\\", "\\"));
                    }
                }
            }
        }
        paths
    }

    fn acf_value(path: &str, key: &str) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let parts: Vec<&str> = line.trim().splitn(5, '"').collect();
            if parts.get(1)?.to_lowercase() == key.to_lowercase() {
                return parts.get(3).map(|s| s.to_string());
            }
        }
        None
    }

    // ── Epic Games ─────────────────────────────────────────────────────────────

    const EPIC_KNOWN: &[(&str, &str)] = &[
        ("fortnite",      "fortniteclient-win64-shipping"),
        ("rocket league", "rocketleague"),
        ("fall guys",     "fallguys"),
        ("the outer worlds", "outerworlds"),
    ];

    fn epic() -> Vec<InstalledGame> {
        let manifests = "C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests";
        let dir = match std::fs::read_dir(manifests) {
            Ok(d) => d,
            Err(_) => return vec![],
        };

        let mut games = Vec::new();
        for entry in dir.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            let content = match std::fs::read_to_string(&p) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let name = match json_str_field(&content, "DisplayName") {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };
            let location = json_str_field(&content, "InstallLocation").unwrap_or_default();
            let name_lower = name.to_lowercase();
            for (key, exe) in EPIC_KNOWN {
                if name_lower.contains(key) {
                    games.push(InstalledGame {
                        id: format!("epic:{}", key.replace(' ', "_")),
                        name: name.clone(),
                        exe: exe.to_string(),
                        launcher: "epic".to_string(),
                        install_path: location.clone(),
                    });
                    break;
                }
            }
        }
        games
    }

    fn json_str_field(json: &str, key: &str) -> Option<String> {
        let pattern = format!("\"{}\"", key);
        let start = json.find(&pattern)?;
        let rest = json[start + pattern.len()..].trim_start();
        let rest = rest.strip_prefix(':')?.trim_start();
        let inner = rest.strip_prefix('"')?;
        let end = inner.find('"')?;
        Some(inner[..end].replace("\\\\", "\\").replace("\\/", "/"))
    }

    // ── Riot Games ─────────────────────────────────────────────────────────────

    fn riot() -> Vec<InstalledGame> {
        const RIOT_TITLES: &[(&str, &str, &str)] = &[
            ("VALORANT",            "Valorant",           "valorant-win64-shipping"),
            ("League of Legends",   "League of Legends",  "leagueclient"),
            ("Teamfight Tactics",   "Teamfight Tactics",  "leagueclient"),
        ];
        let riot_root = "C:\\Riot Games";
        let mut games = Vec::new();
        for (folder, display, exe) in RIOT_TITLES {
            let path = format!("{}\\{}", riot_root, folder);
            if Path::new(&path).exists() {
                games.push(InstalledGame {
                    id: format!("riot:{}", folder.to_lowercase().replace(' ', "_")),
                    name: display.to_string(),
                    exe: exe.to_string(),
                    launcher: "riot".to_string(),
                    install_path: path,
                });
            }
        }
        games
    }

    // ── Battle.net ─────────────────────────────────────────────────────────────

    fn battlenet() -> Vec<InstalledGame> {
        let installed = Path::new("C:\\Program Files (x86)\\Battle.net").exists()
            || registry_hklm::read_string(
                "SOFTWARE\\WOW6432Node\\Battle.net\\Install Info",
                "InstallPath",
            ).ok().flatten().is_some();
        if !installed {
            return vec![];
        }

        const BNET_TITLES: &[(&str, &str, &str)] = &[
            ("C:\\Program Files (x86)\\Overwatch", "Overwatch 2", "overwatch"),
            ("C:\\Program Files (x86)\\Call of Duty", "Call of Duty", "cod"),
            ("C:\\Program Files (x86)\\Diablo IV", "Diablo IV", "diablo4"),
            ("C:\\Program Files (x86)\\World of Warcraft", "World of Warcraft", "wow"),
            ("C:\\Program Files (x86)\\StarCraft II", "StarCraft II", "sc2"),
        ];

        let mut games = Vec::new();
        for (path, name, exe) in BNET_TITLES {
            if Path::new(path).exists() {
                games.push(InstalledGame {
                    id: format!("battlenet:{}", exe),
                    name: name.to_string(),
                    exe: exe.to_string(),
                    launcher: "battlenet".to_string(),
                    install_path: path.to_string(),
                });
            }
        }
        games
    }

    // ── EA App ─────────────────────────────────────────────────────────────────

    fn ea_app() -> Vec<InstalledGame> {
        let installed = registry_hklm::read_string(
            "SOFTWARE\\WOW6432Node\\Electronic Arts\\EA Desktop",
            "DesktopAppPath",
        ).ok().flatten().is_some()
            || Path::new("C:\\Program Files\\Electronic Arts").exists();
        if !installed {
            return vec![];
        }

        const EA_TITLES: &[(&str, &str, &str)] = &[
            ("C:\\Program Files\\EA Games\\Battlefield 2042", "Battlefield 2042", "bf2042"),
            ("C:\\Program Files\\EA Games\\FIFA 24", "EA SPORTS FC 24", "fc24"),
            ("C:\\Program Files\\EA Games\\The Sims 4", "The Sims 4", "ts4"),
        ];

        let mut games = Vec::new();
        for (path, name, exe) in EA_TITLES {
            if Path::new(path).exists() {
                games.push(InstalledGame {
                    id: format!("ea:{}", exe),
                    name: name.to_string(),
                    exe: exe.to_string(),
                    launcher: "ea".to_string(),
                    install_path: path.to_string(),
                });
            }
        }
        games
    }

    // ── Ubisoft Connect ────────────────────────────────────────────────────────

    fn ubisoft() -> Vec<InstalledGame> {
        let _install_dir = match registry_hklm::read_string(
            "SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher",
            "InstallDir",
        ) {
            Ok(Some(d)) => d,
            _ => return vec![],
        };

        let game_ids = match registry_hklm::enumerate_subkeys(
            "SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs",
        ) {
            Ok(ids) => ids,
            Err(_) => return vec![],
        };

        let mut games = Vec::new();
        for gid in &game_ids {
            let sub = format!("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs\\{}", gid);
            let install_dir = match registry_hklm::read_string(&sub, "InstallDir") {
                Ok(Some(d)) => d,
                _ => continue,
            };
            let name = install_dir
                .replace('/', "\\")
                .split('\\')
                .next_back()
                .unwrap_or("Jogo Ubisoft")
                .to_string();
            let exe = name.to_lowercase().replace([' ', '-'], "");
            games.push(InstalledGame {
                id: format!("ubisoft:{}", gid),
                name,
                exe,
                launcher: "ubisoft".to_string(),
                install_path: install_dir,
            });
        }
        games
    }

    // ── GOG Galaxy ────────────────────────────────────────────────────────────

    fn gog() -> Vec<InstalledGame> {
        let ids = match registry_hklm::enumerate_subkeys("SOFTWARE\\WOW6432Node\\GOG.com\\Games") {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let mut games = Vec::new();
        for gid in &ids {
            let sub = format!("SOFTWARE\\WOW6432Node\\GOG.com\\Games\\{}", gid);
            let name = match registry_hklm::read_string(&sub, "GAMENAME") {
                Ok(Some(n)) if !n.is_empty() => n,
                _ => continue,
            };
            let path = registry_hklm::read_string(&sub, "PATH").ok().flatten().unwrap_or_default();
            let exe_raw = registry_hklm::read_string(&sub, "LAUNCHCOMMAND").ok().flatten().unwrap_or_default();
            let exe = exe_raw
                .replace('/', "\\")
                .split('\\')
                .next_back()
                .unwrap_or("game")
                .trim_end_matches(".exe")
                .to_lowercase()
                .to_string();
            games.push(InstalledGame {
                id: format!("gog:{}", gid),
                name,
                exe,
                launcher: "gog".to_string(),
                install_path: path,
            });
        }
        games
    }

    // ── Rockstar Games ─────────────────────────────────────────────────────────

    fn rockstar() -> Vec<InstalledGame> {
        const ROCKSTAR_TITLES: &[(&str, &str, &str)] = &[
            ("C:\\Program Files\\Rockstar Games\\GTA V",                  "Grand Theft Auto V",    "gta5"),
            ("C:\\Program Files\\Rockstar Games\\Red Dead Redemption 2",  "Red Dead Redemption 2", "rdr2"),
        ];

        let mut games = Vec::new();
        for (path, name, exe) in ROCKSTAR_TITLES {
            if Path::new(path).exists() {
                games.push(InstalledGame {
                    id: format!("rockstar:{}", exe),
                    name: name.to_string(),
                    exe: exe.to_string(),
                    launcher: "rockstar".to_string(),
                    install_path: path.to_string(),
                });
            }
        }
        games
    }
}
