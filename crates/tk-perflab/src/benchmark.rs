//! PL-2 — benchmarks sintéticos determinísticos: CPU, RAM, Storage e Completo.
//! Tudo em Rust puro (sem elevação, sem libs externas). O Storage usa um arquivo
//! temporário em sandbox e o remove ao final — nunca toca dados do usuário.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Instant;

use tk_contracts::{BenchmarkMetric, BenchmarkResult};

use crate::aggregate::aggregate;
use crate::collector::{MetricsCollector, SysinfoCollector};
use crate::confidence::assess;
use crate::hardware::{cpu_temp_c, GpuCollector};

pub const CPU_SUITE: &str = "cpu-1.0.0";
pub const RAM_SUITE: &str = "ram-1.0.0";
pub const IO_SUITE: &str = "io-1.0.0";
pub const COMPLETE_SUITE: &str = "complete-1.0.0";
/// Compat: versão exposta por `SUITE_VERSION` (CPU) — mantida para PL-1.
pub const SUITE_VERSION: &str = CPU_SUITE;

const KERNEL_ITERS: u64 = 300_000_000;
const SCORE_SCALE: f64 = 100_000.0;

fn metric(name: &str, value: f64, unit: &str, stddev: f64, samples: u32) -> BenchmarkMetric {
    BenchmarkMetric { metric: name.into(), value, unit: unit.into(), stddev, samples }
}

// ───────────────────────── CPU ─────────────────────────

fn cpu_kernel(iters: u64) -> u64 {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..iters {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    }
    x
}

fn score_single() -> f64 {
    let t = Instant::now();
    std::hint::black_box(cpu_kernel(KERNEL_ITERS));
    (KERNEL_ITERS as f64) / t.elapsed().as_secs_f64().max(1e-9) / SCORE_SCALE
}

fn score_multi(threads: usize) -> f64 {
    let t = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|_| std::thread::spawn(|| std::hint::black_box(cpu_kernel(KERNEL_ITERS))))
        .collect();
    for h in handles {
        let _ = h.join();
    }
    (KERNEL_ITERS as f64) * threads as f64 / t.elapsed().as_secs_f64().max(1e-9) / SCORE_SCALE
}

fn cpu_metrics(runs: u32) -> Vec<BenchmarkMetric> {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut collector = SysinfoCollector::new();
    let _ = collector.start();

    let _ = score_single(); // warmup
    let _ = score_multi(threads);

    let (mut singles, mut multis, mut loads) = (vec![], vec![], vec![]);
    for _ in 0..runs {
        singles.push(score_single());
        multis.push(score_multi(threads));
        for p in collector.poll() {
            if p.metric == "cpu_usage" {
                loads.push(p.value);
            }
        }
    }
    let s = aggregate(&singles);
    let m = aggregate(&multis);
    let l = aggregate(&loads);
    vec![
        metric("cpu_single", s.mean, "score", s.stddev, s.samples),
        metric("cpu_multi", m.mean, "score", m.stddev, m.samples),
        metric("cpu_load_pct", l.mean, "%", l.stddev, l.samples),
    ]
}

// ───────────────────────── RAM ─────────────────────────

/// Largura de banda de leitura (GB/s) varrendo um buffer maior que o cache.
fn ram_bandwidth_gbs() -> f64 {
    let n = 32 * 1024 * 1024; // 32M u64 = 256 MB
    let buf: Vec<u64> = (0..n as u64).collect();
    let passes = 3u64;
    let t = Instant::now();
    let mut acc = 0u64;
    for _ in 0..passes {
        for &v in &buf {
            acc = acc.wrapping_add(v);
        }
    }
    std::hint::black_box(acc);
    let bytes = (n * 8) as f64 * passes as f64;
    bytes / t.elapsed().as_secs_f64().max(1e-9) / 1e9
}

