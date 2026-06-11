//! Detectores de gargalo — funções puras sobre a janela de telemetria.
//! Cada uma retorna um `Finding` real (severidade, impacto, recomendação) ou `None`.

use tk_contracts::{Finding, Severity};

fn finding(kind: &str, severity: Severity, title: &str, impact: &str, solution: &str) -> Finding {
    Finding {
        kind: kind.into(),
        severity,
        title: title.into(),
        impact: impact.into(),
        solution: solution.into(),
    }
}

/// Utilização de CPU continuamente alta (média da janela).
pub fn detect_cpu_sustained(cpu_avg: f64) -> Option<Finding> {
    if cpu_avg >= 85.0 {
        Some(finding(
            "bottleneck_cpu_sustained",
            if cpu_avg >= 93.0 { Severity::High } else { Severity::Medium },
            "Uso de CPU continuamente alto",
            "A CPU está sob carga elevada de forma sustentada, limitando a responsividade do sistema.",
            "Identifique e feche processos pesados em segundo plano; avalie se há malware ou tarefas presas.",
        ))
    } else {
        None
    }
}

/// Picos de CPU com média baixa → uso intermitente anormal (possível stutter).
pub fn detect_cpu_spikes(cpu_avg: f64, cpu_peak: f64) -> Option<Finding> {
    if cpu_peak >= 97.0 && cpu_avg < 50.0 {
        Some(finding(
            "anomaly_cpu_spikes",
            Severity::Low,
            "Picos de CPU intermitentes",
            "Picos curtos de uso total de CPU podem causar travamentos momentâneos (stutter).",
            "Verifique processos que disparam em rajadas (atualizadores, antivírus, indexação).",
        ))
    } else {
        None
    }
}

/// Pressão de memória: uso de RAM alto.
pub fn detect_ram_pressure(ram_used_pct: f64) -> Option<Finding> {
    if ram_used_pct >= 85.0 {
        Some(finding(
            "bottleneck_ram",
            if ram_used_pct >= 92.0 { Severity::High } else { Severity::Medium },
            "Pressão de memória",
            "RAM quase saturada força paginação em disco e gera lentidão perceptível.",
            "Feche aplicativos pesados, reduza abas do navegador ou adicione mais RAM.",
        ))
    } else {
        None
    }
}

/// Pouca memória disponível em valor absoluto.
pub fn detect_low_memory(ram_avail_gb: f64) -> Option<Finding> {
    if ram_avail_gb < 1.0 {
        Some(finding(
            "low_memory_critical",
            Severity::High,
            "Memória disponível crítica",
            "Menos de 1 GB de RAM livre — risco de travamentos e fechamento de aplicativos.",
            "Feche aplicativos imediatamente; considere upgrade de memória.",
        ))
    } else if ram_avail_gb < 2.0 {
        Some(finding(
            "low_memory",
            Severity::Medium,
            "Pouca memória disponível",
            "Menos de 2 GB de RAM livre reduz a folga para multitarefa e jogos.",
            "Libere memória fechando programas ou adicione RAM.",
        ))
    } else {
        None
    }
}

/// Espaço em disco do sistema quase esgotado.
pub fn detect_storage_space(disk_free_pct: f64) -> Option<Finding> {
    if disk_free_pct < 5.0 {
        Some(finding(
            "storage_space_critical",
            Severity::High,
            "Espaço em disco crítico",
            "Menos de 5% livre degrada o Windows e impede atualizações e arquivos temporários.",
            "Execute uma limpeza segura (com quarentena) e mova arquivos grandes para outro volume.",
        ))
    } else if disk_free_pct < 10.0 {
        Some(finding(
            "storage_space_low",
            Severity::Medium,
            "Pouco espaço em disco",
            "Espaço livre baixo começa a afetar desempenho e cache do sistema.",
            "Libere espaço com limpeza segura e desinstale programas não usados.",
        ))
    } else {
        None
    }
}

/// Volume do sistema em disco mecânico (HDD).
pub fn detect_hdd_system(is_ssd: bool) -> Option<Finding> {
    if !is_ssd {
        Some(finding(
            "storage_hdd_system",
            Severity::Low,
            "Disco do sistema é mecânico (HDD)",
            "HDDs têm latência muito maior que SSDs, deixando boot e aplicativos lentos.",
            "Considere migrar o Windows para um SSD NVMe/SATA.",
        ))
    } else {
        None
    }
}

/// Impacto potencial em jogos quando CPU e RAM estão ambas pressionadas.
pub fn detect_gaming_impact(cpu_avg: f64, ram_used_pct: f64) -> Option<Finding> {
    if cpu_avg >= 80.0 && ram_used_pct >= 80.0 {
        Some(finding(
            "gaming_impact",
            Severity::Info,
            "Possível impacto em jogos",
            "CPU e memória pressionadas tendem a reduzir o FPS médio e aumentar o stutter em jogos.",
            "Ative o Game Boost e feche aplicativos secundários antes de jogar.",
        ))
    } else {
        None
    }
}
