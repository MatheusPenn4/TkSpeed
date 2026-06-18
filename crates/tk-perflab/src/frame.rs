//! PL-3 — pipeline de FPS / frame time a partir de uma sequência de frametimes (ms).
//!
//! O cálculo (FPS médio/min/máx, frametime médio/P95/P99, 1% low, 0.1% low) é
//! Rust puro e testável. A fonte dos frametimes é abstraída por `FrameSource`:
//! `PresentMonFrameSource` (captura real via Intel PresentMon — encapsula o ETW)
//! ou `ReplayFrameSource` (trace para validar o pipeline). FPS reaproveita todo
//! o Confidence Engine: as métricas são divididas em janelas → média + desvio.

use std::path::PathBuf;

use tk_contracts::{BenchmarkMetric, BenchmarkResult};

use crate::aggregate::{aggregate, Aggregate};
use crate::confidence::assess;
use crate::hardware::{cpu_temp_c, GpuCollector};

pub const FPS_SUITE: &str = "fps-1.0.0";

/// Fonte de frametimes (ms) para um processo-alvo, durante uma janela.
pub trait FrameSource {
    fn name(&self) -> &'static str;
    fn capture(&self, target: &str, duration_ms: u64) -> Result<Vec<f64>, String>;
}

// ───────────────────────── Estatística de frames ─────────────────────────

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Percentil (entrada já ordenada ascendente). p em 0..=100.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

fn fps(ft_ms: f64) -> f64 {
    if ft_ms > 0.0 {
        1000.0 / ft_ms
    } else {
        0.0
    }
}

struct ChunkVals {
    fps_avg: f64,
    fps_min: f64,
    fps_max: f64,
    ft_avg: f64,
    ft_p95: f64,
    ft_p99: f64,
    low_1: f64,
    low_01: f64,
}

fn chunk_metrics(ft: &[f64]) -> ChunkVals {
    let mut s = ft.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let ft_avg = mean(ft);
    ChunkVals {
        fps_avg: fps(ft_avg),
        fps_min: fps(*s.last().unwrap_or(&0.0)), // pior frametime → menor FPS
        fps_max: fps(*s.first().unwrap_or(&0.0)),
        ft_avg,
        ft_p95: percentile(&s, 95.0),
        ft_p99: percentile(&s, 99.0),
        low_1: fps(percentile(&s, 99.0)),    // 1% low = FPS no P99 de frametime
        low_01: fps(percentile(&s, 99.9)),   // 0.1% low = FPS no P99.9
    }
}

fn bm(metric: &str, a: Aggregate, unit: &str) -> BenchmarkMetric {
    BenchmarkMetric { metric: metric.into(), value: a.mean, unit: unit.into(), stddev: a.stddev, samples: a.samples }
}

/// Calcula as métricas de FPS/frametime a partir dos frametimes (ms).
/// Divide em até 5 janelas para obter desvio → o Confidence Engine mede a estabilidade.
pub fn frame_stats(frametimes: &[f64]) -> Vec<BenchmarkMetric> {
    if frametimes.is_empty() {
        return Vec::new();
    }
    let n = frametimes.len();
    let k = (n / 50).clamp(1, 5);
    let size = n / k;

    let (mut fa, mut fmin, mut fmax) = (vec![], vec![], vec![]);
    let (mut fta, mut p95, mut p99) = (vec![], vec![], vec![]);
    let (mut l1, mut l01) = (vec![], vec![]);

    for c in 0..k {
        let start = c * size;
        let end = if c == k - 1 { n } else { (c + 1) * size };
        let cv = chunk_metrics(&frametimes[start..end]);
        fa.push(cv.fps_avg);
        fmin.push(cv.fps_min);
        fmax.push(cv.fps_max);
        fta.push(cv.ft_avg);
        p95.push(cv.ft_p95);
        p99.push(cv.ft_p99);
        l1.push(cv.low_1);
        l01.push(cv.low_01);
    }

    vec![
        bm("fps_avg", aggregate(&fa), "fps"),
        bm("fps_min", aggregate(&fmin), "fps"),
        bm("fps_max", aggregate(&fmax), "fps"),
        bm("fps_1pct_low", aggregate(&l1), "fps"),
        bm("fps_01pct_low", aggregate(&l01), "fps"),
        bm("frametime_avg", aggregate(&fta), "ms"),
        bm("frametime_p95", aggregate(&p95), "ms"),
        bm("frametime_p99", aggregate(&p99), "ms"),
    ]
}