/// Latência de acesso aleatório (ns/acesso) via pointer-chasing num ciclo único.
fn ram_latency_ns() -> f64 {
    let n: usize = 4 * 1024 * 1024; // 4M usize = 32 MB
    // Permutação determinística (Fisher–Yates com LCG) → ciclo único de ponteiros.
    let mut perm: Vec<usize> = (0..n).collect();
    let mut state: u64 = 0x1234_5678_9ABC_DEF0;
    for i in (1..n).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        perm.swap(i, j);
    }
    let mut next = vec![0usize; n];
    for i in 0..n {
        next[perm[i]] = perm[(i + 1) % n];
    }

    let steps = 20_000_000u64;
    let t = Instant::now();
    let mut p = 0usize;
    for _ in 0..steps {
        p = next[p];
    }
    std::hint::black_box(p);
    t.elapsed().as_secs_f64() * 1e9 / steps as f64
}

fn ram_metrics(runs: u32) -> Vec<BenchmarkMetric> {
    let _ = ram_bandwidth_gbs(); // warmup
    let (mut bw, mut lat) = (vec![], vec![]);
    for _ in 0..runs {
        bw.push(ram_bandwidth_gbs());
        lat.push(ram_latency_ns());
    }
    let b = aggregate(&bw);
    let l = aggregate(&lat);
    vec![
        metric("ram_bandwidth_gbs", b.mean, "GB/s", b.stddev, b.samples),
        metric("ram_latency_ns", l.mean, "ns", l.stddev, l.samples),
    ]
}

// ───────────────────────── Storage ─────────────────────────

const IO_FILE_SIZE: usize = 96 * 1024 * 1024; // 96 MB
const IO_CHUNK: usize = 4 * 1024 * 1024; // 4 MB
const IO_RAND_OPS: u64 = 2000;
const IO_RAND_BLOCK: usize = 4096;

/// Retorna (seq_write_MB/s, seq_read_MB/s, rand_read_IOPS, latência_us).
/// Observação: a leitura sequencial pode tocar o cache do SO (sem admin não há
/// como descartá-lo) — válido para comparação relativa na MESMA máquina.
fn io_once() -> std::io::Result<(f64, f64, f64, f64)> {
    let path = std::env::temp_dir().join("tkspeed_bench.dat");
    let chunk = vec![0xABu8; IO_CHUNK];

    // Escrita sequencial.
    let t = Instant::now();
    {
        let mut f = File::create(&path)?;
        let mut written = 0;
        while written < IO_FILE_SIZE {
            f.write_all(&chunk)?;
            written += chunk.len();
        }
        f.sync_all()?;
    }
    let w_mbs = IO_FILE_SIZE as f64 / 1e6 / t.elapsed().as_secs_f64().max(1e-9);

    // Leitura sequencial.
    let t = Instant::now();
    {
        let mut f = File::open(&path)?;
        let mut rbuf = vec![0u8; IO_CHUNK];
        loop {
            let n = f.read(&mut rbuf)?;
            if n == 0 {
                break;
            }
            std::hint::black_box(rbuf[0]);
        }
    }
    let r_mbs = IO_FILE_SIZE as f64 / 1e6 / t.elapsed().as_secs_f64().max(1e-9);

    // Leitura aleatória (IOPS + latência) — offsets via LCG determinístico.
    let mut f = OpenOptions::new().read(true).open(&path)?;
    let mut small = vec![0u8; IO_RAND_BLOCK];
    let range = (IO_FILE_SIZE - IO_RAND_BLOCK) as u64;
    let mut state: u64 = 0xC0FF_EE12_3456_789A;
    let t = Instant::now();
    for _ in 0..IO_RAND_OPS {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let off = (state >> 33) % range;
        f.seek(SeekFrom::Start(off))?;
        f.read_exact(&mut small)?;
        std::hint::black_box(small[0]);
    }
    let secs = t.elapsed().as_secs_f64().max(1e-9);
    let iops = IO_RAND_OPS as f64 / secs;
    let latency_us = secs * 1e6 / IO_RAND_OPS as f64;

    let _ = std::fs::remove_file(&path);
    Ok((w_mbs, r_mbs, iops, latency_us))
}

