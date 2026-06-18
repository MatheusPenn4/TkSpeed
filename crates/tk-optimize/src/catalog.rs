//! Catálogo de otimizações. Modelo genérico: cada otimização só declara `plan()`
//! (estado antigo → novo, como `ReversibleAction`); o Engine aplica/verifica/reverte
//! genericamente. `validation()` define como o efeito é provado.

use std::path::{Path, PathBuf};

use tk_contracts::{OptimizationInfo, StartupItem};
use tk_platform_win::{elevation, gpu, memory, power, registry, registry_hklm, startup};
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

const ULTIMATE_PERF_GUID: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";
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
            expected_impact: "Mantém os clocks da CPU elevados sob carga — ganho visível em jogos e cargas sustentadas.".into(),
            requires_elevation: false,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation {
        Validation::Manual
    }
    fn preview(&self) -> Preview {
        Preview { summary: "Trocar o plano de energia ativo para Alto Desempenho".into(), changes: vec!["Plano de energia ativo → Alto Desempenho".into()] }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let old = power::get_active_scheme().map_err(|e| e.to_string())?;

        // 1. Busca dinâmica por nome — funciona com planos OEM, localizados e customizados.
        //    Cobre casos como "Desempenho Máximo" (OEM Dell/HP/Lenovo) e variantes regionais.
        if let Some((guid, _name)) = power::find_high_perf_scheme() {
            if guid == old {
                return Err("Plano de alto desempenho já está ativo.".into());
            }
            return Ok(vec![ReversibleAction::PowerPlan { old_guid: old, new_guid: guid }]);
        }

        // 2. Tenta criar Ultimate Performance via duplicatescheme (disponível em Pro/Enterprise).
        if power::ensure_scheme(ULTIMATE_PERF_GUID).is_ok() {
            if ULTIMATE_PERF_GUID == old {
                return Err("Plano Ultimate Performance já está ativo.".into());
            }
            return Ok(vec![ReversibleAction::PowerPlan { old_guid: old, new_guid: ULTIMATE_PERF_GUID.into() }]);
        }

        // 3. Tenta criar Alto Desempenho via duplicatescheme (disponível na maioria das edições).
        if power::ensure_scheme(HIGH_PERF_GUID).is_ok() {
            if HIGH_PERF_GUID == old {
                return Err("Plano Alto Desempenho já está ativo.".into());
            }
            return Ok(vec![ReversibleAction::PowerPlan { old_guid: old, new_guid: HIGH_PERF_GUID.into() }]);
        }

        Err("Windows não permitiu ativar ou criar um plano de alto desempenho nesta instalação.".into())
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
            requires_reboot: false,
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
        OptimizationInfo { id: self.id.into(), name: self.name.into(), description: self.description.into(), category: "game".into(), risk: "Moderate".into(), expected_impact: self.impact.into(), requires_elevation: false, requires_reboot: false }
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

// ───────────────────────── Efeitos visuais para gaming (V4.5-A fix) ─────────────────────────

pub struct VisualEffectsGaming;

impl Optimization for VisualEffectsGaming {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: "visual_effects_gaming".into(),
            name: "Ajustar Efeitos Visuais para Desempenho".into(),
            description: "Configura o Windows para ajustar automaticamente os efeitos visuais priorizando desempenho (VisualFXSetting=2).".into(),
            category: "game".into(),
            risk: "Safe".into(),
            expected_impact: "Reduz overhead de renderização da interface. Efeito subjetivo — comprovar manualmente.".into(),
            requires_elevation: false,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview { summary: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects\\VisualFXSetting = 2".into(), changes: vec!["Efeitos visuais → Ajustar para melhor desempenho".into()] }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let sub = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects";
        let old = registry::read_u32(sub, "VisualFXSetting").map_err(|e| e.to_string())?;
        if old == Some(2) {
            return Err("Já configurado para desempenho.".into());
        }
        Ok(vec![ReversibleAction::RegistryHkcuDword { subkey: sub.into(), name: "VisualFXSetting".into(), old, new: Some(2) }])
    }
}

// ───────────────────────── RAM / Standby Memory (V4.5-C) ─────────────────────────

pub struct MemoryStandbyClean;

impl Optimization for MemoryStandbyClean {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: "cleanup.memory_standby".into(),
            name: "Limpar Memória em Espera (Standby)".into(),
            description: "Libera a standby list da RAM, devolvendo memória cached ao pool de livre. Útil antes de iniciar um jogo pesado.".into(),
            category: "cleanup".into(),
            risk: "Safe".into(),
            expected_impact: "Pode reduzir stutters no carregamento de assets. Efeito depende de quanto está em standby — comprovar com monitor de RAM.".into(),
            requires_elevation: true,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::SpaceFreed }
    fn preview(&self) -> Preview {
        let avail_mb = memory::available_bytes() / (1024 * 1024);
        Preview { summary: format!("Liberar standby list (~{avail_mb} MB disponível agora)"), changes: vec!["NtSetSystemInformation(MemoryPurgeStandbyList)".into()] }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        if !elevation::is_elevated() {
            return Err("Requer administrador para liberar a standby list.".into());
        }
        Ok(vec![ReversibleAction::MemoryStandbyFlush])
    }
}