// ───────────────────────── Fontes de frames ─────────────────────────

/// Fonte de teste/validação: reproduz uma sequência de frametimes conhecida.
pub struct ReplayFrameSource {
    pub frametimes: Vec<f64>,
}

impl FrameSource for ReplayFrameSource {
    fn name(&self) -> &'static str {
        "replay"
    }
    fn capture(&self, _target: &str, _duration_ms: u64) -> Result<Vec<f64>, String> {
        Ok(self.frametimes.clone())
    }
}

/// Trace sintético (~60fps com micro-stutters) — valida o pipeline sem depender
/// de captura real. Determinístico (LCG). NÃO é medição real de jogo.
pub fn demo_trace() -> Vec<f64> {
    let n = 6000usize;
    let mut state: u64 = 0xDEAD_BEEF_1234_5678;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let jitter = ((state >> 40) % 1000) as f64 / 1000.0; // 0..1
        let mut ft = 16.6 + (jitter - 0.5) * 2.0; // ~15.6..17.6 ms
        if i % 250 == 0 {
            ft += 18.0; // stutter periódico → afeta 1%/0.1% low
        }
        out.push(ft);
    }
    out
}

/// Captura real via Intel PresentMon (encapsula o ETW). Requer o executável e,
/// em geral, privilégios de administrador. Falha graciosa se ausente.
pub struct PresentMonFrameSource {
    exe: PathBuf,
}

impl PresentMonFrameSource {
    /// Localiza `PresentMon*.exe` em ordem de prioridade:
    /// 1. `{exe_dir}/tools/` — recurso embutido no instalador Tauri.
    /// 2. `%APPDATA%\TkSpeed\tools\` — compatibilidade com versões anteriores.
    pub fn locate() -> Option<Self> {
        // 1. Diretório do executável (recurso Tauri bundled)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let named = dir.join("tools").join("PresentMon-x64.exe");
                if named.exists() {
                    return Some(Self { exe: named });
                }
                // Varredura de PresentMon*.exe em tools/
                if let Ok(entries) = std::fs::read_dir(dir.join("tools")) {
                    for e in entries.flatten() {
                        let name = e.file_name().to_string_lossy().to_lowercase();
                        if name.starts_with("presentmon") && name.ends_with(".exe") {
                            return Some(Self { exe: e.path() });
                        }
                    }
                }
            }
        }
        // 2. Legado: %APPDATA%\TkSpeed\tools
        let dir = std::env::var("APPDATA").ok()
            .map(|p| PathBuf::from(p).join("TkSpeed").join("tools"))?;
        let entries = std::fs::read_dir(&dir).ok()?;
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name()?.to_string_lossy().to_lowercase();
            if name.starts_with("presentmon") && name.ends_with(".exe") {
                return Some(Self { exe: p });
            }
        }
        None
    }
}

impl FrameSource for PresentMonFrameSource {
    fn name(&self) -> &'static str {
        "presentmon"
    }

    fn capture(&self, target: &str, duration_ms: u64) -> Result<Vec<f64>, String> {
        let out = std::env::temp_dir().join("tkspeed_fps.csv");
        let _ = std::fs::remove_file(&out);
        let secs = (duration_ms / 1000).max(1).to_string();
        let out_str = out.to_string_lossy().to_string();

        // .arg() encadeado (cada um aceita AsRef<OsStr> — evita array de tipos mistos).
        let mut cmd = std::process::Command::new(&self.exe);
        cmd.arg("-process_name")
            .arg(target)
            .arg("-output_file")
            .arg(&out_str)
            .arg("-timed")
            .arg(&secs)
            .arg("-terminate_after_timed")
            .arg("-no_top")
            .arg("-stop_existing_session");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let status = cmd
            .status()
            .map_err(|e| format!("Falha ao executar o PresentMon: {e} (precisa de privilégios de administrador)"))?;

        if !status.success() {
            return Err("PresentMon retornou erro — verifique elevação (admin) e se o jogo estava em execução.".into());
        }

        let csv = std::fs::read_to_string(&out)
            .map_err(|_| "PresentMon não gerou saída — o jogo estava em foco e usando a GPU?".to_string())?;
        let _ = std::fs::remove_file(&out);
        parse_presentmon_csv(&csv)
    }
}

