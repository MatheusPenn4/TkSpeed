//! Contratos compartilhados entre backend (Rust) e frontend (TS).
//! Fonte única de verdade — `ts-rs` exporta os tipos `.ts` para o frontend.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ───────────────────────── Telemetria ─────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MetricSource {
    Cpu,
    Gpu,
    Ram,
    Ssd,
    Hdd,
    Net,
    Temp,
    Fps,
}

impl MetricSource {
    /// Chave estável usada na persistência (coluna `source`).
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricSource::Cpu => "cpu",
            MetricSource::Gpu => "gpu",
            MetricSource::Ram => "ram",
            MetricSource::Ssd => "ssd",
            MetricSource::Hdd => "hdd",
            MetricSource::Net => "net",
            MetricSource::Temp => "temp",
            MetricSource::Fps => "fps",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MetricSample {
    pub ts: i64,
    pub source: MetricSource,
    pub metric: String,
    pub value: f64,
    pub unit: String,
}

/// Snapshot agregado de telemetria emitido a cada tick para a UI (evento `metrics:tick`).
/// Forma "pronta para renderizar" — complementa o `MetricSample` granular (persistência).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MetricsTick {
    pub ts: i64,
    pub cpu_usage: f64,    // %
    pub ram_usage: f64,    // %
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub disk_usage: f64,   // % do volume do sistema
    pub disk_label: String,
    pub disk_is_ssd: bool, // tipo do volume do sistema (conservador: false se desconhecido)
}

/// Disco detectado no inventário de hardware.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub total_gb: f64,
    pub available_gb: f64,
    pub is_ssd: bool,
}

/// Hardware básico detectado (evento `hardware:info` / comando `get_hardware`).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HardwareInfo {
    pub hostname: String,
    pub os_name: String,
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub ram_total_gb: f64,
    pub disks: Vec<DiskInfo>,
}

// ───────────────────────── Diagnóstico ─────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Finding {
    pub kind: String,
    pub severity: Severity,
    pub title: String,
    pub impact: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScoreBreakdown {
    pub cpu: u16,
    pub gpu: u16,
    pub ram: u16,
    pub storage: u16,
    pub windows: u16,
    pub network: u16,
    pub temperature: u16,
    pub games: u16,
    pub stability: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Classification {
    Critico,
    Regular,
    Bom,
    Excelente,
    Elite,
}

impl Classification {
    pub fn as_str(&self) -> &'static str {
        match self {
            Classification::Critico => "critico",
            Classification::Regular => "regular",
            Classification::Bom => "bom",
            Classification::Excelente => "excelente",
            Classification::Elite => "elite",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TkSpeedScore {
    pub total: u16, // 0..=1000
    pub classification: Classification,
    pub breakdown: ScoreBreakdown,
    pub score_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Diagnosis {
    pub run_id: i64,
    pub findings: Vec<Finding>,
    pub score: TkSpeedScore,
}

/// Item do histórico de scores (consulta de análises passadas).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScoreHistoryItem {
    pub ts: i64,
    pub total: u16,
    pub classification: Classification,
}

// ───────────────────────── Proteção (Snapshots / Rollback) ─────────────────────────

/// Resumo de um snapshot (para listagem e UI de proteção).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SnapshotInfo {
    pub id: i64,
    pub ts: i64,
    pub reason: String,
    pub status: String, // active | restored | expired
    pub changes: u32,
    pub target: String,
}

/// Estado de proteção exibido no Dashboard (seção "Proteção").
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProtectionState {
    pub status: String,
    pub total: u32,
    pub last_snapshot: Option<SnapshotInfo>,
    pub last_rollback_ts: Option<i64>,
}

/// Resultado de um rollback.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RollbackOutcome {
    pub snapshot_id: i64,
    pub restored: u32,
    pub ok: bool,
    pub message: String,
}

/// Passo de um autoteste de proteção (snapshot→apply→verify→rollback→verify).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SelfTestStep {
    pub name: String,
    pub detail: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SelfTestReport {
    pub steps: Vec<SelfTestStep>,
    pub passed: bool,
}

// ───────────────────────── Performance Lab ─────────────────────────

/// Métrica agregada de um benchmark (média + desvio entre runs).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BenchmarkMetric {
    pub metric: String, // cpu_single | cpu_multi | cpu_load_pct ...
    pub value: f64,     // média entre runs
    pub unit: String,
    pub stddev: f64,    // desvio-padrão entre runs (base da margem de erro)
    pub samples: u32,   // nº de runs
}