// ───────────────────────── Gamer Debloat (V4.5-D) ─────────────────────────

pub struct HkcuDwordOpt {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    subkey: &'static str,
    value_name: &'static str,
    target: u32,
    already_msg: &'static str,
}

impl Optimization for HkcuDwordOpt {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: "game".into(),
            risk: "Safe".into(),
            expected_impact: self.impact.into(),
            requires_elevation: false,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKCU\\{}\\{} = {}", self.subkey, self.value_name, self.target),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let old = registry::read_u32(self.subkey, self.value_name).map_err(|e| e.to_string())?;
        if old == Some(self.target) {
            return Err(self.already_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHkcuDword {
            subkey: self.subkey.into(),
            name: self.value_name.into(),
            old,
            new: Some(self.target),
        }])
    }
}

// ───────────────────────── Network Gaming Pack (V4.5-E) ─────────────────────────

pub struct HklmDwordOpt {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    subkey: &'static str,
    value_name: &'static str,
    target: u32,
    already_msg: &'static str,
}

impl Optimization for HklmDwordOpt {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: "network".into(),
            risk: "Moderate".into(),
            expected_impact: self.impact.into(),
            requires_elevation: true,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKLM\\{}\\{} = {}", self.subkey, self.value_name, self.target),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        if !elevation::is_elevated() {
            return Err("Requer administrador para alterar configurações do sistema (HKLM).".into());
        }
        let old = registry_hklm::read_u32(self.subkey, self.value_name).map_err(|e| e.to_string())?;
        if old == Some(self.target) {
            return Err(self.already_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHklmDword {
            subkey: self.subkey.into(),
            name: self.value_name.into(),
            old,
            new: Some(self.target),
        }])
    }
}

// ───────────────────────── V4.6: Structs genéricos adicionais ─────────────────────────

/// HKLM DWORD com categoria e reboot configuráveis (V4.6+).
pub struct HklmDwordOptFlex {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    category: &'static str,
    subkey: &'static str,
    value_name: &'static str,
    target: u32,
    already_msg: &'static str,
    requires_reboot: bool,
}

impl Optimization for HklmDwordOptFlex {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: self.category.into(),
            risk: "Moderate".into(),
            expected_impact: self.impact.into(),
            requires_elevation: true,
            requires_reboot: self.requires_reboot,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKLM\\{}\\{} = {}", self.subkey, self.value_name, self.target),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        if !elevation::is_elevated() {
            return Err("Requer administrador para alterar configurações do sistema (HKLM).".into());
        }
        let old = registry_hklm::read_u32(self.subkey, self.value_name).map_err(|e| e.to_string())?;
        if old == Some(self.target) {
            return Err(self.already_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHklmDword {
            subkey: self.subkey.into(),
            name: self.value_name.into(),
            old,
            new: Some(self.target),
        }])
    }
}

/// Desabilita um item de inicialização (HKCU\Run). Reversível: restaura o valor original.
pub struct HkcuRunDisable {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    run_value_name: &'static str,
    not_found_msg: &'static str,
}

impl Optimization for HkcuRunDisable {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: "services".into(),
            risk: "Safe".into(),
            expected_impact: self.impact.into(),
            requires_elevation: false,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKCU\\...\\Run\\{} → remover (app não é desinstalado)", self.run_value_name),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let sub = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        let old = registry::read_string(sub, self.run_value_name).map_err(|e| e.to_string())?;
        if old.is_none() {
            return Err(self.not_found_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHkcu {
            subkey: sub.into(),
            name: self.run_value_name.into(),
            old,
            new: None,
        }])
    }
}

/// Desativa aceleração do mouse (PointerPrecision) — 3 valores HKCU simultâneos.
pub struct MouseAccelOff;

