//! Catálogo de otimizações. Modelo genérico: cada otimização só declara `plan()`
//! (estado antigo → novo, como `ReversibleAction`); o Engine aplica/verifica/reverte
//! genericamente. `validation()` define como o efeito é provado.

use std::path::{Path, PathBuf};

use tk_contracts::{OptimizationInfo, StartupItem};
use tk_platform_win::{power, registry, startup};
use tk_rollback::ReversibleAction;

/// Como o efeito de uma otimização é validado.
pub enum Validation {
    /// Benchmark automático (suite) antes/depois → Confidence Engine decide.
    Benchmark(&'static str),
    /// Sem benchmark: prova é o espaço liberado (limpezas).
    SpaceFreed,
    /// Só comprovável manualmente (ex.: FPS em jogo). Aplica e fica pendente de evidência.
    Manual,
}

pub struct Preview {
    pub summary: String,
    pub changes: Vec<String>,
}

pub trait Optimization: Send + Sync {
    fn meta(&self) -> OptimizationInfo;
    fn validation(&self) -> Validation;
    fn preview(&self) -> Preview;
    /// Lê o estado atual e define o novo (base do snapshot + da aplicação).
    fn plan(&self) -> Result<Vec<ReversibleAction>, String>;
}

const HIGH_PERF_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
const MAX_CLEANUP_FILES: usize = 4000;

// ───────────────────────── Power Plan (OE-1) ─────────────────────────

pub struct PowerPlanHigh;

impl Optimization for PowerPlanHigh {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: "energy.power_plan_high".into(),
            name: "Plano de Energia: Alto Desempenho".into(),
            description: "Ativa o plano de Alto Desempenho do Windows (mantém os clocks da CPU elevados sob carga).".into(),
            category: "energy".into(),
            risk: "Moderate".into(),
            expected_impact: "Pode melhorar o desempenho sustentado de CPU (comprovado por benchmark).".into(),
            requires_elevation: false,
        }
    }
    fn validation(&self) -> Validation {
        Validation::Benchmark("cpu-1.0.0")
    }
    fn preview(&self) -> Preview {
        Preview { summary: "Trocar o plano de energia ativo para Alto Desempenho".into(), changes: vec!["Plano de energia ativo → Alto Desempenho".into()] }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let old = power::get_active_scheme().map_err(|e| e.to_string())?;
        Ok(vec![ReversibleAction::PowerPlan { old_guid: old, new_guid: HIGH_PERF_GUID.into() }])
    }
}

// ───────────────────────── Limpezas (genérico, SpaceFreed) ─────────────────────────

pub struct FileCleanup {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    dirs: Vec<PathBuf>,
    min_age_days: u64,
    exts: Vec<&'static str>,
    requires_elevation: bool,
    depth: usize,
}

impl FileCleanup {
    pub fn temp_files() -> Self {
        Self { id: "cleanup.temp_files", name: "Limpeza de Temporários", description: "Move arquivos temporários antigos para a quarentena (recuperáveis).", dirs: temp_dirs(), min_age_days: 1, exts: vec![], requires_elevation: false, depth: 2 }
    }
    pub fn old_logs() -> Self {
        Self { id: "cleanup.old_logs", name: "Limpeza de Logs Antigos", description: "Move logs antigos (.log/.etl/.dmp) para a quarentena.", dirs: temp_dirs(), min_age_days: 7, exts: vec!["log", "etl", "dmp"], requires_elevation: false, depth: 2 }
    }
    pub fn wu_cache() -> Self {
        Self { id: "cleanup.wu_cache", name: "Cache do Windows Update", description: "Move arquivos baixados do Windows Update para a quarentena (best-effort; pode exigir admin).", dirs: vec![PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download")], min_age_days: 0, exts: vec![], requires_elevation: true, depth: 1 }
    }
}

impl Optimization for FileCleanup {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: "cleanup".into(),
            risk: if self.requires_elevation { "Moderate" } else { "Safe" }.into(),
            expected_impact: "Libera espaço em disco. NÃO aumenta FPS.".into(),
            requires_elevation: self.requires_elevation,
        }
    }
    fn validation(&self) -> Validation {
        Validation::SpaceFreed
    }
    fn preview(&self) -> Preview {
        Preview { summary: "Mover arquivos para a quarentena (recuperável por TTL)".into(), changes: self.dirs.iter().map(|d| format!("Varredura: {}", d.display())).collect() }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let qbase = quarantine_base();
        let mut actions: Vec<ReversibleAction> = Vec::new();
        let mut idx = 0usize;
        for dir in &self.dirs {
            scan(dir, self.depth, self.min_age_days, &self.exts, &mut |path, size| {
                if actions.len() >= MAX_CLEANUP_FILES {
                    return;
                }
                let fname = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| "f".into());
                let q = qbase.join(format!("{idx}_{fname}"));
                actions.push(ReversibleAction::FileQuarantine {
                    original: path.to_string_lossy().to_string(),
                    quarantine: q.to_string_lossy().to_string(),
                    size,
                });
                idx += 1;
            });
        }
        Ok(actions)
    }
}