fn io_metrics(runs: u32) -> Vec<BenchmarkMetric> {
    let (mut w, mut r, mut iops, mut lat) = (vec![], vec![], vec![], vec![]);
    for _ in 0..runs {
        if let Ok((wv, rv, iv, lv)) = io_once() {
            w.push(wv);
            r.push(rv);
            iops.push(iv);
            lat.push(lv);
        }
    }
    let aw = aggregate(&w);
    let ar = aggregate(&r);
    let ai = aggregate(&iops);
    let al = aggregate(&lat);
    vec![
        metric("io_seq_write_mbs", aw.mean, "MB/s", aw.stddev, aw.samples),
        metric("io_seq_read_mbs", ar.mean, "MB/s", ar.stddev, ar.samples),
        metric("io_rand_read_iops", ai.mean, "IOPS", ai.stddev, ai.samples),
        metric("io_latency_us", al.mean, "µs", al.stddev, al.samples),
    ]
}

// ───────────────────────── Orquestração ─────────────────────────

/// Temperatura representativa (°C): CPU via sysinfo (best-effort) ou, na falta,
/// temperatura da GPU via NVML. `None` quando nenhum sensor está disponível.
fn read_temp() -> Option<f64> {
    cpu_temp_c().or_else(|| GpuCollector::new().snapshot().and_then(|g| g.temp_c))
}

/// Monta o resultado calculando confiança/estabilidade/contaminação (PL-2d).
fn finish(
    kind: &str,
    suite: &str,
    runs: u32,
    started: Instant,
    metrics: Vec<BenchmarkMetric>,
    temp_start: Option<f64>,
    temp_end: Option<f64>,
) -> BenchmarkResult {
    let q = assess(&metrics, temp_start, temp_end);
    BenchmarkResult {
        kind: kind.into(),
        suite_version: suite.into(),
        runs,
        duration_ms: started.elapsed().as_millis() as i64,
        metrics,
        confidence: q.confidence,
        stable: q.stable,
        contaminated: q.contaminated,
        temp_start_c: temp_start,
        temp_end_c: temp_end,
    }
}

pub fn run_cpu(runs: u32) -> BenchmarkResult {
    let runs = runs.clamp(1, 20);
    let t = Instant::now();
    let ts = read_temp();
    let m = cpu_metrics(runs);
    finish("synthetic_cpu", CPU_SUITE, runs, t, m, ts, read_temp())
}

pub fn run_ram(runs: u32) -> BenchmarkResult {
    let runs = runs.clamp(1, 20);
    let t = Instant::now();
    let ts = read_temp();
    let m = ram_metrics(runs);
    finish("synthetic_ram", RAM_SUITE, runs, t, m, ts, read_temp())
}

pub fn run_io(runs: u32) -> BenchmarkResult {
    let runs = runs.clamp(1, 10);
    let t = Instant::now();
    let ts = read_temp();
    let m = io_metrics(runs);
    finish("synthetic_io", IO_SUITE, runs, t, m, ts, read_temp())
}

pub fn run_complete(runs: u32) -> BenchmarkResult {
    let runs = runs.clamp(1, 10);
    let t = Instant::now();
    let ts = read_temp();
    let mut metrics = cpu_metrics(runs);
    metrics.extend(ram_metrics(runs));
    metrics.extend(io_metrics(runs.min(3)));
    finish("complete", COMPLETE_SUITE, runs, t, metrics, ts, read_temp())
}

/// Dispatcher por tipo (UI envia "cpu" | "ram" | "io" | "complete").
pub fn run_benchmark(kind: &str, runs: u32) -> BenchmarkResult {
    match kind {
        "ram" => run_ram(runs),
        "io" => run_io(runs),
        "complete" => run_complete(runs),
        _ => run_cpu(runs),
    }
}