impl Optimization for MouseAccelOff {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: "game.mouse_accel_off".into(),
            name: "Desativar Aceleração do Mouse".into(),
            description: "Remove a aceleração do ponteiro (Enhance Pointer Precision) para movimentos 1:1 precisos — essencial para jogos de mira.".into(),
            category: "game".into(),
            risk: "Safe".into(),
            expected_impact: "Movimentos do mouse tornam-se proporcionais à velocidade física. Recomendado para FPS e jogos de precisão.".into(),
            requires_elevation: false,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: "HKCU\\Control Panel\\Mouse: MouseSpeed=0, Threshold1=0, Threshold2=0".into(),
            changes: vec!["Aceleração do mouse desativada (movimentos 1:1)".into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let sub = "Control Panel\\Mouse";
        let speed = registry::read_string(sub, "MouseSpeed").map_err(|e| e.to_string())?;
        let t1    = registry::read_string(sub, "MouseThreshold1").map_err(|e| e.to_string())?;
        let t2    = registry::read_string(sub, "MouseThreshold2").map_err(|e| e.to_string())?;
        if speed.as_deref() == Some("0") && t1.as_deref() == Some("0") && t2.as_deref() == Some("0") {
            return Err("Aceleração do mouse já desativada.".into());
        }
        Ok(vec![
            ReversibleAction::RegistryHkcu { subkey: sub.into(), name: "MouseSpeed".into(),     old: speed, new: Some("0".into()) },
            ReversibleAction::RegistryHkcu { subkey: sub.into(), name: "MouseThreshold1".into(), old: t1,    new: Some("0".into()) },
            ReversibleAction::RegistryHkcu { subkey: sub.into(), name: "MouseThreshold2".into(), old: t2,    new: Some("0".into()) },
        ])
    }
}

// ───────────────────────── V4.6.1: HKCU com detecção de vendor de GPU ─────────────────────────

/// HKCU DWORD aplicado somente se o vendor de GPU exigido estiver presente.
pub struct HkcuVendorDwordOpt {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    category: &'static str,
    required_vendor: &'static str,
    subkey: &'static str,
    value_name: &'static str,
    target: u32,
    already_msg: &'static str,
}

impl Optimization for HkcuVendorDwordOpt {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: self.category.into(),
            risk: "Safe".into(),
            expected_impact: self.impact.into(),
            requires_elevation: false,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKCU\\{}\\{} = {} (GPU: {})", self.subkey, self.value_name, self.target, self.required_vendor),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let gpus = gpu::detect();
        if !gpus.iter().any(|g| g.vendor == self.required_vendor) {
            return Err(format!("GPU {} não detectada neste sistema.", self.required_vendor));
        }
        let old = registry::read_u32(self.subkey, self.value_name).map_err(|e| e.to_string())?;
        if old == Some(self.target) {
            return Err(self.already_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHkcuDword {
            subkey: self.subkey.into(),
            name: self.value_name.into(),
            old,
            new: Some(self.target),
        }])
    }
}

/// HKLM DWORD aplicado somente se o vendor de GPU exigido estiver presente. Requer admin.
pub struct GpuVendorHklmOpt {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    category: &'static str,
    required_vendor: &'static str,
    subkey: &'static str,
    value_name: &'static str,
    target: u32,
    already_msg: &'static str,
    requires_reboot: bool,
}

impl Optimization for GpuVendorHklmOpt {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: self.category.into(),
            risk: "Moderate".into(),
            expected_impact: self.impact.into(),
            requires_elevation: true,
            requires_reboot: self.requires_reboot,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKLM\\{}\\{} = {} (GPU: {})", self.subkey, self.value_name, self.target, self.required_vendor),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let gpus = gpu::detect();
        if !gpus.iter().any(|g| g.vendor == self.required_vendor) {
            return Err(format!("GPU {} não detectada neste sistema.", self.required_vendor));
        }
        if !elevation::is_elevated() {
            return Err("Requer administrador para modificar configurações do driver.".into());
        }
        let old = registry_hklm::read_u32(self.subkey, self.value_name).map_err(|e| e.to_string())?;
        if old == Some(self.target) {
            return Err(self.already_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHklmDword {
            subkey: self.subkey.into(),
            name: self.value_name.into(),
            old,
            new: Some(self.target),
        }])
    }
}

/// Desativa ULPS (Ultra Low Power State) por adaptador AMD na chave de classe de dispositivo.
pub struct AmdUlpsDisable;

