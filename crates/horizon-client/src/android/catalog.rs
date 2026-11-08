//! Mobile launcher state. Browser operations reuse the desktop backend unchanged.
use crate::browser::{self, Kind, Project, Query, Source, installed::Installed};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};
pub fn source_only_mod(project: &Project) -> bool {
    project.source == browser::SourceId::Horizon
        && project.loaders.iter().any(|loader| loader == "hnmod")
}

/// Temporary dev registration published as Minecraft Nintendo Switch by PrismarineJS.
/// Account tokens and client secrets are never bundled.
pub const MICROSOFT_CLIENT_ID: &str = "00000000441cc96b";

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccentColor {
    #[default]
    Zinc,
    Emerald,
    Sky,
    Amber,
    Rose,
    Violet,
}

impl AccentColor {
    pub const ALL: [Self; 6] = [
        Self::Zinc,
        Self::Emerald,
        Self::Sky,
        Self::Amber,
        Self::Rose,
        Self::Violet,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Zinc => "Zinc (монохром)",
            Self::Emerald => "Emerald (изумрудный)",
            Self::Sky => "Sky (лазурный)",
            Self::Amber => "Amber (янтарный)",
            Self::Rose => "Rose (коралловый)",
            Self::Violet => "Violet (фиолетовый)",
        }
    }

    pub fn bg(self) -> u32 {
        match self {
            Self::Zinc => horizon_client::hud::rgba(244, 244, 245, 255),
            Self::Emerald => horizon_client::hud::rgba(16, 185, 129, 255),
            Self::Sky => horizon_client::hud::rgba(14, 165, 233, 255),
            Self::Amber => horizon_client::hud::rgba(245, 158, 11, 255),
            Self::Rose => horizon_client::hud::rgba(244, 63, 94, 255),
            Self::Violet => horizon_client::hud::rgba(139, 92, 246, 255),
        }
    }

    pub fn text(self) -> u32 {
        match self {
            Self::Zinc => horizon_client::hud::rgba(15, 17, 23, 255),
            _ => horizon_client::hud::rgba(255, 255, 255, 255),
        }
    }

    pub fn border(self) -> u32 {
        match self {
            Self::Zinc => horizon_client::hud::rgba(255, 255, 255, 200),
            Self::Emerald => horizon_client::hud::rgba(52, 211, 153, 200),
            Self::Sky => horizon_client::hud::rgba(56, 189, 248, 200),
            Self::Amber => horizon_client::hud::rgba(252, 211, 77, 200),
            Self::Rose => horizon_client::hud::rgba(251, 113, 133, 200),
            Self::Violet => horizon_client::hud::rgba(196, 181, 253, 200),
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiScale {
    Compact,
    #[default]
    Normal,
    Large,
}

impl UiScale {
    pub const ALL: [Self; 3] = [
        Self::Compact,
        Self::Normal,
        Self::Large,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "80% (компактный)",
            Self::Normal => "100% (стандартный)",
            Self::Large => "120% (крупный)",
        }
    }

    pub fn factor(self) -> f32 {
        match self {
            Self::Compact => 0.8,
            Self::Normal => 1.0,
            Self::Large => 1.2,
        }
    }
}

/// Real Vulkan rendering paths; these do not select a different driver or API.
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RendererMode {
    Vulkan,
    OpenGl,
    #[default]
    VulkanCompatibility,
}
impl RendererMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vulkan => "Vulkan / GPU batching",
            Self::OpenGl => "OpenGL ES / experimental",
            Self::VulkanCompatibility => "Vulkan / compatibility",
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub profile: u32,
    pub proxy: Option<String>,
    pub name: String,
    pub server: String,
    #[serde(skip)]
    pub microsoft_client_id: String,
    pub renderer: RendererMode,
    pub render_scale: crate::display::RenderScale,
    pub orientation: Orientation,
    #[serde(flatten)]
    pub screen: crate::display::ScreenPreferences,
    pub input_host: InputHost,
    pub keypad_controls: bool,
    pub record_performance: bool,
    pub worker_limit: u32,
    pub cache_budget_mib: u32,
    #[serde(flatten)]
    pub device: crate::device_preferences::Settings,
    pub shader_source: crate::shader_catalog::Origin,
    pub mod_source: crate::shader_catalog::Origin,
    pub accent_color: AccentColor,
    pub ui_scale: UiScale,
}

pub use horizon_client::android_gamepad::InputMode as InputHost;