// ───────────────────────── Toggles de registro (DWORD, HKCU, Manual) ─────────────────────────

pub struct RegistryToggle {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    subkey: &'static str,
    value: &'static str,
    on: u32,
}

impl RegistryToggle {
    pub fn game_mode() -> Self {
        Self { id: "game.game_mode", name: "Game Mode", description: "Ativa o Modo Jogo do Windows.", impact: "Prioriza recursos do sistema para o jogo (comprovar com captura de FPS).", subkey: "Software\\Microsoft\\GameBar", value: "AutoGameModeEnabled", on: 1 }
    }
    pub fn xbox_game_bar() -> Self {
        Self { id: "game.xbox_game_bar", name: "Desativar Game DVR (Xbox)", description: "Desativa a gravação em segundo plano do Game DVR.", impact: "Reduz overhead de captura em segundo plano (comprovar com FPS).", subkey: "System\\GameConfigStore", value: "GameDVR_Enabled", on: 0 }
    }
}

impl Optimization for RegistryToggle {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo { id: self.id.into(), name: self.name.into(), description: self.description.into(), category: "game".into(), risk: "Moderate".into(), expected_impact: self.impact.into(), requires_elevation: false }
    }
    fn validation(&self) -> Validation {
        Validation::Manual
    }
    fn preview(&self) -> Preview {
        Preview { summary: format!("HKCU\\{}\\{} = {}", self.subkey, self.value, self.on), changes: vec![self.impact.into()] }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let old = registry::read_u32(self.subkey, self.value).map_err(|e| e.to_string())?;
        Ok(vec![ReversibleAction::RegistryHkcuDword { subkey: self.subkey.into(), name: self.value.into(), old, new: Some(self.on) }])
    }
}

// ───────────────────────── Registro do catálogo ─────────────────────────

pub fn catalog() -> Vec<Box<dyn Optimization>> {
    vec![
        Box::new(PowerPlanHigh),
        Box::new(FileCleanup::temp_files()),
        Box::new(FileCleanup::old_logs()),
        Box::new(FileCleanup::wu_cache()),
        Box::new(RegistryToggle::game_mode()),
        Box::new(RegistryToggle::xbox_game_bar()),
    ]
}

pub fn get(id: &str) -> Option<Box<dyn Optimization>> {
    catalog().into_iter().find(|o| o.meta().id == id)
}

pub fn catalog_info() -> Vec<OptimizationInfo> {
    catalog().iter().map(|o| o.meta()).collect()
}

/// Análise de inicialização (somente leitura — não é um run do pipeline).
pub fn startup_items() -> Vec<StartupItem> {
    startup::list()
        .into_iter()
        .map(|(name, command, location)| StartupItem { name, command, location })
        .collect()
}

// ───────────────────────── helpers ─────────────────────────

fn temp_dirs() -> Vec<PathBuf> {
    let mut v = vec![std::env::temp_dir()];
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        v.push(PathBuf::from(local).join("Temp"));
    }
    v
}

fn quarantine_base() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
    base.join("TkSpeed").join("quarantine").join(now_ms().to_string())
}

fn now_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

/// Varre `dir` (até `depth` níveis) chamando `cb(path, size)` para cada arquivo
/// que satisfaz extensão (se houver) e idade mínima. Erros são ignorados.
fn scan(dir: &Path, depth: usize, min_age_days: u64, exts: &[&str], cb: &mut dyn FnMut(&Path, u64)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let now = std::time::SystemTime::now();
    let min_age = std::time::Duration::from_secs(min_age_days * 86_400);

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_dir() {
            if depth > 0 {
                scan(&path, depth - 1, min_age_days, exts, cb);
            }
            continue;
        }
        if !exts.is_empty() {
            let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !exts.iter().any(|x| *x == ext) {
                continue;
            }
        }
        if min_age_days > 0 {
            match meta.modified() {
                Ok(m) => {
                    if now.duration_since(m).map(|age| age < min_age).unwrap_or(true) {
                        continue;
                    }
                }
                Err(_) => continue,
            }
        }
        cb(&path, meta.len());
    }
}