impl Optimization for AmdUlpsDisable {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: "amd.ulps_disable".into(),
            name: "AMD: Desativar ULPS (Ultra Low Power State)".into(),
            description: "Desativa o Ultra Low Power State do driver AMD. ULPS reduz os clocks da GPU a quase zero em idle — ao voltar a alta carga, o transitório pode causar micro-travadas e stutters de 100–500ms.".into(),
            category: "amd".into(),
            risk: "Moderate".into(),
            expected_impact: "Elimina stutters causados pelo driver AMD ao acordar do estado de ultra baixo consumo. Aumenta consumo de energia em idle. Especialmente eficaz em setups multi-GPU ou multi-monitor.".into(),
            requires_elevation: true,
            requires_reboot: true,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: "HKLM\\...\\{AMD adapter}\\EnableUlps = 0".into(),
            changes: vec!["ULPS desativado para cada adaptador AMD detectado".into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        let gpus = gpu::detect();
        if !gpus.iter().any(|g| g.vendor == "AMD") {
            return Err("GPU AMD não detectada neste sistema.".into());
        }
        if !elevation::is_elevated() {
            return Err("Requer administrador para modificar configurações do driver AMD.".into());
        }
        const DISPLAY_CLASS: &str = "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";
        let mut actions: Vec<ReversibleAction> = Vec::new();
        for i in 0..16u32 {
            let sub = format!("{DISPLAY_CLASS}\\{i:04}");
            let is_amd = match registry_hklm::read_string(&sub, "DriverDesc") {
                Ok(Some(s)) => s.contains("AMD") || s.contains("Radeon") || s.contains("Advanced Micro"),
                _ => false,
            };
            if !is_amd { continue; }
            let old = match registry_hklm::read_u32(&sub, "EnableUlps") {
                Ok(v) => v,
                Err(_) => continue,
            };
            if old == Some(0) { continue; }
            actions.push(ReversibleAction::RegistryHklmDword {
                subkey: sub,
                name: "EnableUlps".into(),
                old,
                new: Some(0),
            });
        }
        if actions.is_empty() {
            return Err("ULPS já desativado em todos os adaptadores AMD encontrados.".into());
        }
        Ok(actions)
    }
}

// ───────────────────────── V4.6.1-C: Network Gaming Pack extras ──────────

/// HKLM String reversível. Requer admin. Usa rollback RegistryHklmString.
pub struct HklmStringOpt {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    impact: &'static str,
    category: &'static str,
    subkey: &'static str,
    value_name: &'static str,
    target: &'static str,
    already_msg: &'static str,
    requires_reboot: bool,
}

impl Optimization for HklmStringOpt {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: self.id.into(),
            name: self.name.into(),
            description: self.description.into(),
            category: self.category.into(),
            risk: "Moderate".into(),
            expected_impact: self.impact.into(),
            requires_elevation: true,
            requires_reboot: self.requires_reboot,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: format!("HKLM\\{}\\{} = \"{}\"", self.subkey, self.value_name, self.target),
            changes: vec![self.impact.into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        if !elevation::is_elevated() {
            return Err("Requer administrador.".into());
        }
        let old = registry_hklm::read_string(self.subkey, self.value_name).map_err(|e| e.to_string())?;
        if old.as_deref() == Some(self.target) {
            return Err(self.already_msg.into());
        }
        Ok(vec![ReversibleAction::RegistryHklmString {
            subkey: self.subkey.into(),
            name: self.value_name.into(),
            old,
            new: Some(self.target.into()),
        }])
    }
}

/// Aplica TcpAckFrequency = 1 em todas as interfaces TCP/IP detectadas no registro.
/// Reduz a latência de ACK de rede — cada pacote recebe ACK imediato em vez de aguardar lote.
pub struct NetworkTcpAckFrequency;