/// Resultado de uma sessão de benchmark (+ qualidade da medição — PL-2d).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BenchmarkResult {
    pub kind: String,          // synthetic_cpu
    pub suite_version: String, // comparabilidade só dentro da mesma versão
    pub runs: u32,
    pub duration_ms: i64,
    pub metrics: Vec<BenchmarkMetric>,
    pub confidence: u8,        // 0–100 (quão estável foi a medição)
    pub stable: bool,          // confiança >= mínimo e não contaminada
    pub contaminated: bool,    // throttling/temperatura comprometeu a medição
    pub temp_start_c: Option<f64>,
    pub temp_end_c: Option<f64>,
}

/// Sessão persistida (resultado + identidade/rótulo).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BenchmarkSessionInfo {
    pub id: i64,
    pub ts: i64,
    pub kind: String,
    pub label: String,
    pub suite_version: String,
    pub metrics: Vec<BenchmarkMetric>,
    pub confidence: u8,
    pub stable: bool,
    pub contaminated: bool,
}

/// Variabilidade natural (ruído) de uma métrica entre sessões.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoiseEntry {
    pub metric: String,
    pub cv_pct: f64, // coeficiente de variação entre sessões (%)
}

/// Perfil de ruído da máquina para uma suite (aprendido do histórico).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoiseProfile {
    pub suite: String,
    pub sessions: u32,
    pub source: String, // "learned" (>=3 sessões) | "default" (estimativa conservadora)
    pub entries: Vec<NoiseEntry>,
}

/// Veredito de uma métrica na comparação (anti-alegação-falsa).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PerfVerdict {
    Gain,
    Loss,
    NoChange,
    Unstable, // medição instável → não é possível afirmar ganho/perda
}

/// Linha da comparação antes/depois de uma métrica.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComparisonRow {
    pub metric: String,
    pub before: f64,
    pub after: f64,
    pub delta_pct: f64,
    pub margin_pct: f64, // margem de erro (~95%)
    pub verdict: PerfVerdict,
    pub unit: String,
}

/// Comparação completa entre duas sessões.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerfComparison {
    pub before_id: i64,
    pub after_id: i64,
    pub rows: Vec<ComparisonRow>,
    pub summary: String,
    pub confidence: u8, // confiança geral da comparação (0–100)
    pub reliable: bool, // ambas as sessões estáveis e confiança suficiente
}

/// Leitura instantânea da GPU (via NVML quando disponível).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GpuInfo {
    pub name: String,
    pub usage_pct: f64,
    pub vram_used_mb: f64,
    pub vram_total_mb: f64,
    pub clock_mhz: Option<f64>,
    pub temp_c: Option<f64>,
}

/// Fotografia ao vivo do hardware (não persistida) para o dashboard do Lab.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HardwareSnapshot {
    pub gpu: Option<GpuInfo>, // None = sem GPU NVIDIA / NVML indisponível
    pub cpu_temp_c: Option<f64>,
    pub ram_usage_pct: f64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
}

/// Tipo de gargalo identificado por uma janela de amostragem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum BottleneckKind {
    Cpu,
    Gpu,
    Ram,
    Storage,
    Thermal,
    Balanced,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BottleneckReport {
    pub primary: BottleneckKind,
    pub detail: String,
    pub cpu_avg: f64,
    pub gpu_avg: Option<f64>,
    pub ram_avg: f64,
    pub gpu_available: bool,
    pub thermal_available: bool,
}

// ───────────────────────── Optimization Engine (Fase 2) ─────────────────────────

/// Metadado público de uma otimização do catálogo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OptimizationInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk: String, // Safe | Moderate | Advanced | Experimental
    pub expected_impact: String,
    pub requires_elevation: bool,
    pub requires_reboot: bool,
}

/// Decisão do pipeline após a comparação com evidência.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum OptDecision {
    Keep,
    Revert,
    Inconclusive,
}

/// Item de inicialização (Análise de Startup, somente leitura).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location: String, // HKCU | HKLM
}

/// Registro de uma execução do pipeline (com evidência) — para UI e histórico.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OptimizationRunInfo {
    pub id: i64,
    pub ts: i64,
    pub optimization_id: String,
    pub name: String,
    pub status: String,   // applied | kept | reverted | inconclusive | failed
    pub decision: OptDecision,
    pub confidence: u8,
    pub before_session: Option<i64>,
    pub after_session: Option<i64>,
    pub comparison: Option<PerfComparison>,
    pub message: String,
}

// ───────────────────────── Otimização (modelo legado de contrato) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RiskLevel {
    Safe,
    Moderate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlanItem {
    pub tweak_id: String,
    pub label: String,
    pub risk_level: RiskLevel,
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Plan {
    pub reason: String,
    pub items: Vec<PlanItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PlanStatus {
    Committed,
    Compensated,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlanResult {
    pub status: PlanStatus,
    pub snapshot_id: i64,
    pub applied: Vec<String>,
}
