//! TkRollback — snapshots imutáveis e rollback funcional, auditável e seguro.
//!
//! O `ProtectionService` orquestra o ciclo:
//!   capturar estado → criar snapshot → aplicar → validar → (rollback) → verificar
//! sobre os repositórios reais (`SnapshotRepo`/`AuditRepo`) e a operação-piloto
//! em registro HKCU (sem elevação, 100% reversível). Nada é excluído de forma
//! permanente; restaurações são validadas por hash de integridade.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use tk_contracts::{ProtectionState, RollbackOutcome, SelfTestReport, SelfTestStep, SnapshotInfo};
use tk_platform_win::{power, registry};
use tk_storage::{AuditRepo, Db, SnapshotEntryRow, SnapshotRepo};

// ── Operação-piloto: HKCU\Control Panel\Desktop\MenuShowDelay ──
const PILOT_SUBKEY: &str = "Control Panel\\Desktop";
const PILOT_NAME: &str = "MenuShowDelay";
const PILOT_KEY: &str = "Control Panel\\Desktop\\MenuShowDelay"; // target_key (HKCU implícito)
const PILOT_NEW: &str = "0"; // menus instantâneos
const TARGET_LABEL: &str = "HKCU\\Control Panel\\Desktop\\MenuShowDelay";

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error(transparent)]
    Storage(#[from] tk_storage::StorageError),
    #[error("registro: {0}")]
    Platform(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("snapshot {0} não encontrado")]
    NotFound(i64),
    #[error("integridade do snapshot inválida")]
    Integrity,
    #[error("validação falhou: {0}")]
    Validation(String),
}