impl Optimization for NetworkTcpAckFrequency {
    fn meta(&self) -> OptimizationInfo {
        OptimizationInfo {
            id: "network.tcp_ack_frequency".into(),
            name: "TCP ACK Imediato por Interface (TcpAckFrequency)".into(),
            description: "Define TcpAckFrequency=1 para cada interface de rede — o Windows envia o ACK de cada pacote imediatamente, sem aguardar o timer de 200ms do algoritmo de Nagle. Atua por interface, complementando o TcpNoDelay global.".into(),
            category: "network".into(),
            risk: "Moderate".into(),
            expected_impact: "Pode reduzir latência de rede em jogos online com pacotes pequenos e frequentes. Pode aumentar o número de pacotes enviados — ideal em conexões de baixa latência (fibra/cabo). Em redes de alta latência ou congestionadas pode aumentar o overhead.".into(),
            requires_elevation: true,
            requires_reboot: false,
        }
    }
    fn validation(&self) -> Validation { Validation::Manual }
    fn preview(&self) -> Preview {
        Preview {
            summary: "HKLM\\...\\Tcpip\\Parameters\\Interfaces\\{GUID}\\TcpAckFrequency = 1".into(),
            changes: vec!["Aplicado a cada interface detectada".into()],
        }
    }
    fn plan(&self) -> Result<Vec<ReversibleAction>, String> {
        if !elevation::is_elevated() {
            return Err("Requer administrador para modificar parâmetros TCP por interface.".into());
        }
        const IFACES: &str = "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces";
        let guids = registry_hklm::enumerate_subkeys(IFACES).map_err(|e| e.to_string())?;
        if guids.is_empty() {
            return Err("Nenhuma interface TCP detectada no registro.".into());
        }
        let mut actions: Vec<ReversibleAction> = Vec::new();
        for guid in &guids {
            let sub = format!("{IFACES}\\{guid}");
            // Só aplica a interfaces com IP configurado (físicas ou DHCP)
            let has_ip = registry_hklm::read_string(&sub, "DhcpIPAddress")
                .ok().flatten()
                .or_else(|| registry_hklm::read_string(&sub, "IPAddress").ok().flatten())
                .map(|ip| !ip.is_empty() && ip != "0.0.0.0")
                .unwrap_or(false);
            if !has_ip { continue; }
            let old = registry_hklm::read_u32(&sub, "TcpAckFrequency").map_err(|e| e.to_string())?;
            if old == Some(1) { continue; }
            actions.push(ReversibleAction::RegistryHklmDword {
                subkey: sub,
                name: "TcpAckFrequency".into(),
                old,
                new: Some(1),
            });
        }
        if actions.is_empty() {
            return Err("TcpAckFrequency já configurado em todas as interfaces ativas.".into());
        }
        Ok(actions)
    }
}

// ───────────────────────── Registro do catálogo ─────────────────────────

