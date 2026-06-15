//! Identidade estável de hardware — Machine Fingerprint.
//!
//! Baseada EXCLUSIVAMENTE em: CPU model, physical cores, primary GPU, RAM GB.
//! Não usa: serial, MAC, disco, SID nem informações pessoais.
//! O hash SHA-256 vincula sessões de benchmark e ativações de perfil ao hardware
//! que as gerou — base do Evidence Engine e do Noise Floor discriminado por máquina.

use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use sysinfo::System;
use tk_perflab::GpuCollector;

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Identidade de hardware desta máquina.
#[derive(Debug, Clone)]
pub struct MachineFingerprint {
    /// SHA-256 hex de `"{cpu_model}:{physical_cores}:{primary_gpu}:{ram_gb}"`.
    pub fingerprint: String,
    pub cpu_model: String,
    pub physical_cores: u32,
    pub primary_gpu: String,
    pub ram_gb: u32,
}

/// Detecta o hardware atual e computa o fingerprint SHA-256.
///
/// Chamada síncrona (usa sysinfo + NVML, < 100 ms).
/// Em uso normal, prefira `fingerprint()` que cacheia via `OnceLock`.
pub fn build() -> MachineFingerprint {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_model = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown_cpu".into());

    // Prefere contagem de cores físicos; cai para lógicos se não disponível.
    let physical_cores = sys.physical_core_count()
        .unwrap_or_else(|| sys.cpus().len()) as u32;

    let primary_gpu = GpuCollector::new()
        .snapshot()
        .map(|g| g.name)
        .unwrap_or_else(|| "no_gpu".into());

    let ram_gb = (sys.total_memory() as f64 / BYTES_PER_GB).round() as u32;

    let fingerprint = hash_components(&cpu_model, physical_cores, &primary_gpu, ram_gb);

    MachineFingerprint { fingerprint, cpu_model, physical_cores, primary_gpu, ram_gb }
}

/// Retorna o fingerprint como String hex de 64 chars (SHA-256).
/// Inicializa uma única vez via `OnceLock` — hardware detectado somente na 1ª chamada.
pub fn fingerprint() -> String {
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED.get_or_init(|| build().fingerprint).clone()
}

/// Computa o hash SHA-256 a partir dos quatro componentes.
/// Função interna e de teste — separada de `build()` para ser testável sem hardware real.
pub(crate) fn hash_components(cpu: &str, cores: u32, gpu: &str, ram: u32) -> String {
    let input = format!("{cpu}:{cores}:{gpu}:{ram}");
    let result = Sha256::digest(input.as_bytes());
    result.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SHA-256 output ──────────────────────────────────────────────────────────

    #[test]
    fn fingerprint_is_64_hex_chars() {
        let fp = hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 32);
        assert_eq!(fp.len(), 64, "SHA-256 deve produzir 64 chars hex");
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()), "deve ser hex válido");
    }

    #[test]
    fn hash_is_deterministic() {
        let a = hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 32);
        let b = hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 32);
        assert_eq!(a, b, "mesma entrada = mesmo hash");
    }

    // ── Qualquer mudança de componente altera o hash ────────────────────────────

    fn base() -> String {
        hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 32)
    }

    #[test]
    fn changing_cpu_changes_hash() {
        let changed = hash_components("Intel Core i9-13900K", 8, "RTX 4070", 32);
        assert_ne!(base(), changed, "CPU diferente = hash diferente");
    }

    #[test]
    fn changing_cores_changes_hash() {
        let changed = hash_components("AMD Ryzen 7 5700X", 16, "RTX 4070", 32);
        assert_ne!(base(), changed, "cores diferentes = hash diferente");
    }

    #[test]
    fn changing_gpu_changes_hash() {
        let changed = hash_components("AMD Ryzen 7 5700X", 8, "RTX 3080", 32);
        assert_ne!(base(), changed, "GPU diferente = hash diferente");
    }

    #[test]
    fn changing_ram_changes_hash() {
        let changed = hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 64);
        assert_ne!(base(), changed, "RAM diferente = hash diferente");
    }

    // ── build() retorna dados plausíveis ──────────────────────────────────────

    #[test]
    fn build_cpu_model_non_empty() {
        assert!(!build().cpu_model.is_empty(), "cpu_model não deve ser vazio");
    }

    #[test]
    fn build_physical_cores_nonzero() {
        assert!(build().physical_cores > 0, "physical_cores deve ser > 0");
    }

    #[test]
    fn build_ram_gb_nonzero() {
        assert!(build().ram_gb > 0, "ram_gb deve ser > 0");
    }

    #[test]
    fn build_fingerprint_is_64_chars() {
        assert_eq!(build().fingerprint.len(), 64);
    }

    // ── Separação semântica ───────────────────────────────────────────────────

    #[test]
    fn no_gpu_differs_from_with_gpu() {
        let with_gpu = hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 32);
        let no_gpu = hash_components("AMD Ryzen 7 5700X", 8, "no_gpu", 32);
        assert_ne!(with_gpu, no_gpu);
    }

    #[test]
    fn input_format_uses_colon_separator() {
        // Garantia explícita do formato do input para rastreabilidade.
        // Se este teste quebrar, significa que o formato mudou e os hashes históricos
        // serão incompatíveis com novos.
        let fp_explicit = {
            let input = "AMD Ryzen 7 5700X:8:RTX 4070:32";
            let result = Sha256::digest(input.as_bytes());
            result.iter().fold(String::with_capacity(64), |mut s, b| {
                use std::fmt::Write;
                let _ = write!(s, "{b:02x}");
                s
            })
        };
        let fp_fn = hash_components("AMD Ryzen 7 5700X", 8, "RTX 4070", 32);
        assert_eq!(fp_explicit, fp_fn, "formato da string de hash deve ser cpu:cores:gpu:ram");
    }
}