/// Android ActivityInfo orientations, independent of the device's natural axis.
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Orientation {
    #[default]
    Landscape,
    Portrait,
    ReverseLandscape,
    ReversePortrait,
    Auto,
}
impl Orientation {
    pub const ALL: [Self; 5] = [
        Self::Landscape,
        Self::Portrait,
        Self::ReverseLandscape,
        Self::ReversePortrait,
        Self::Auto,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Landscape => "Landscape (sensor)",
            Self::Portrait => "Portrait (sensor)",
            Self::ReverseLandscape => "Landscape (reversed)",
            Self::ReversePortrait => "Portrait (reversed)",
            Self::Auto => "Automatic rotation",
        }
    }
    pub fn android_value(self) -> i32 {
        match self {
            Self::Landscape => 6,
            Self::Portrait => 7,
            Self::ReverseLandscape => 8,
            Self::ReversePortrait => 9,
            Self::Auto => 10,
        }
    }
}
impl Config {
    pub fn load(home: &Path) -> Self {
        let value: serde_json::Value = std::fs::read(home.join("android-launcher.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let mut config: Self = serde_json::from_value(value.clone()).unwrap_or_default();
        if cfg!(target_pointer_width = "32") && value.get("renderer").is_none() {
            config.renderer = RendererMode::OpenGl;
        }
        // Ignore saved app registrations while this dev build uses the fixed ID.
        config.microsoft_client_id =
            crate::account::client_id().unwrap_or_else(|| MICROSOFT_CLIENT_ID.into());
        if value.get("renderer").is_none() && value["gpu_batching"].as_bool() == Some(true) {
            config.renderer = RendererMode::Vulkan;
        }
        let legacy = std::fs::read(config.game_home(home).join("horizon-touch.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .unwrap_or_default();
        config.device.migrate_missing(&value, &legacy);
        config
    }
    pub fn save(&self, home: &Path) -> Result<(), String> {
        let part = home.join("android-launcher.partial");
        std::fs::write(
            &part,
            serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        std::fs::rename(part, home.join("android-launcher.json")).map_err(|e| e.to_string())
    }
    pub fn profile_home(&self, home: &Path) -> PathBuf {
        if self.profile == 0 {
            home.into()
        } else {
            home.join("profiles")
                .join(format!("profile-{}", self.profile))
        }
    }
    pub fn game_home(&self, home: &Path) -> PathBuf {
        // Legacy modpack selections are ignored; their files remain untouched.
        self.profile_home(home)
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Play,
    Mods,
    Packs,
    Shaders,
    Installed,
    Settings,
}
#[allow(dead_code)]
impl Page {
    pub const ALL: [Self; 6] = [
        Self::Play,
        Self::Settings,
        Self::Packs,
        Self::Shaders,
        Self::Mods,
        Self::Installed,
    ];
    pub const SETTINGS_TABS: [Self; 5] = [
        Self::Settings,
        Self::Packs,
        Self::Shaders,
        Self::Mods,
        Self::Installed,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Mods => "Mods",
            Self::Packs => "Packs",
            Self::Shaders => "Shaders",
            Self::Installed => "Installed",
            Self::Settings => "Settings",
        }
    }
    pub fn tab_title(self) -> &'static str {
        match self {
            Self::Play => "Главная",
            Self::Settings => "Параметры",
            Self::Packs => "Ресурспаки",
            Self::Shaders => "Шейдеры",
            Self::Mods => "Моды",
            Self::Installed => "Установленное",
        }
    }
    pub fn is_catalog(self) -> bool {
        matches!(self, Self::Packs | Self::Mods | Self::Shaders)
    }
}
enum Reply {
    Done(String, horizon_client::content_files::Kind),
}
pub struct Catalog {
    pub local: Vec<LocalItem>,
    pub bundled_enabled: std::collections::BTreeSet<String>,
    pub sections: [bool; 3],
    pub pending_delete: Option<LocalItem>,
    downloaded: Vec<(browser::SourceId, String)>,
    downloading: Option<(browser::SourceId, String)>,
    progress: Arc<Mutex<f32>>,
    failed: Option<(browser::SourceId, String)>,
    pub page: Page,
    pub query: String,
    pub results: Vec<Project>,
    pub status: String,
    pub offset: usize,
    pub scroll: f32,
    pub config: Config,
    pub editing: Option<&'static str>,
    pub render_scale_text: String,
    worker: Option<mpsc::Receiver<Result<Reply, String>>>,
    search_worker: Option<mpsc::Receiver<Result<Vec<Project>, String>>>,
    searches: crate::search_queue::Queue<(Page, browser::SourceId, String, usize), Vec<Project>>,
}
#[derive(Clone)]
pub struct LocalItem {
    pub kind: horizon_client::content_files::Kind,
    pub file: String,
    pub title: String,
    pub enabled: bool,
}
impl Catalog {
    pub fn new(home: &Path) -> Self {
        let config = Config::load(home);
        let mut result = Self {
            local: vec![],
            bundled_enabled: Default::default(),
            sections: [true, false, false],
            pending_delete: None,
            downloaded: vec![],
            downloading: None,
            progress: Arc::new(Mutex::new(0.0)),
            failed: None,
            page: Page::Play,
            query: String::new(),
            results: vec![],
            status: String::new(),
            offset: 0,
            scroll: 0.0,
            config,
            editing: None,
            render_scale_text: String::new(),
            worker: None,
            search_worker: None,
            searches: crate::search_queue::Queue::new(),
        };
        result.refresh(home);
        result
    }
    pub fn show_installed(&mut self, kind: Option<horizon_client::content_files::Kind>) {
        self.searches.cancel_interest();
        self.page = Page::Installed;
        self.scroll = 0.0;
        self.editing = None;
        self.sections = [false; 3];
        if let Some(i) = horizon_client::content_files::Kind::ALL
            .iter()
            .position(|k| Some(*k) == kind)
        {
            self.sections[i] = true;
        }
    }
    pub fn sync_local_mods(game: &Path) {
        let dir = game.join("horizon-mods");
        let native_dir = game.join(horizon_client::mod_list::NATIVE_DIR);
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::create_dir_all(&native_dir);
        let Ok(read) = std::fs::read_dir(&dir) else { return; };
        let mut on = horizon_client::mod_list::ModList::load(game);
        let mut changed = false;

        for entry in read.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let lower = name.to_ascii_lowercase();

            if lower.ends_with(".jar") || (lower.ends_with(".zip") && !lower.ends_with(".hshader")) {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                let cache_dir = game.join(".import-cache");
                let _ = std::fs::create_dir_all(&cache_dir);
                let temp_out = cache_dir.join(format!(".check-{stamp}"));
                let _ = std::fs::remove_dir_all(&temp_out);
                let mut ported_id = None;

                if let Ok(check) = horizon_port_pack::check::check(&path, &temp_out) {
                    let carry = temp_out.join("carry");
                    if carry.join("horizon-mod.toml").is_file() {
                        let target_hmod = dir.join(format!("{}.hmod", check.mod_id));
                        if crate::documents::pack_hmod(&carry, &target_hmod).is_ok() {
                            let _ = std::fs::remove_file(&path);
                            ported_id = Some(check.mod_id);
                        }
                    }
                }

                if ported_id.is_none() {
                    // Fallback porting: extract assets and data
                    let fallback_dir = temp_out.join("fallback");
                    let _ = std::fs::create_dir_all(&fallback_dir);
                    if let Ok(file) = std::fs::File::open(&path) {
                        if let Ok(mut zip) = zip::ZipArchive::new(file) {
                            let mut mod_id = String::new();
                            let mut mod_name = String::new();
                            let mut mod_version = "1.0.0".to_string();

                            for i in 0..zip.len() {
                                let Ok(mut e) = zip.by_index(i) else { continue };
                                if e.name() == "fabric.mod.json" {
                                    let mut text = String::new();
                                    use std::io::Read;
                                    let _ = e.read_to_string(&mut text);
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(id) = v["id"].as_str() { mod_id = id.to_string(); }
                                        if let Some(n) = v["name"].as_str() { mod_name = n.to_string(); }
                                        if let Some(ver) = v["version"].as_str() { mod_version = ver.to_string(); }
                                    }
                                }
                            }
                            if mod_id.is_empty() {
                                mod_id = Path::new(&name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "mod".into());
                            }
                            let mut clean_id = String::new();
                            for c in mod_id.chars() {
                                if c.is_ascii_lowercase() || c.is_ascii_digit() { clean_id.push(c); }
                                else if c.is_ascii_uppercase() { clean_id.push(c.to_ascii_lowercase()); }
                                else if c == '-' || c == '.' || c == '_' { clean_id.push('_'); }
                            }
                            while clean_id.contains("__") { clean_id = clean_id.replace("__", "_"); }
                            let clean_id = clean_id.trim_matches('_').to_string();
                            let clean_id = if clean_id.is_empty() { "mod".to_string() } else { clean_id[..clean_id.len().min(64)].to_string() };
                            if mod_name.is_empty() { mod_name = clean_id.clone(); }

                            for i in 0..zip.len() {
                                let Ok(mut e) = zip.by_index(i) else { continue };
                                let ename = e.name().to_string();
                                if ename.starts_with("assets/") || ename.starts_with("data/") {
                                    let target_p = fallback_dir.join(&ename);
                                    if e.is_dir() {
                                        let _ = std::fs::create_dir_all(&target_p);
                                    } else {
                                        if let Some(p) = target_p.parent() { let _ = std::fs::create_dir_all(p); }
                                        if let Ok(mut out_f) = std::fs::File::create(&target_p) {
                                            let _ = std::io::copy(&mut e, &mut out_f);
                                        }
                                    }
                                }
                            }
                            let manifest = format!("id = \"{clean_id}\"\nname = \"{mod_name}\"\nversion = \"{mod_version}\"\napi = \"0.2\"\nside = \"both\"\ncapabilities = [\"registries\"]\n");
                            let _ = std::fs::write(fallback_dir.join("horizon-mod.toml"), manifest);
                            let target_hmod = dir.join(format!("{clean_id}.hmod"));
                            if crate::documents::pack_hmod(&fallback_dir, &target_hmod).is_ok() {
                                let _ = std::fs::remove_file(&path);
                                ported_id = Some(clean_id);
                            }
                        }
                    }
                }
                let _ = std::fs::remove_dir_all(&temp_out);

                if let Some(id) = ported_id {
                    on.enabled.insert(id.clone());
                    let _ = crate::browser::java::set_enabled(game, &id, true);
                    changed = true;
                }
            } else if lower.ends_with(".hnmod") {
                let mut id = String::new();
                if let Ok(file) = std::fs::File::open(&path) {
                    if let Ok(mut zip) = zip::ZipArchive::new(file) {
                        for i in 0..zip.len() {
                            let Ok(mut e) = zip.by_index(i) else { continue };
                            if e.name().ends_with("horizon-native.json") {
                                use std::io::Read;
                                let mut text = String::new();
                                let _ = e.read_to_string(&mut text);
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(i) = v["id"].as_str() { id = i.to_string(); }
                                }
                            }
                        }
                    }
                }
                if id.is_empty() {
                    id = name.strip_suffix(".hnmod").unwrap_or(&name).to_string();
                }
                let _ = std::fs::copy(&path, native_dir.join(format!("{id}.hnmod")));
                if !on.enabled.contains(&id) && !on.disabled.contains(&id) {
                    on.enabled.insert(id.clone());
                    let _ = crate::browser::java::set_enabled(game, &id, true);
                    changed = true;
                }
            } else if lower.ends_with(".hmod") {
                let mut id = String::new();
                if let Ok(pkg) = horizon_mods::Package::read(&path) {
                    id = pkg.manifest.id;
                } else if let Ok(file) = std::fs::File::open(&path) {
                    if let Ok(mut zip) = zip::ZipArchive::new(file) {
                        for i in 0..zip.len() {
                            let Ok(mut e) = zip.by_index(i) else { continue };
                            if e.name().ends_with("horizon-mod.toml") {
                                use std::io::Read;
                                let mut text = String::new();
                                let _ = e.read_to_string(&mut text);
                                if let Ok(m) = horizon_mods::Manifest::parse(&text) {
                                    id = m.id;
                                }
                            }
                        }
                    }
                }
                if id.is_empty() {
                    id = name.strip_suffix(".hmod").unwrap_or(&name).to_string();
                }
                if !on.enabled.contains(&id) && !on.disabled.contains(&id) {
                    on.enabled.insert(id.clone());
                    let _ = crate::browser::java::set_enabled(game, &id, true);
                    changed = true;
                }
            }
        }
        if changed {
            let _ = on.save(game);
        }
    }

    pub fn refresh(&mut self, home: &Path) {
        let game = self.config.game_home(home);
        Self::sync_local_mods(&game);
        let installed = Installed::load(&game);
        self.downloaded = installed
            .entries
            .iter()
            .filter(|e| game.join(&e.file).is_file())
            .map(|e| (e.source, e.project.clone()))
            .collect();
        let on = horizon_client::mod_list::ModList::load(&game);
        self.bundled_enabled = on.enabled.difference(&on.disabled).cloned().collect();
        let settings = horizon_client::settings::Settings::load(&game.join("options.txt"));
        self.local.clear();
        for kind in horizon_client::content_files::Kind::ALL {
            let mut dirs = vec![game.join(kind.folder())];
            if kind == horizon_client::content_files::Kind::Mods {
                dirs.push(game.join(horizon_client::mod_list::NATIVE_DIR));
            }
            for dir in dirs {
                if std::fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) {
                    continue;
                }
                for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                    let is_file = entry.file_type().is_ok_and(|t| t.is_file());
                    let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
                    if !is_file && !is_dir {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if is_dir {
                        if kind != horizon_client::content_files::Kind::Mods {
                            continue;
                        }
                        if !entry.path().join("horizon-mod.toml").is_file() {
                            continue;
                        }
                    } else if !horizon_client::content_files::plain_name(&name) || !kind.accepts(&name) {
                        continue;
                    }
                    let enabled = match kind {
                        horizon_client::content_files::Kind::Packs => {
                            settings.resource_packs.contains(&format!("file/{name}"))
                        }
                        horizon_client::content_files::Kind::Shaders => {
                            settings.horizon.shader_pack == name
                        }
                        horizon_client::content_files::Kind::Mods => {
                            let id = name
                                .strip_suffix(".hmod")
                                .or_else(|| name.strip_suffix(".hnmod"))
                                .or_else(|| name.strip_suffix(".jar"))
                                .or_else(|| name.strip_suffix(".zip"))
                                .unwrap_or(&name);
                            on.enabled.contains(id)
                        }
                    };
                    if !self.local.iter().any(|item| item.kind == kind && item.file == name) {
                        self.local.push(LocalItem {
                            kind,
                            file: name.clone(),
                            title: name,
                            enabled,
                        });
                    }
                }
            }
        }
        for (id, title, _desc) in horizon_client::mod_list::ANDROID_BUNDLED {
            let enabled = self.bundled_enabled.contains(id);
            if !self.local.iter().any(|item| item.kind == horizon_client::content_files::Kind::Mods && item.file == id) {
                self.local.push(LocalItem {
                    kind: horizon_client::content_files::Kind::Mods,
                    file: id.to_string(),
                    title: format!("{title} (Встроенный)"),
                    enabled,
                });
            }
        }
        self.local.sort_by_key(|e| e.title.to_lowercase());
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some()
    }
    pub fn download_label(&self, project: &Project) -> Option<String> {
        if self
            .downloading
            .as_ref()
            .is_some_and(|(source, id)| *source == project.source && *id == project.id)
        {
            return Some(format!(
                "{:.0}%",
                *self.progress.lock().unwrap_or_else(|e| e.into_inner()) * 100.0
            ));
        }
        if self
            .downloaded
            .iter()
            .any(|(source, id)| *source == project.source && *id == project.id)
            || (project.kind == Kind::Mod
                && self.local.iter().any(|item| {
                    item.kind == horizon_client::content_files::Kind::Mods
                        && (item.file == format!("{}.hmod", project.id) || item.file == project.id)
                }))
        {
            return Some("Installed".into());
        }
        None
    }
    pub fn install_button(&self, project: &Project) -> &'static str {
        if self
            .failed
            .as_ref()
            .is_some_and(|(source, id)| *source == project.source && *id == project.id)
        {
            "Retry"
        } else {
            "Install"
        }
    }
    fn job(&mut self, status: &str, job: impl FnOnce() -> Result<Reply, String> + Send + 'static) {
        if self.busy() {
            return;
        }
        self.status = status.into();
        let (tx, rx) = mpsc::channel();
        self.worker = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(job());
        });
    }
    pub fn poll(&mut self, home: &Path) {
        let reply = self.worker.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(v) => Some(v),
            Err(mpsc::TryRecvError::Disconnected) => Some(Err("Launcher worker stopped".into())),
            Err(_) => None,
        });
        if let Some(reply) = reply {
            self.worker = None;
            let downloading = self.downloading.take();
            if reply.is_err() {
                self.failed = downloading;
            }
            match reply {
                Ok(Reply::Done(s, kind)) => {
                    self.status = s;
                    self.show_installed(Some(kind));
                }
                Err(e) => self.status = e,
            }
            self.refresh(home);
        }
        let result = self
            .search_worker
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(v) => Some(v),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("Search worker stopped".into())),
                Err(_) => None,
            });
        if let Some(result) = result {
            self.search_worker = None;
            let visible = self.searches.finish(result.as_ref().ok().cloned());
            if visible && self.page.is_catalog() && !self.busy() {
                match result {
                    Ok(items) => self.show_results(items),
                    Err(error) => self.status = error,
                }
            }
        }
        self.start_search();
    }
    pub fn search(&mut self, home: &Path) {
        self.request_search(home, true);
    }
    pub fn browse(&mut self, home: &Path) {
        self.searches.cancel_interest();
        if self.page.is_catalog() {
            self.request_search(home, false);
        }
    }
    fn request_search(&mut self, home: &Path, refresh: bool) {
        if !self.page.is_catalog() {
            return;
        }
        if self.page == Page::Mods && self.query.trim().starts_with("https://") {
            self.searches.cancel_interest();
            if refresh {
                self.import_native(home);
            } else {
                self.results.clear();
                self.status = "Press Import .hmod to download this HTTPS link".into();
            }
            return;
        }
        self.scroll = 0.0;
        let source = match self.page {
            Page::Packs => browser::SourceId::Modrinth,
            Page::Shaders => self.config.shader_source.source(),
            Page::Mods => self.config.mod_source.source(),
            _ => browser::SourceId::Horizon,
        };
        let key = (self.page, source, self.query.trim().to_owned(), self.offset);
        if let Some(items) = self.searches.request(key, refresh) {
            self.show_results(items);
        } else {
            self.results.clear();
            self.status = "Searching... You can switch tabs while results load.".into();
        }
        self.start_search();
    }
    fn show_results(&mut self, items: Vec<Project>) {
        self.status = if self.page == Page::Shaders
            && items.is_empty()
            && self.config.shader_source == crate::shader_catalog::Origin::Horizon
        {
            "No Horizon shader packs match this search. Try another query or import a .hshader."
                .into()
        } else if self.page == Page::Mods
            && self.config.mod_source == crate::shader_catalog::Origin::Modrinth
        {
            format!(
                "{} Java mods - browsing only; this Rust client cannot run Java loaders",
                items.len()
            )
        } else if self.page == Page::Mods {
            let native = items.iter().filter(|p| source_only_mod(p)).count();
            format!(
                "{} .hmod / {} source-only .hnmod (not supported yet)",
                items.len() - native,
                native
            )
        } else {
            format!(
                "{} results - select Install to check compatibility",
                items.len()
            )
        };
        self.results = items;
    }
    fn start_search(&mut self) {
        if self.busy() || self.search_worker.is_some() {
            return;
        }
        let Some((page, source, text, offset)) = self.searches.take_next() else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        self.search_worker = Some(rx);
        std::thread::spawn(move || {
            let kind = match page {
                Page::Mods => Kind::Mod,
                Page::Shaders => Kind::Shader,
                _ => Kind::ResourcePack,
            };
            let mut query = Query::new(&text, kind);
            query.offset = offset;
            query.limit = 12;
            let result = if page == Page::Shaders && source == browser::SourceId::Modrinth {
                query.everything = true;
                browser::modrinth::search_shader_packs(&query)
            } else if page == Page::Packs
                || (page == Page::Mods && source == browser::SourceId::Modrinth)
            {
                query.everything = true;
                browser::modrinth::Modrinth.search(&query)
            } else {
                browser::horizon::Registry.search(&query)
            };
            let _ = tx.send(result);
        });
    }
    pub fn import_native(&mut self, home: &Path) {
        let url = self.query.trim().to_owned();
        let game = self.config.game_home(home);
        self.job("Downloading sandboxed .hmod over HTTPS...",move||{
            use std::io::Read;
            let mut bytes=Vec::new();
            ureq::AgentBuilder::new().https_only(true).timeout_connect(std::time::Duration::from_secs(20)).timeout_read(std::time::Duration::from_secs(90)).build().get(&url).call().map_err(|e|e.to_string())?.into_reader().take(32*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
            if bytes.len()>32*1024*1024 {return Err("Horizon mod exceeds 32 MiB".into());}
            let dir=game.join("horizon-mods");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
            let stage=dir.join("import.partial");std::fs::write(&stage,bytes).map_err(|e|e.to_string())?;
            let result=(||{
                let package=horizon_mods::Package::read(&stage)?;
                let destination=dir.join(format!("{}.hmod",package.manifest.id));
                if destination.exists(){std::fs::copy(&destination,destination.with_extension("hmod.previous")).map_err(|e|e.to_string())?;}
                std::fs::rename(&stage,&destination).map_err(|e|e.to_string())?;
                let mut on = horizon_client::mod_list::ModList::load(&game);
                on.enabled.insert(package.manifest.id.clone());
                let _ = on.save(&game);
                let _ = crate::browser::java::set_enabled(&game, &package.manifest.id, true);
                Ok(Reply::Done(format!("Installed {}. Enable it in Installed after reviewing its capabilities in game Mods.",package.manifest.name), horizon_client::content_files::Kind::Mods))
            })();
            let _=std::fs::remove_file(stage);result
        });
    }
    pub fn install(&mut self, home: &Path, index: usize) {
        let Some(project) = self.results.get(index).cloned() else {
            return;
        };
        if source_only_mod(&project) {
            self.status = "This .hnmod needs a separately compiled client. Android support is planned; sandboxed .hmod mods can be installed now.".into();
            return;
        }
        if self.busy() || self.download_label(&project).is_some() {
            return;
        }
        let game = self.config.game_home(home);
        if project.kind != Kind::ResourcePack
            && !(project.kind == Kind::Shader
                && matches!(
                    project.source,
                    browser::SourceId::Horizon | browser::SourceId::Modrinth
                ))
            && !(cfg!(target_pointer_width = "64")
                && project.kind == Kind::Mod
                && matches!(
                    project.source,
                    browser::SourceId::Horizon | browser::SourceId::Modrinth
                ))
        {
            return;
        }
        self.downloading = Some((project.source, project.id.clone()));
        self.failed = None;
        *self.progress.lock().unwrap_or_else(|e| e.into_inner()) = 0.0;
        let progress = self.progress.clone();
        let status =
            if project.kind == Kind::Shader && project.source == browser::SourceId::Modrinth {
                "Downloading experimental shader ZIP..."
            } else {
                "Checking compatibility and downloading..."
            };
        self.job(status, move || {
            std::fs::create_dir_all(&game).map_err(|e| e.to_string())?;
            let source = browser::source(project.source).ok_or("Unsupported download source")?;
            let count = if project.kind == Kind::Shader
                && project.source == browser::SourceId::Modrinth
            {
                crate::shader_catalog::install(source.as_ref(), &project, &game, &|_name, p| {
                    if p.is_finite() {
                        *progress.lock().unwrap_or_else(|e| e.into_inner()) = p.clamp(0.0, 1.0);
                    }
                })?
            } else {
                let entries = browser::installed::install(source.as_ref(), &project, true, &game, &|_name, p| {
                    if p.is_finite() {
                        *progress.lock().unwrap_or_else(|e| e.into_inner()) = p.clamp(0.0, 1.0);
                    }
                })?;
                if project.kind == Kind::Mod {
                    let mut on = horizon_client::mod_list::ModList::load(&game);
                    on.enabled.insert(project.id.clone());
                    for entry in &entries {
                        if let Some(id) = entry.file.strip_prefix("horizon-mods/") {
                            let id = id.strip_suffix(".hmod").unwrap_or(id).trim_end_matches('/');
                            on.enabled.insert(id.to_string());
                            let _ = crate::browser::java::set_enabled(&game, id, true);
                        }
                    }
                    let _ = on.save(&game);
                    let _ = crate::browser::java::set_enabled(&game, &project.id, true);
                }
                entries.len()
            };
            Ok(Reply::Done(
                format!("Installed {count} item(s). Open Play to load this profile."),
                match project.kind {
                    Kind::Mod => horizon_client::content_files::Kind::Mods,
                    Kind::Shader => horizon_client::content_files::Kind::Shaders,
                    _ => horizon_client::content_files::Kind::Packs,
                },
            ))
        });
    }
    pub fn toggle_local(&mut self, home: &Path, index: usize) {
        let Some(item) = self.local.get(index).cloned() else {
            return;
        };
        let game = self.config.game_home(home);
        let path = game.join("options.txt");
        let mut settings = horizon_client::settings::Settings::load(&path);
        match item.kind {
            horizon_client::content_files::Kind::Mods => {
                if horizon_client::mod_list::ANDROID_BUNDLED.iter().any(|(b_id, _, _)| *b_id == item.file) {
                    let mut list = horizon_client::mod_list::ModList::load(&game);
                    if item.enabled {
                        list.enabled.remove(&item.file);
                        list.disabled.insert(item.file.clone());
                    } else {
                        list.disabled.remove(&item.file);
                        list.enabled.insert(item.file.clone());
                    }
                    let _ = std::fs::create_dir_all(&game);
                    let _ = std::fs::write(game.join(horizon_client::mod_list::FILE), list.serialize());
                    self.status = format!(
                        "Мод {} {}.",
                        item.title,
                        if item.enabled { "отключен" } else { "включен" }
                    );
                    self.refresh(home);
                    return;
                }
                if !cfg!(target_pointer_width = "64") {
                    self.status = "Sandboxed code mods require a 64-bit APK".into();
                    return;
                }
                let mut on = horizon_client::mod_list::ModList::load(&game);
                let id = item
                    .file
                    .strip_suffix(".hmod")
                    .or_else(|| item.file.strip_suffix(".hnmod"))
                    .or_else(|| item.file.strip_suffix(".jar"))
                    .or_else(|| item.file.strip_suffix(".zip"))
                    .unwrap_or(&item.file);
                if item.enabled {
                    on.enabled.remove(id);
                } else {
                    on.enabled.insert(id.to_owned());
                }
                let _ = on.save(&game);
                let _ = crate::browser::java::set_enabled(&game, id, !item.enabled);
            }
            horizon_client::content_files::Kind::Packs => {
                let id = format!("file/{}", item.file);
                if item.enabled {
                    settings.resource_packs.retain(|p| p != &id);
                } else {
                    settings.resource_packs.push(id);
                }
                settings.save(&path);
            }
            horizon_client::content_files::Kind::Shaders => {
                settings.horizon.shader_pack = if item.enabled {
                    String::new()
                } else {
                    item.file
                };
                settings.save(&path);
            }
        }
        self.status = "Selection saved for the next game launch".into();
        self.refresh(home);
    }
    pub fn remove_local(&mut self, home: &Path) {
        let Some(item) = self.pending_delete.take() else {
            return;
        };
        let game = self.config.game_home(home);
        if horizon_client::mod_list::ANDROID_BUNDLED.iter().any(|(b_id, _, _)| *b_id == item.file) {
            let mut list = horizon_client::mod_list::ModList::load(&game);
            list.enabled.remove(&item.file);
            list.disabled.insert(item.file.clone());
            let _ = std::fs::create_dir_all(&game);
            let _ = std::fs::write(game.join(horizon_client::mod_list::FILE), list.serialize());
            self.refresh(home);
            return;
        }
        let result = (|| -> Result<(), String> {
            if !horizon_client::content_files::plain_name(&item.file) {
                return Err("Invalid content filename".into());
            }
            let dir = game.join(item.kind.folder());
            if std::fs::symlink_metadata(&dir)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err("Refusing a redirected content directory".into());
            }
            let target = dir.join(&item.file);
            let meta = std::fs::symlink_metadata(&target).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() {
                return Err(
                    "Only an explicitly selected regular add-on file can be removed".into(),
                );
            }
            if meta.is_dir() {
                std::fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
            } else {
                std::fs::remove_file(&target).map_err(|e| e.to_string())?;
            }
            let relative = format!("{}/{}", item.kind.folder(), item.file);
            let mut installed = Installed::load(&game);
            installed.entries.retain(|e| e.file != relative);
            installed.save(&game)?;
            let path = game.join("options.txt");
            let mut settings = horizon_client::settings::Settings::load(&path);
            if item.kind == horizon_client::content_files::Kind::Packs {
                settings
                    .resource_packs
                    .retain(|p| p != &format!("file/{}", item.file));
            }
            if item.kind == horizon_client::content_files::Kind::Shaders
                && settings.horizon.shader_pack == item.file
            {
                settings.horizon.shader_pack.clear();
            }
            settings.save(&path);
            if item.kind == horizon_client::content_files::Kind::Mods {
                let id = item
                    .file
                    .strip_suffix(".hmod")
                    .or_else(|| item.file.strip_suffix(".hnmod"))
                    .or_else(|| item.file.strip_suffix(".jar"))
                    .or_else(|| item.file.strip_suffix(".zip"))
                    .unwrap_or(&item.file);
                let mut on = horizon_client::mod_list::ModList::load(&game);
                on.enabled.remove(id);
                let _ = on.save(&game);
                let _ = crate::browser::java::set_enabled(&game, id, false);
                let native_target = game
                    .join(horizon_client::mod_list::NATIVE_DIR)
                    .join(&item.file);
                if native_target.is_file() {
                    let _ = std::fs::remove_file(native_target);
                }
            }
            Ok(())
        })();
        self.status = match result {
            Ok(()) => format!("Removed {}", item.title),
            Err(e) => e,
        };
        self.refresh(home);
    }
    pub fn edit(&mut self, out: horizon_client::osk::Output, home: &Path) {
        use horizon_client::osk::Output;
        let field = match self.editing {
            Some("name") => &mut self.config.name,
            Some("server") => &mut self.config.server,
            Some("render_scale") => &mut self.render_scale_text,
            Some("client_id") => &mut self.config.microsoft_client_id,
            _ => &mut self.query,
        };
        match out {
            Output::Text(s) => {
                field.push_str(&s);
                if self.editing == Some("name") {
                    let _ = self.config.save(home);
                }
            }
            Output::Backspace => {
                field.pop();
                if self.editing == Some("name") {
                    let _ = self.config.save(home);
                }
            }
            Output::Enter | Output::Close => {
                if self.editing == Some("render_scale") {
                    match crate::display::RenderScale::parse(&self.render_scale_text) {
                        Ok(scale) => {
                            self.config.render_scale = scale;
                            crate::display::set_scale(scale);
                        }
                        Err(error) => {
                            self.status = error.into();
                            return;
                        }
                    }
                }
                self.editing = None;
                let _ = self.config.save(home);
                if self.page.is_catalog() {
                    self.offset = 0;
                    self.search(home);
                }
            }
        }
    }
}

pub fn set_mod_enabled(home: &Path, id: &str, on: bool) -> Result<(), String> {
    crate::browser::java::set_enabled(home, id, on)
}