pub fn catalog() -> Vec<Box<dyn Optimization>> {
    vec![
        // Existentes
        Box::new(PowerPlanHigh),
        Box::new(FileCleanup::temp_files()),
        Box::new(FileCleanup::old_logs()),
        Box::new(FileCleanup::wu_cache()),
        Box::new(RegistryToggle::game_mode()),
        Box::new(RegistryToggle::xbox_game_bar()),
        // V4.5-A: bridge para IDs do ConfigRegistry
        Box::new(VisualEffectsGaming),
        // V4.5-C: RAM cleaner
        Box::new(MemoryStandbyClean),
        // V4.5-D: Gamer Debloat (HKCU, reversível sem admin)
        Box::new(HkcuDwordOpt {
            id: "game.widgets_disable",
            name: "Desativar Widgets da Barra de Tarefas",
            description: "Remove o botão de Widgets (notícias/tempo) da barra de tarefas.",
            impact: "Elimina processo de widgets em segundo plano. Comprovar com monitor de CPU.",
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
            value_name: "TaskbarDa",
            target: 0,
            already_msg: "Widgets já desativados.",
        }),
        Box::new(HkcuDwordOpt {
            id: "game.copilot_disable",
            name: "Desativar Botão do Copilot",
            description: "Remove o botão do Copilot da barra de tarefas.",
            impact: "Remove entrada visual e processo de fundo do Copilot.",
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
            value_name: "ShowCopilotButton",
            target: 0,
            already_msg: "Botão do Copilot já desativado.",
        }),
        Box::new(HkcuDwordOpt {
            id: "game.game_dvr_fse",
            name: "Game DVR: Modo FSE",
            description: "Configura o Game DVR para não interferir em modo full-screen exclusivo.",
            impact: "Reduz overhead de captura em full-screen exclusivo. Comprovar com FPS.",
            subkey: "System\\GameConfigStore",
            value_name: "GameDVR_FSEBehaviorMode",
            target: 2,
            already_msg: "Game DVR FSE já configurado.",
        }),
        // V4.5-E: Network Gaming Pack (HKLM, requer admin)
        Box::new(HklmDwordOpt {
            id: "network.throttling_disable",
            name: "Desativar Network Throttling",
            description: "Define NetworkThrottlingIndex=0xFFFFFFFF para remover limite de largura de banda do subsistema multimídia.",
            impact: "Pode reduzir latência de rede em jogos online. Comprovar com ping estável — não garante melhora em todas as redes.",
            subkey: "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
            value_name: "NetworkThrottlingIndex",
            target: 0xFFFFFFFF,
            already_msg: "Network throttling já desativado.",
        }),
        Box::new(HklmDwordOpt {
            id: "network.system_responsiveness",
            name: "Responsividade do Sistema (Gaming)",
            description: "Define SystemResponsiveness=0 para priorizar tarefas de multimídia/jogos sobre serviços de background.",
            impact: "Reserva mais CPU para o jogo em detrimento de tarefas de background. Comprovar com FPS e latência.",
            subkey: "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
            value_name: "SystemResponsiveness",
            target: 0,
            already_msg: "SystemResponsiveness já configurado para jogos.",
        }),
        // ── V4.6-G: Windows Gaming Pack ──────────────────────────────────────────
        Box::new(HkcuDwordOpt {
            id: "game.background_recording_off",
            name: "Desativar Gravação em Segundo Plano",
            description: "Desativa o AppCapture do Windows — impede captura automática de gameplay em segundo plano pelo Game DVR.",
            impact: "Libera CPU/GPU de captura de vídeo. Mais recursos para o jogo sem gravar nada.",
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR",
            value_name: "AppCaptureEnabled",
            target: 0,
            already_msg: "Gravação em segundo plano já desativada.",
        }),
        Box::new(HkcuDwordOpt {
            id: "game.game_bar_overlay_off",
            name: "Desativar Overlay do Game Bar",
            description: "Desativa a interface overlay do Xbox Game Bar durante o jogo. O app permanece instalado e pode ser reativado.",
            impact: "Elimina sobreposição da interface durante gameplay e reduz uso de memória pelo Game Bar.",
            subkey: "Software\\Microsoft\\GameBar",
            value_name: "UseNexusForGameBarEnabled",
            target: 0,
            already_msg: "Overlay do Game Bar já desativado.",
        }),
        Box::new(MouseAccelOff),
        Box::new(HklmDwordOptFlex {
            id: "game.hags_enable",
            name: "Hardware Accelerated GPU Scheduling (HAGS)",
            description: "Permite que a GPU gerencie sua própria memória de vídeo diretamente, reduzindo a latência de frames no pipeline de renderização.",
            impact: "Pode reduzir input lag e micro-stutters em GPUs compatíveis. Requer reinicialização para ativar. Comprovar com captura de FPS após reiniciar.",
            category: "game",
            subkey: "SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers",
            value_name: "HwSchMode",
            target: 2,
            already_msg: "Hardware Accelerated GPU Scheduling já ativado.",
            requires_reboot: true,
        }),
        // ── V4.6-H: Background Services Pack ─────────────────────────────────────
        Box::new(HkcuRunDisable {
            id: "services.onedrive_startup_disable",
            name: "Desativar OneDrive na Inicialização",
            description: "Remove o OneDrive da inicialização automática do Windows. O aplicativo não é desinstalado e pode ser aberto manualmente.",
            impact: "Elimina o processo do OneDrive no boot — reduz uso de RAM e tempo de inicialização.",
            run_value_name: "OneDrive",
            not_found_msg: "OneDrive não encontrado na inicialização. Pode já estar desativado ou usar outra versão.",
        }),
        Box::new(HkcuRunDisable {
            id: "services.teams_startup_disable",
            name: "Desativar Microsoft Teams na Inicialização",
            description: "Remove o Microsoft Teams da inicialização automática do Windows. O app não é desinstalado.",
            impact: "Elimina o processo do Teams no boot — reduz uso de RAM e tempo de inicialização.",
            run_value_name: "com.squirrel.Teams.Teams",
            not_found_msg: "Microsoft Teams não encontrado na inicialização. Pode já estar desativado ou usar versão diferente (Teams 2.0 não usa esta chave).",
        }),
        Box::new(HklmDwordOptFlex {
            id: "services.delivery_opt_lan",
            name: "Delivery Optimization: Somente Rede Local",
            description: "Configura o Delivery Optimization (Windows Update P2P) para compartilhar atualizações apenas com PCs da rede local, não com a internet.",
            impact: "Elimina upload de atualizações para desconhecidos na internet. Mantém o benefício de cache local.",
            category: "services",
            subkey: "SOFTWARE\\Policies\\Microsoft\\Windows\\DeliveryOptimization",
            value_name: "DODownloadMode",
            target: 1,
            already_msg: "Delivery Optimization já configurado para rede local.",
            requires_reboot: false,
        }),
        // ── V4.6-F: Network Gaming Pack Pro ──────────────────────────────────────
        Box::new(HklmDwordOptFlex {
            id: "network.tcp_nodelay",
            name: "TCP NoDelay (Desativar Nagle)",
            description: "Desativa o algoritmo de Nagle para conexões TCP — elimina o delay de agrupamento de pacotes pequenos nas conexões de jogos.",
            impact: "Reduz latência de pacotes de input em jogos online sensíveis. Pode aumentar número de pacotes enviados — ideal em redes de baixa latência.",
            category: "network",
            subkey: "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters",
            value_name: "TcpNoDelay",
            target: 1,
            already_msg: "TCP NoDelay já ativado.",
            requires_reboot: false,
        }),
        // ── V4.6.1-A: Memory & Stutter Pack ──────────────────────────────────────
        Box::new(HklmDwordOptFlex {
            id: "memory.sysmain_disable",
            name: "Desativar SysMain (Superfetch)",
            description: "Desativa o serviço SysMain (Superfetch) que pré-carrega apps na RAM em antecipação ao uso. Em PCs com SSD NVMe e bastante RAM, o overhead de gerenciamento supera o benefício.",
            impact: "Libera RAM de buffer de prefetch. Mais eficaz em sistemas com 8–16 GB de RAM e SSD. Em HDDs pode piorar carregamento de apps — use somente com SSD.",
            category: "memory",
            subkey: "SYSTEM\\CurrentControlSet\\Services\\SysMain",
            value_name: "Start",
            target: 4,
            already_msg: "SysMain já desativado.",
            requires_reboot: true,
        }),
        Box::new(HklmDwordOptFlex {
            id: "memory.prefetch_disable",
            name: "Desativar Prefetch do Windows",
            description: "Define EnablePrefetcher=0 para desativar o pré-carregamento de apps no boot. Com SSDs NVMe modernos o Prefetch adiciona escritas desnecessárias sem ganho mensurável.",
            impact: "Reduz escritas no SSD durante boot. Pode aumentar IOPS disponíveis. Não recomendado em HDDs — o Prefetch é essencial para performance em discos mecânicos.",
            category: "memory",
            subkey: "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters",
            value_name: "EnablePrefetcher",
            target: 0,
            already_msg: "Prefetch já desativado.",
            requires_reboot: false,
        }),
        Box::new(HklmDwordOptFlex {
            id: "memory.pagefile_optimize",
            name: "Não Limpar Pagefile no Desligamento",
            description: "Garante que ClearPageFileAtShutdown=0 — impede que o Windows apague o arquivo de paginação a cada desligamento. A limpeza pode adicionar 30–60s ao desligamento sem benefício prático de segurança em uso doméstico.",
            impact: "Desligamento mais rápido (pode economizar 30–60 segundos). Sem impacto em performance durante uso.",
            category: "memory",
            subkey: "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
            value_name: "ClearPageFileAtShutdown",
            target: 0,
            already_msg: "Pagefile já configurado (ClearPageFileAtShutdown=0).",
            requires_reboot: false,
        }),
        Box::new(HklmDwordOptFlex {
            id: "memory.large_system_cache_off",
            name: "Desativar Large System Cache",
            description: "Define LargeSystemCache=0 para que o Windows priorize memória para processos de usuário em vez de cache do sistema de arquivos. Padrão para estações de trabalho; o modo '1' é otimizado para servidores de arquivos.",
            impact: "Garante que a RAM seja alocada prioritariamente para o jogo/app em execução, não para cache de disco. Mais efetivo em sistemas com menos de 16 GB de RAM.",
            category: "memory",
            subkey: "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
            value_name: "LargeSystemCache",
            target: 0,
            already_msg: "Large System Cache já desativado (modo workstation).",
            requires_reboot: true,
        }),
        // ── V4.6.1-B: Windows Gaming Pack extras ─────────────────────────────────
        Box::new(HklmDwordOptFlex {
            id: "game.mpo_disable",
            name: "Desativar MPO (Multiplane Overlay)",
            description: "Define OverlayTestMode=5 para desativar o Multiplane Overlay do DWM. MPO é a principal causa de flickering de tela preta e micro-stutters em GPUs NVIDIA com múltiplos monitores.",
            impact: "Elimina o flickering/stutter do MPO em GPUs NVIDIA (GTX 900+ e RTX). Alta eficácia para quem usa múltiplos monitores ou GSYNC/FreeSync. Requer reinicialização.",
            category: "game",
            subkey: "SOFTWARE\\Microsoft\\Windows\\Dwm",
            value_name: "OverlayTestMode",
            target: 5,
            already_msg: "MPO já desativado (OverlayTestMode=5).",
            requires_reboot: true,
        }),
        Box::new(HklmDwordOptFlex {
            id: "game.tasks_games_priority",
            name: "Prioridade MMCSS para Jogos (Priority=6)",
            description: "Define Priority=6 no perfil de tarefas Games do MMCSS (Multimedia Class Scheduler Service) — garante que threads de jogos recebam preferência de agendamento de CPU sobre tarefas de background.",
            impact: "Pode reduzir frame time variance (frametimes irregulares) em jogos com muitas threads. Complementa SystemResponsiveness=0.",
            category: "game",
            subkey: "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile\\Tasks\\Games",
            value_name: "Priority",
            target: 6,
            already_msg: "Prioridade de tarefas de jogos já configurada.",
            requires_reboot: false,
        }),
        // ── V4.6.1-D: NVIDIA Pack ─────────────────────────────────────────────────
        Box::new(HkcuVendorDwordOpt {
            id: "nvidia.ansel_disable",
            name: "NVIDIA: Desativar Ansel (Screenshot AI)",
            description: "Desativa o NVIDIA Ansel — ferramenta de screenshot de IA que injeta hooks nos jogos. Pode causar micro-stutters ao detectar jogos compatíveis.",
            impact: "Elimina o overhead de injeção do Ansel em jogos. Sem perda de funcionalidade para quem não usa screenshots 360°.",
            category: "nvidia",
            required_vendor: "NVIDIA",
            subkey: "Software\\NVIDIA Corporation\\Global\\Ansel",
            value_name: "Enable",
            target: 0,
            already_msg: "NVIDIA Ansel já desativado.",
        }),
        Box::new(HkcuVendorDwordOpt {
            id: "nvidia.overlay_disable",
            name: "NVIDIA: Desativar GeForce Overlay (In-Game)",
            description: "Desativa o overlay in-game da GeForce Experience. O overlay injeta código nos processos de jogo para exibir métricas e permitir gravação — fonte conhecida de stutters e crashes.",
            impact: "Elimina a injeção do overlay NVIDIA nos jogos. Pode resolver crashes inexplicáveis em jogos anti-cheat. Não afeta a GeForce Experience fora dos jogos.",
            category: "nvidia",
            required_vendor: "NVIDIA",
            subkey: "Software\\NVIDIA Corporation\\Global\\GFExperience",
            value_name: "EnableGameOverlay",
            target: 0,
            already_msg: "Overlay NVIDIA in-game já desativado.",
        }),
        // ── V4.6.1-E: AMD Pack ────────────────────────────────────────────────────
        Box::new(AmdUlpsDisable),
        // ── V4.6.1-C: Network Gaming Pack extras ─────────────────────────────────
        Box::new(HklmDwordOptFlex {
            id: "network.dns_negative_cache_off",
            name: "DNS: Desativar Cache de Falhas (MaxNegativeCacheTtl=0)",
            description: "Define MaxNegativeCacheTtl=0 no Dnscache — o Windows para de armazenar respostas 'domínio não encontrado' em cache. Útil quando servidores de jogo usam DNS dinâmico que muda rapidamente.",
            impact: "Evita que erros de resolução DNS sejam mantidos em cache por até 300s (padrão). Pode reduzir o tempo de reconexão quando o IP do servidor muda. Pode aumentar levemente a carga no DNS resolver em caso de domínios inexistentes.",
            category: "network",
            subkey: "SYSTEM\\CurrentControlSet\\Services\\Dnscache\\Parameters",
            value_name: "MaxNegativeCacheTtl",
            target: 0,
            already_msg: "MaxNegativeCacheTtl já desativado.",
            requires_reboot: false,
        }),
        Box::new(HklmDwordOptFlex {
            id: "network.lanman_bandwidth_throttle_off",
            name: "LanMan: Desativar Throttling de Largura de Banda",
            description: "Define DisableBandwidthThrottling=1 no LanmanWorkstation — remove o limite de 512 KB/s que o Windows aplica a transferências de rede em segundo plano via SMB.",
            impact: "Pode melhorar throughput em redes locais (NAS, compartilhamentos) e reduzir interferência de transferências SMB em background enquanto joga. Sem efeito em conexões à internet direta.",
            category: "network",
            subkey: "SYSTEM\\CurrentControlSet\\Services\\LanmanWorkstation\\Parameters",
            value_name: "DisableBandwidthThrottling",
            target: 1,
            already_msg: "Throttling de largura de banda LanMan já desativado.",
            requires_reboot: false,
        }),
        Box::new(HklmStringOpt {
            id: "network.mmcss_games_scheduling_high",
            name: "MMCSS: Categoria de Agendamento de Jogos (High)",
            description: "Define Scheduling Category=High no perfil de tarefas Games do MMCSS — reforça a prioridade de agendamento de CPU para threads de jogos no Multimedia Class Scheduler.",
            impact: "Complementa SystemResponsiveness=0 e Priority=6. Pode reduzir variação de frame time em jogos com muitas threads de renderização. Ganho marginal quando outros ajustes de MMCSS já estão ativos.",
            category: "network",
            subkey: "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile\\Tasks\\Games",
            value_name: "Scheduling Category",
            target: "High",
            already_msg: "MMCSS Scheduling Category para Games já está em High.",
            requires_reboot: false,
        }),
        Box::new(NetworkTcpAckFrequency),
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