impl From<tk_platform_win::PlatformError> for RollbackError {
    fn from(e: tk_platform_win::PlatformError) -> Self {
        RollbackError::Platform(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RollbackError>;

/// Representação serializável do valor de registro capturado (ausente = None).
#[derive(Serialize, Deserialize)]
struct RegVal {
    value: Option<String>,
}

/// Representação serializável de um plano de energia (GUID).
#[derive(Serialize, Deserialize)]
struct PowerVal {
    guid: String,
}

/// Valor DWORD de registro (ausente = None).
#[derive(Serialize, Deserialize)]
struct DwordVal {
    value: Option<u32>,
}

/// Arquivo movido para quarentena (caminho de quarentena + tamanho).
#[derive(Serialize, Deserialize)]
struct FileVal {
    quarantine: String,
    size: u64,
}

/// Ação reversível genérica capturada antes de uma mutação (base do snapshot).
/// Permite que qualquer otimização seja revertida pela infra já validada.
#[derive(Debug, Clone)]
pub enum ReversibleAction {
    RegistryHkcu {
        subkey: String,
        name: String,
        old: Option<String>,
        new: Option<String>,
    },
    RegistryHkcuDword {
        subkey: String,
        name: String,
        old: Option<u32>,
        new: Option<u32>,
    },
    PowerPlan {
        old_guid: String,
        new_guid: String,
    },
    /// Move `original` → `quarantine` (limpeza reversível; nunca apaga).
    FileQuarantine {
        original: String,
        quarantine: String,
        size: u64,
    },
}

fn action_to_entry(a: &ReversibleAction) -> SnapshotEntryRow {
    match a {
        ReversibleAction::RegistryHkcu { subkey, name, old, new } => SnapshotEntryRow {
            target_type: "registry".into(),
            target_key: format!("{subkey}\\{name}"),
            old_value_json: Some(serde_json::to_string(&RegVal { value: old.clone() }).unwrap_or_default()),
            new_value_json: Some(serde_json::to_string(&RegVal { value: new.clone() }).unwrap_or_default()),
        },
        ReversibleAction::RegistryHkcuDword { subkey, name, old, new } => SnapshotEntryRow {
            target_type: "registry_dword".into(),
            target_key: format!("{subkey}\\{name}"),
            old_value_json: Some(serde_json::to_string(&DwordVal { value: *old }).unwrap_or_default()),
            new_value_json: Some(serde_json::to_string(&DwordVal { value: *new }).unwrap_or_default()),
        },
        ReversibleAction::PowerPlan { old_guid, new_guid } => SnapshotEntryRow {
            target_type: "power".into(),
            target_key: "active_scheme".into(),
            old_value_json: Some(serde_json::to_string(&PowerVal { guid: old_guid.clone() }).unwrap_or_default()),
            new_value_json: Some(serde_json::to_string(&PowerVal { guid: new_guid.clone() }).unwrap_or_default()),
        },
        ReversibleAction::FileQuarantine { original, quarantine, size } => SnapshotEntryRow {
            target_type: "file".into(),
            target_key: original.clone(),
            old_value_json: Some(serde_json::to_string(&FileVal { quarantine: quarantine.clone(), size: *size }).unwrap_or_default()),
            new_value_json: None,
        },
    }
}

/// Aplica o estado NOVO de cada ação (genérico). Erros de registro/power abortam;
/// arquivos bloqueados são pulados (best-effort). Retorna bytes liberados (arquivos).
pub fn apply_actions(actions: &[ReversibleAction]) -> Result<u64> {
    let mut freed = 0u64;
    for a in actions {
        match a {
            ReversibleAction::RegistryHkcu { subkey, name, new, .. } => match new {
                Some(v) => registry::write_string(subkey, name, v)?,
                None => registry::delete_value(subkey, name)?,
            },
            ReversibleAction::RegistryHkcuDword { subkey, name, new, .. } => match new {
                Some(v) => registry::write_u32(subkey, name, *v)?,
                None => registry::delete_value(subkey, name)?,
            },
            ReversibleAction::PowerPlan { new_guid, .. } => power::set_active_scheme(new_guid)?,
            ReversibleAction::FileQuarantine { original, quarantine, size } => {
                if let Some(parent) = std::path::Path::new(quarantine).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if std::fs::rename(original, quarantine).is_ok() {
                    freed += *size; // arquivo bloqueado → pulado
                }
            }
        }
    }
    Ok(freed)
}

/// Verifica que o estado NOVO foi efetivado (registro/power). Arquivos não são
/// verificados aqui (limpeza é best-effort; quem usa SpaceFreed não chama verify).
pub fn verify_actions(actions: &[ReversibleAction]) -> Result<bool> {
    for a in actions {
        let ok = match a {
            ReversibleAction::RegistryHkcu { subkey, name, new, .. } => {
                registry::read_string(subkey, name)? == *new
            }
            ReversibleAction::RegistryHkcuDword { subkey, name, new, .. } => {
                registry::read_u32(subkey, name)? == *new
            }
            ReversibleAction::PowerPlan { new_guid, .. } => {
                power::get_active_scheme().ok().as_deref() == Some(new_guid.as_str())
            }
            ReversibleAction::FileQuarantine { .. } => true,
        };
        if !ok {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Serviço de proteção: criação de snapshots, aplicação reversível e rollback.
pub struct ProtectionService {
    db: Db,
}

impl ProtectionService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn snapshots(&self) -> SnapshotRepo {
        SnapshotRepo::new(self.db.clone())
    }
    fn audit(&self) -> AuditRepo {
        AuditRepo::new(self.db.clone())
    }

    /// Cria um snapshot a partir de um conjunto de ações reversíveis (genérico).
    /// `machine_fingerprint`: hash da máquina (V4 Evidence Engine). Passar `None` em
    /// contextos sem acesso ao módulo machine (testes, demo interno).
    pub async fn create_snapshot(
        &self,
        reason: &str,
        actions: &[ReversibleAction],
        machine_fingerprint: Option<&str>,
    ) -> Result<i64> {
        let entries: Vec<SnapshotEntryRow> = actions.iter().map(action_to_entry).collect();
        let hash = integrity(&entries);
        let snap_id = self.snapshots().create(reason, &hash, machine_fingerprint).await?;
        for e in &entries {
            self.snapshots().add_entry(snap_id, e).await?;
        }
        self.audit()
            .log("system", "snapshot.created", &format!("{{\"snapshot\":{snap_id},\"reason\":\"{reason}\"}}"))
            .await?;
        Ok(snap_id)
    }

    /// Ciclo: captura estado → cria snapshot+entrada → aplica → valida.
    /// Em falha de validação, reverte automaticamente (fail-safe).
    pub async fn apply_demo(&self) -> Result<SnapshotInfo> {
        let old = registry::read_string(PILOT_SUBKEY, PILOT_NAME)?;
        let entry = SnapshotEntryRow {
            target_type: "registry".into(),
            target_key: PILOT_KEY.into(),
            old_value_json: Some(serde_json::to_string(&RegVal { value: old.clone() })?),
            new_value_json: Some(serde_json::to_string(&RegVal {
                value: Some(PILOT_NEW.into()),
            })?),
        };
        let hash = integrity(std::slice::from_ref(&entry));

        // 1) snapshot (estado original)
        let reason = format!("Demo reversível: {PILOT_NAME} → {PILOT_NEW}");
        let snap_id = self.snapshots().create(&reason, &hash, None).await?;
        self.snapshots().add_entry(snap_id, &entry).await?;
        self.audit()
            .log("system", "snapshot.created", &format!("{{\"snapshot\":{snap_id},\"target\":\"{TARGET_LABEL}\"}}"))
            .await?;

        // 2) apply
        registry::write_string(PILOT_SUBKEY, PILOT_NAME, PILOT_NEW)?;
        self.audit()
            .log("user", "optimize.applied", &format!("{{\"snapshot\":{snap_id},\"value\":\"{PILOT_NEW}\"}}"))
            .await?;

        // 3) validate (pós-condição); se falhar, rollback automático
        let now = registry::read_string(PILOT_SUBKEY, PILOT_NAME)?;
        if now.as_deref() != Some(PILOT_NEW) {
            self.restore_entry(&entry)?;
            self.snapshots().set_status(snap_id, "restored").await?;
            self.audit()
                .log("system", "optimize.rolledback", "validação pós-apply falhou")
                .await?;
            return Err(RollbackError::Validation(
                "valor aplicado não foi confirmado".into(),
            ));
        }

        self.snapshot_info(snap_id).await
    }

    /// Restaura exatamente o estado capturado por um snapshot e verifica.
    pub async fn rollback(&self, snapshot_id: i64) -> Result<RollbackOutcome> {
        let snap = self
            .snapshots()
            .get(snapshot_id)
            .await?
            .ok_or(RollbackError::NotFound(snapshot_id))?;
        let entries = self.snapshots().entries(snapshot_id).await?;

        // Integridade: o estado salvo não pode ter sido adulterado.
        if integrity(&entries) != snap.integrity_hash {
            self.audit()
                .log("system", "rollback.failed", &format!("{{\"snapshot\":{snapshot_id},\"reason\":\"integrity\"}}"))
                .await?;
            return Err(RollbackError::Integrity);
        }

        let mut restored = 0u32;
        for e in &entries {
            // restore_entry restaura E verifica (por tipo). Erro → aborta + audita.
            if let Err(err) = self.restore_entry(e) {
                self.audit()
                    .log("system", "rollback.failed", &format!("{{\"snapshot\":{snapshot_id},\"target\":\"{}\"}}", e.target_key))
                    .await?;
                return Err(err);
            }
            restored += 1;
        }

        self.snapshots().set_status(snapshot_id, "restored").await?;
        self.audit()
            .log("user", "rollback.completed", &format!("{{\"snapshot\":{snapshot_id},\"restored\":{restored}}}"))
            .await?;

        Ok(RollbackOutcome {
            snapshot_id,
            restored,
            ok: true,
            message: "Estado restaurado e verificado com sucesso.".into(),
        })
    }

    /// Restaura UMA entrada ao estado original (por tipo) e VERIFICA. Idempotente.
    fn restore_entry(&self, e: &SnapshotEntryRow) -> Result<()> {
        match e.target_type.as_str() {
            "registry" => {
                let old = parse_regval(e.old_value_json.as_deref())?;
                let (sub, name) = split_key(&e.target_key);
                match &old.value {
                    Some(v) => registry::write_string(sub, name, v)?,
                    None => registry::delete_value(sub, name)?,
                }
                let now = registry::read_string(sub, name)?;
                if now != old.value {
                    return Err(RollbackError::Validation(format!("registry {}", e.target_key)));
                }
            }
            "registry_dword" => {
                let old: DwordVal =
                    serde_json::from_str(e.old_value_json.as_deref().unwrap_or("{\"value\":null}"))?;
                let (sub, name) = split_key(&e.target_key);
                match old.value {
                    Some(v) => registry::write_u32(sub, name, v)?,
                    None => registry::delete_value(sub, name)?,
                }
                if registry::read_u32(sub, name)? != old.value {
                    return Err(RollbackError::Validation(format!("registry_dword {}", e.target_key)));
                }
            }
            "power" => {
                let pv: PowerVal =
                    serde_json::from_str(e.old_value_json.as_deref().unwrap_or("{\"guid\":\"\"}"))?;
                power::set_active_scheme(&pv.guid)?;
                let now = power::get_active_scheme().ok();
                if now.as_deref() != Some(pv.guid.as_str()) {
                    return Err(RollbackError::Validation("plano de energia não confirmado".into()));
                }
            }
            "file" => {
                // Restaura da quarentena para o local original.
                let fv: FileVal =
                    serde_json::from_str(e.old_value_json.as_deref().unwrap_or("{\"quarantine\":\"\",\"size\":0}"))?;
                let original = &e.target_key;
                if std::path::Path::new(&fv.quarantine).exists() {
                    if let Some(parent) = std::path::Path::new(original).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    std::fs::rename(&fv.quarantine, original)
                        .map_err(|err| RollbackError::Platform(err.to_string()))?;
                }
                // Se o arquivo não foi movido na aplicação (bloqueado), ele já está no lugar.
            }
            other => {
                tracing::warn!("restore_entry: tipo não suportado: {other}");
            }
        }
        Ok(())
    }

    pub async fn list(&self, limit: i64) -> Result<Vec<SnapshotInfo>> {
        let rows = self.snapshots().list(limit).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let changes = self.snapshots().entries(r.id).await?.len() as u32;
            out.push(SnapshotInfo {
                id: r.id,
                ts: r.ts,
                reason: r.reason,
                status: r.status,
                changes,
                target: TARGET_LABEL.into(),
            });
        }
        Ok(out)
    }

    async fn snapshot_info(&self, id: i64) -> Result<SnapshotInfo> {
        let r = self
            .snapshots()
            .get(id)
            .await?
            .ok_or(RollbackError::NotFound(id))?;
        let changes = self.snapshots().entries(id).await?.len() as u32;
        Ok(SnapshotInfo {
            id: r.id,
            ts: r.ts,
            reason: r.reason,
            status: r.status,
            changes,
            target: TARGET_LABEL.into(),
        })
    }

    pub async fn protection_state(&self) -> Result<ProtectionState> {
        let total = self.snapshots().count().await? as u32;
        let last_snapshot = self.list(1).await?.into_iter().next();
        let last_rollback_ts = self.audit().last_ts("rollback.completed").await?;
        let status = if total == 0 { "Sem snapshots" } else { "Protegido" }.to_string();
        Ok(ProtectionState {
            status,
            total,
            last_snapshot,
            last_rollback_ts,
        })
    }

    /// Cenário completo de validação: snapshot → aplicação → verificação →
    /// rollback → verificação final. Retorna um relatório passo a passo.
    pub async fn selftest(&self) -> Result<SelfTestReport> {
        let mut steps = Vec::new();

        let before = registry::read_string(PILOT_SUBKEY, PILOT_NAME).unwrap_or(None);
        steps.push(step("Captura inicial", &format!("{PILOT_NAME} = {before:?}"), true));

        let info = match self.apply_demo().await {
            Ok(i) => i,
            Err(e) => {
                steps.push(step("Snapshot + Aplicação", &e.to_string(), false));
                return Ok(SelfTestReport { steps, passed: false });
            }
        };
        let applied = registry::read_string(PILOT_SUBKEY, PILOT_NAME).unwrap_or(None);
        steps.push(step(
            "Snapshot + Aplicação",
            &format!("snapshot #{} criado; valor agora = {applied:?}", info.id),
            applied.as_deref() == Some(PILOT_NEW),
        ));

        match self.rollback(info.id).await {
            Ok(o) => steps.push(step("Rollback", &o.message, o.ok)),
            Err(e) => {
                steps.push(step("Rollback", &e.to_string(), false));
                return Ok(SelfTestReport { steps, passed: false });
            }
        }

        let after = registry::read_string(PILOT_SUBKEY, PILOT_NAME).unwrap_or(None);
        steps.push(step(
            "Verificação final",
            &format!("valor restaurado = {after:?} (original = {before:?})"),
            after == before,
        ));

        let passed = steps.iter().all(|s| s.ok);
        Ok(SelfTestReport { steps, passed })
    }
}

fn step(name: &str, detail: &str, ok: bool) -> SelfTestStep {
    SelfTestStep {
        name: name.into(),
        detail: detail.into(),
        ok,
    }
}

fn parse_regval(json: Option<&str>) -> Result<RegVal> {
    Ok(serde_json::from_str(json.unwrap_or("{\"value\":null}"))?)
}

/// Divide "Control Panel\\Desktop\\MenuShowDelay" em (subkey, nome do valor).
fn split_key(full: &str) -> (&str, &str) {
    match full.rsplit_once('\\') {
        Some((sub, name)) => (sub, name),
        None => ("", full),
    }
}

/// Hash de integridade determinístico sobre (target_key, old_value_json).
/// `DefaultHasher::new()` usa chave fixa → estável entre execuções.
fn integrity(entries: &[SnapshotEntryRow]) -> String {
    let mut h = DefaultHasher::new();
    for e in entries {
        e.target_key.hash(&mut h);
        e.old_value_json.hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

// ───────────────────────── Testes (A2.1 + A2.4) ─────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, old: &str) -> SnapshotEntryRow {
        SnapshotEntryRow {
            target_type: "registry".into(),
            target_key: key.into(),
            old_value_json: Some(old.into()),
            new_value_json: None,
        }
    }

    // ── integridade: determinística e sensível a alterações ──
    #[test]
    fn integrity_is_deterministic_and_sensitive() {
        let a = [entry("A\\B", "{\"value\":\"x\"}")];
        let b = [entry("A\\B", "{\"value\":\"x\"}")];
        assert_eq!(integrity(&a), integrity(&b), "mesmas entradas → mesmo hash");

        let c = [entry("A\\B", "{\"value\":\"y\"}")]; // valor diferente
        assert_ne!(integrity(&a), integrity(&c), "valor diferente → hash diferente");

        let d = [entry("A\\C", "{\"value\":\"x\"}")]; // chave diferente
        assert_ne!(integrity(&a), integrity(&d), "chave diferente → hash diferente");
    }

    // ── FileQuarantine: ciclo completo com DB (cross-platform) ──
    // capture(arquivo)→snapshot→apply(move p/ quarentena)→rollback(restaura)→verify
    #[tokio::test]
    async fn file_quarantine_full_cycle_with_db() {
        let dir = std::env::temp_dir().join(format!("tkspeed_fq_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let original = dir.join("victim.tmp");
        std::fs::write(&original, b"hello world").unwrap();
        let size = std::fs::metadata(&original).unwrap().len();
        let q = dir.join("q").join("victim.tmp");

        let action = ReversibleAction::FileQuarantine {
            original: original.to_string_lossy().into_owned(),
            quarantine: q.to_string_lossy().into_owned(),
            size,
        };

        let dbfile = dir.join("fq.db");
        let db = tk_storage::open(&dbfile.to_string_lossy()).await.unwrap();
        let svc = ProtectionService::new(db);

        // snapshot → apply: arquivo vai para a quarentena; freed == tamanho
        let snap_id = svc.create_snapshot("cleanup", std::slice::from_ref(&action), None).await.unwrap();
        let freed = apply_actions(std::slice::from_ref(&action)).unwrap();
        assert_eq!(freed, size, "bytes liberados == tamanho do arquivo");
        assert!(!original.exists(), "original foi movido");
        assert!(q.exists(), "arquivo está na quarentena");

        // rollback (restore_entry "file") → arquivo volta ao local original, intacto
        let outcome = svc.rollback(snap_id).await.unwrap();
        assert!(outcome.ok, "rollback OK");
        assert!(original.exists(), "arquivo restaurado ao local original");
        assert_eq!(std::fs::read(&original).unwrap(), b"hello world", "conteúdo intacto");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── RegistryHkcu (string): aplicar→verificar→reverter→verificar ──
    #[cfg(windows)]
    #[test]
    fn registry_hkcu_string_cycle() {
        let sub = format!("Software\\TkSpeedTest\\rbstr_{}", std::process::id());
        let name = "Val";
        let _ = registry::delete_value(&sub, name);
        let old = registry::read_string(&sub, name).unwrap(); // None

        let apply = vec![ReversibleAction::RegistryHkcu {
            subkey: sub.clone(),
            name: name.into(),
            old: old.clone(),
            new: Some("123".into()),
        }];
        apply_actions(&apply).unwrap();
        assert!(verify_actions(&apply).unwrap(), "novo valor efetivado");
        assert_eq!(registry::read_string(&sub, name).unwrap(), Some("123".to_string()));

        // reverter = aplicar a ação inversa (new = estado original)
        let revert = vec![ReversibleAction::RegistryHkcu {
            subkey: sub.clone(),
            name: name.into(),
            old: Some("123".into()),
            new: old.clone(),
        }];
        apply_actions(&revert).unwrap();
        assert_eq!(registry::read_string(&sub, name).unwrap(), old, "estado original restaurado");

        let _ = registry::delete_value(&sub, name);
    }

    // ── RegistryHkcuDword: aplicar→verificar→reverter→verificar (old = Some) ──
    #[cfg(windows)]
    #[test]
    fn registry_hkcu_dword_cycle() {
        let sub = format!("Software\\TkSpeedTest\\rbdw_{}", std::process::id());
        let name = "Num";
        registry::write_u32(&sub, name, 1).unwrap(); // estado inicial conhecido
        let old = registry::read_u32(&sub, name).unwrap(); // Some(1)

        let apply = vec![ReversibleAction::RegistryHkcuDword {
            subkey: sub.clone(),
            name: name.into(),
            old,
            new: Some(0),
        }];
        apply_actions(&apply).unwrap();
        assert!(verify_actions(&apply).unwrap());
        assert_eq!(registry::read_u32(&sub, name).unwrap(), Some(0));

        let revert = vec![ReversibleAction::RegistryHkcuDword {
            subkey: sub.clone(),
            name: name.into(),
            old: Some(0),
            new: old,
        }];
        apply_actions(&revert).unwrap();
        assert_eq!(registry::read_u32(&sub, name).unwrap(), old, "DWORD original restaurado");

        let _ = registry::delete_value(&sub, name);
    }

    // ── PowerPlan: aplicar→verificar para o MESMO plano (no-op seguro) ──
    #[cfg(windows)]
    #[test]
    fn power_plan_set_to_self_is_safe() {
        let current = match power::get_active_scheme() {
            Ok(g) => g,
            Err(_) => return, // sem powercfg → não falha o suite
        };
        let action = vec![ReversibleAction::PowerPlan {
            old_guid: current.clone(),
            new_guid: current.clone(),
        }];
        apply_actions(&action).unwrap();
        assert!(verify_actions(&action).unwrap(), "plano ativo confirmado");
        assert_eq!(power::get_active_scheme().unwrap(), current, "plano inalterado");
    }

    // ── A2.4: ciclo COMPLETO com DB — capture→snapshot→apply→verify→rollback→verify ──
    #[cfg(windows)]
    #[tokio::test]
    async fn integration_full_cycle_with_db() {
        let dbfile = std::env::temp_dir().join(format!("tkspeed_it_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dbfile);
        let db = tk_storage::open(&dbfile.to_string_lossy()).await.unwrap();
        let svc = ProtectionService::new(db);

        let sub = format!("Software\\TkSpeedTest\\it_{}", std::process::id());
        let name = "Cycle";
        let _ = registry::delete_value(&sub, name);
        let original = registry::read_string(&sub, name).unwrap(); // capture (None)

        let action = ReversibleAction::RegistryHkcu {
            subkey: sub.clone(),
            name: name.into(),
            old: original.clone(),
            new: Some("999".into()),
        };

        // snapshot (estado original) → apply → verify
        let snap_id = svc.create_snapshot("integration", std::slice::from_ref(&action), None).await.unwrap();
        apply_actions(std::slice::from_ref(&action)).unwrap();
        assert_eq!(registry::read_string(&sub, name).unwrap(), Some("999".to_string()), "valor aplicado");

        // rollback (restore_entry + checagem de integridade) → verify final
        let outcome = svc.rollback(snap_id).await.unwrap();
        assert!(outcome.ok, "rollback OK");
        assert_eq!(outcome.restored, 1, "1 entrada restaurada");
        assert_eq!(registry::read_string(&sub, name).unwrap(), original, "estado ORIGINAL restaurado");

        let _ = registry::delete_value(&sub, name);
        let _ = std::fs::remove_file(&dbfile);
    }

    // ── A2.4: o autoteste de produção (pilot MenuShowDelay) passa de ponta a ponta ──
    #[cfg(windows)]
    #[tokio::test]
    async fn integration_production_selftest_passes() {
        let dbfile = std::env::temp_dir().join(format!("tkspeed_st_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dbfile);
        let db = tk_storage::open(&dbfile.to_string_lossy()).await.unwrap();
        let svc = ProtectionService::new(db);

        let report = svc.selftest().await.unwrap();
        assert!(report.passed, "autoteste capture→apply→verify→rollback→verify deve passar");
        assert!(report.steps.iter().all(|s| s.ok), "todos os passos OK");

        let _ = std::fs::remove_file(&dbfile);
    }
}