/// Extrai a coluna de frametime (qualquer cabeçalho contendo "betweenpresents").
fn parse_presentmon_csv(csv: &str) -> Result<Vec<f64>, String> {
    let mut lines = csv.lines();
    let header = lines.next().ok_or("CSV vazio")?;
    let idx = header
        .split(',')
        .position(|h| h.trim().to_lowercase().contains("betweenpresents"))
        .ok_or("Coluna de frametime não encontrada no CSV do PresentMon")?;

    let mut out = Vec::new();
    for line in lines {
        if let Some(field) = line.split(',').nth(idx) {
            if let Ok(v) = field.trim().parse::<f64>() {
                if v > 0.0 {
                    out.push(v);
                }
            }
        }
    }
    if out.is_empty() {
        return Err("Nenhum frame capturado.".into());
    }
    Ok(out)
}

/// Executa uma captura de FPS e devolve um `BenchmarkResult` (kind=game_capture),
/// já com confiança/estabilidade/térmica — pronto para o Confidence Engine e a
/// comparação (mesmo caminho dos demais benchmarks).
pub fn run_fps_capture(
    source: &dyn FrameSource,
    target: &str,
    duration_ms: u64,
) -> Result<BenchmarkResult, String> {
    let temp_start = GpuCollector::new().snapshot().and_then(|g| g.temp_c).or_else(cpu_temp_c);
    let ft = source.capture(target, duration_ms)?;
    if ft.len() < 50 {
        return Err(format!(
            "Poucos frames capturados ({}) — verifique se o jogo estava em foco e usando a GPU.",
            ft.len()
        ));
    }
    let temp_end = GpuCollector::new().snapshot().and_then(|g| g.temp_c).or_else(cpu_temp_c);

    let metrics = frame_stats(&ft);
    let q = assess(&metrics, temp_start, temp_end);
    let runs = metrics.first().map(|m| m.samples).unwrap_or(1);

    Ok(BenchmarkResult {
        kind: "game_capture".into(),
        suite_version: FPS_SUITE.into(),
        runs,
        duration_ms: duration_ms as i64,
        metrics,
        confidence: q.confidence,
        stable: q.stable,
        contaminated: q.contaminated,
        temp_start_c: temp_start,
        temp_end_c: temp_end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(metrics: &[BenchmarkMetric], name: &str) -> f64 {
        metrics.iter().find(|m| m.metric == name).map(|m| m.value).unwrap_or(0.0)
    }

    #[test]
    fn steady_60fps_gives_60() {
        let ft = vec![16.667_f64; 3000];
        let m = frame_stats(&ft);
        assert!((get(&m, "fps_avg") - 60.0).abs() < 1.0, "fps_avg={}", get(&m, "fps_avg"));
        assert!((get(&m, "fps_1pct_low") - 60.0).abs() < 1.0);
        assert!((get(&m, "frametime_avg") - 16.667).abs() < 0.1);
    }

    #[test]
    fn stutters_lower_the_1pct_low() {
        let m = frame_stats(&demo_trace());
        let avg = get(&m, "fps_avg");
        let low1 = get(&m, "fps_1pct_low");
        let low01 = get(&m, "fps_01pct_low");
        assert!(low1 < avg, "1% low ({low1}) deve ser < FPS médio ({avg})");
        assert!(low01 <= low1, "0.1% low ({low01}) deve ser <= 1% low ({low1})");
    }

    #[test]
    fn empty_is_safe() {
        assert!(frame_stats(&[]).is_empty());
    }
}
