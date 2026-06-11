//! TkCore — núcleo: bootstrap, container de DI, ciclo de vida, permissões,
//! event bus interno, janela de telemetria e (futuro) carregamento de plugins.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use tk_contracts::{HardwareInfo, MetricsTick};
use tk_storage::{AnalysisRepo, AuditRepo, Db, InventoryRepo, MetricRepo, PerfRepo, SnapshotRepo};

/// Janela de telemetria mantida em memória para análise (≈2 min @ 1s).
const WINDOW_CAP: usize = 120;

/// Container de composição (Composition Root da Clean Architecture).
/// Detém o pool de DB, a janela de telemetria e o hardware detectado;
/// cria repositórios sob demanda (o pool SQLite é barato de clonar).
#[derive(Clone)]
pub struct AppContext {
    pub db: Db,
    window: Arc<Mutex<VecDeque<MetricsTick>>>,
    hardware: Arc<Mutex<Option<HardwareInfo>>>,
}

impl AppContext {
    /// Bootstrap: abre o DB (aplica migrations) e prepara o estado compartilhado.
    pub async fn bootstrap(db_path: &str) -> anyhow::Result<Self> {
        let db = tk_storage::open(db_path).await?;
        tracing::info!("AppContext inicializado (db={db_path})");
        Ok(Self {
            db,
            window: Arc::new(Mutex::new(VecDeque::with_capacity(WINDOW_CAP))),
            hardware: Arc::new(Mutex::new(None)),
        })
    }

    // ── Acessores de repositório (DI) ──
    pub fn inventory(&self) -> InventoryRepo {
        InventoryRepo::new(self.db.clone())
    }
    pub fn metrics(&self) -> MetricRepo {
        MetricRepo::new(self.db.clone())
    }
    pub fn analysis(&self) -> AnalysisRepo {
        AnalysisRepo::new(self.db.clone())
    }
    pub fn snapshots(&self) -> SnapshotRepo {
        SnapshotRepo::new(self.db.clone())
    }
    pub fn audit(&self) -> AuditRepo {
        AuditRepo::new(self.db.clone())
    }
    /// Serviço de proteção (snapshots + rollback).
    pub fn protection(&self) -> tk_rollback::ProtectionService {
        tk_rollback::ProtectionService::new(self.db.clone())
    }
    /// Repositório do Performance Lab (sessões de benchmark / comparações).
    pub fn perf(&self) -> PerfRepo {
        PerfRepo::new(self.db.clone())
    }
    /// Engine de otimização (loop fechado com evidência).
    pub fn optimize(&self) -> tk_optimize::Engine {
        tk_optimize::Engine::new(self.db.clone())
    }

    // ── Janela de telemetria (alimentada pelo loop de monitoramento) ──
    pub fn push_tick(&self, tick: MetricsTick) {
        let mut w = self.window.lock().unwrap_or_else(|e| e.into_inner());
        if w.len() >= WINDOW_CAP {
            w.pop_front();
        }
        w.push_back(tick);
    }

    /// Cópia da janela atual para análise (não segura o lock durante o uso).
    pub fn recent_window(&self) -> Vec<MetricsTick> {
        self.window
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .cloned()
            .collect()
    }

    pub fn set_hardware(&self, hw: HardwareInfo) {
        *self.hardware.lock().unwrap_or_else(|e| e.into_inner()) = Some(hw);
    }

    pub fn hardware(&self) -> Option<HardwareInfo> {
        self.hardware.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

/// Avalia/concede privilégios — eleva (UAC) apenas quando estritamente necessário.
/// No MVP nenhuma operação exige elevação (rollback opera em HKCU).
pub struct PermissionBroker;

impl PermissionBroker {
    pub fn ensure_elevated(&self) -> Result<(), PermissionError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("elevação negada pelo usuário")]
    Denied,
}
