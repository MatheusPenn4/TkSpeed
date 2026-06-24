//! Profile Engine — ativa, desativa e previews perfis.
//!
//! Pipeline de ativação (V4.2-B):
//!   1. Encontrar perfil (bundled ou DB)
//!   2. Persistir definição → satisfaz FK de profile_state
//!   3. Construir plano de execução via executor (Executable | Unsupported)
//!   4. Criar snapshot com as ações executáveis
//!   5. apply_actions + verify_actions → rollback em falha
//!   6. Registrar profile_activation
//!   7. Atualizar profile_state

use tk_rollback::{apply_actions, verify_actions, ProtectionService, ReversibleAction};
use tk_storage::{now_ms, session_source, Db};

use crate::configs::{ConfigRegistry, ConfigRisk};
use crate::machine;

use super::bundled;
use super::evidence::ProfileEvidenceRepo;
use super::executor::{self, ConfigAction};
use super::model::{
    ActivationResult, ActivationRow, ProfileConfigPreview, ProfileDefinition, ProfilePreview,
};
use super::repo::ProfileRepo;

pub struct ProfileEngine {
    db: Db,
}

impl ProfileEngine {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn protection(&self) -> ProtectionService {
        ProtectionService::new(self.db.clone())
    }
    fn repo(&self) -> ProfileRepo {
        ProfileRepo::new(self.db.clone())
    }
    pub fn evidence_repo(&self) -> ProfileEvidenceRepo {
        ProfileEvidenceRepo::new(self.db.clone())
    }

    /// Retorna todos os perfis disponíveis (bundled).
    pub fn all_profiles(&self) -> Vec<ProfileDefinition> {
        bundled::all()
    }

    /// Preview de um perfil: configs envolvidas, riscos, reboot, relevância.
    /// Somente leitura — não aplica nada.
    pub fn preview(&self, profile_id: &str) -> Result<ProfilePreview, String> {
        let profile = self.get_profile(profile_id)?;
        let registry = ConfigRegistry::new();

        let mut configs = Vec::new();
        let mut requires_reboot = false;
        let mut max_risk = ConfigRisk::Safe;
        let mut relevance_set: std::collections::BTreeSet<String> = Default::default();

        for comp in &profile.compositions {
            if let Some(cfg) = registry.find(&comp.config_id) {
                let meta = cfg.meta();
                requires_reboot |= meta.requires_reboot;
                if meta.risk > max_risk {
                    max_risk = meta.risk.clone();
                }
                for r in meta.benchmark_relevance {
                    relevance_set.insert(r.to_string());
                }
                configs.push(ProfileConfigPreview {
                    config_id: comp.config_id.clone(),
                    name: meta.name.to_string(),
                    risk: risk_str(&meta.risk),
                    reversible: meta.reversible,
                    requires_reboot: meta.requires_reboot,
                    benchmark_relevance: meta.benchmark_relevance.iter().map(|s| s.to_string()).collect(),
                });
            }
        }

        Ok(ProfilePreview {
            profile_id: profile.id,
            name: profile.name,
            description: profile.description,
            configs,
            requires_reboot,
            max_risk: risk_str(&max_risk),
            benchmark_relevance: relevance_set.into_iter().collect(),
        })
    }

    /// Ativa um perfil: constrói ações reais via executor, snapshot, aplica, registra.
    pub async fn activate(&self, profile_id: &str) -> Result<ActivationResult, String> {
        // 1. Encontrar perfil (falha aqui não toca o banco)
        let profile = self.get_profile(profile_id)?;

        // 2. Upsert da definição (necessário para FK de profile_state)
        self.repo()
            .upsert_definition(&profile)
            .await
            .map_err(|e| e.to_string())?;

        // 3. Construir plano de execução
        let fp = machine::fingerprint();
        let registry = ConfigRegistry::new();
        let mut actions: Vec<ReversibleAction> = Vec::new();
        let mut applied: Vec<String> = Vec::new();
        let mut skipped: Vec<String> = Vec::new();
        let mut pending_reboot = false;

        for comp in &profile.compositions {
            match executor::build_action(&comp.config_id)
                .map_err(|e| format!("Falha ao preparar {}: {e}", comp.config_id))?
            {
                ConfigAction::Executable(action) => {
                    actions.push(action);
                    applied.push(comp.config_id.clone());
                    // pending_reboot reflete apenas o que foi REALMENTE aplicado —
                    // configs skipped (ex.: HAGS sem admin) não marcam reboot.
                    if let Some(cfg) = registry.find(&comp.config_id) {
                        if cfg.meta().requires_reboot {
                            pending_reboot = true;
                        }
                    }
                }
                ConfigAction::Unsupported { .. } => {
                    skipped.push(comp.config_id.clone());
                }
            }
        }

        // 4. Snapshot com as ações reais
        let snap_id = self
            .protection()
            .create_snapshot(&format!("profile:{profile_id}"), &actions, Some(&fp))
            .await
            .map_err(|e| e.to_string())?;

        // 5. Aplicar e verificar
        if !actions.is_empty() {
            if let Err(e) = apply_actions(&actions) {
                let _ = self.protection().rollback(snap_id).await;
                return Err(format!("Falha ao aplicar ações do perfil: {e}"));
            }
            if !verify_actions(&actions).unwrap_or(false) {
                let _ = self.protection().rollback(snap_id).await;
                return Err("Verificação falhou após aplicação do perfil.".into());
            }
        }

        // 6. Registrar activation
        let current_state = self
            .repo()
            .get_state("default")
            .await
            .map_err(|e| e.to_string())?;

        let act_id = self
            .repo()
            .record_activation(&ActivationRow {
                ts: now_ms(),
                user_context: "default".into(),
                profile_id: profile_id.into(),
                from_profile_id: current_state.profile_id.clone(),
                snapshot_id: snap_id,
                machine_fingerprint: Some(fp.clone()),
                pending_reboot,
                source: session_source::PROFILE_ACTIVATION.into(),
            })
            .await
            .map_err(|e| e.to_string())?;

        // 7. Atualizar profile_state
        self.repo()
            .set_state("default", Some(profile_id), Some(snap_id), pending_reboot)
            .await
            .map_err(|e| e.to_string())?;

        let msg = if pending_reboot {
            "Perfil ativo. Reinicialização necessária para algumas configurações.".into()
        } else if applied.is_empty() {
            "Perfil registrado. Implementações pendentes para esta versão.".into()
        } else {
            "Perfil ativo.".into()
        };

        Ok(ActivationResult {
            activation_id: act_id,
            profile_id: profile_id.into(),
            snapshot_id: snap_id,
            applied_configs: applied,
            skipped_configs: skipped,
            pending_reboot,
            message: msg,
        })
    }

    /// Desativa o perfil ativo: reverte snapshot e limpa profile_state.
    pub async fn deactivate(&self) -> Result<(), String> {
        let state = self
            .repo()
            .get_state("default")
            .await
            .map_err(|e| e.to_string())?;

        if state.profile_id.is_none() {
            return Err("Nenhum perfil ativo no momento.".into());
        }

        if let Some(snap_id) = state.snapshot_id {
            self.protection()
                .rollback(snap_id)
                .await
                .map_err(|e| e.to_string())?;
        }

        self.repo()
            .clear_state("default")
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_profile(&self, profile_id: &str) -> Result<ProfileDefinition, String> {
        bundled::all()
            .into_iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| format!("Perfil '{profile_id}' não encontrado."))
    }
}

fn risk_str(r: &ConfigRisk) -> String {
    match r {
        ConfigRisk::Safe => "safe",
        ConfigRisk::Moderate => "moderate",
        ConfigRisk::Advanced => "advanced",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bundled;

    async fn make_engine() -> (ProfileEngine, Db, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.db");
        let db = tk_storage::open(path.to_str().unwrap()).await.unwrap();
        let engine = ProfileEngine::new(db.clone());
        (engine, db, dir)
    }

    // ── Bundled profiles ─────────────────────────────────────────────────────

    #[test]
    fn all_bundled_profiles_load() {
        let profiles = bundled::all();
        assert_eq!(profiles.len(), 4);
        let ids: Vec<&str> = profiles.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"competitive"));
        assert!(ids.contains(&"balanced"));
        assert!(ids.contains(&"streaming"));
        assert!(ids.contains(&"power_saver"));
    }

    #[test]
    fn bundled_profiles_have_compositions() {
        for p in bundled::all() {
            assert!(!p.compositions.is_empty(), "{} deve ter pelo menos 1 config", p.id);
        }
    }

    #[test]
    fn competitive_requires_fps() {
        assert!(bundled::competitive().requires_fps);
    }

    #[test]
    fn non_competitive_profiles_do_not_require_fps() {
        assert!(!bundled::balanced().requires_fps);
        assert!(!bundled::streaming().requires_fps);
        assert!(!bundled::power_saver().requires_fps);
    }

    // ── Preview ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn preview_competitive_has_correct_configs() {
        let (engine, _, _dir) = make_engine().await;
        let preview = engine.preview("competitive").unwrap();
        let ids: Vec<&str> = preview.configs.iter().map(|c| c.config_id.as_str()).collect();
        // P2.4: HAGS + plano de alto desempenho; timer_resolution removido (Unsupported).
        assert!(ids.contains(&"gpu_hardware_scheduling"));
        assert!(ids.contains(&"power_plan_high_performance"));
        assert!(
            !ids.contains(&"timer_resolution"),
            "timer_resolution (Unsupported) não deve mais estar em Competitive"
        );
    }

    #[tokio::test]
    async fn preview_competitive_requires_reboot() {
        let (engine, _, _dir) = make_engine().await;
        let preview = engine.preview("competitive").unwrap();
        assert!(preview.requires_reboot, "competitive tem gpu_hardware_scheduling que exige reboot");
    }

    #[tokio::test]
    async fn preview_streaming_no_reboot() {
        let (engine, _, _dir) = make_engine().await;
        let preview = engine.preview("streaming").unwrap();
        assert!(!preview.requires_reboot, "streaming não tem configs que exigem reboot");
    }

    #[tokio::test]
    async fn preview_power_saver_no_reboot() {
        let (engine, _, _dir) = make_engine().await;
        let preview = engine.preview("power_saver").unwrap();
        assert!(!preview.requires_reboot);
    }

    #[tokio::test]
    async fn preview_has_benchmark_relevance() {
        let (engine, _, _dir) = make_engine().await;
        let preview = engine.preview("competitive").unwrap();
        assert!(!preview.benchmark_relevance.is_empty());
    }

    #[tokio::test]
    async fn preview_unknown_profile_errors() {
        let (engine, _, _dir) = make_engine().await;
        assert!(engine.preview("nonexistent").is_err());
    }

    // ── Activate — estado e registros ────────────────────────────────────────

    #[tokio::test]
    async fn activate_updates_profile_state() {
        let (engine, db, _dir) = make_engine().await;
        let result = engine.activate("streaming").await.unwrap();
        assert_eq!(result.profile_id, "streaming");

        let state = ProfileRepo::new(db).get_state("default").await.unwrap();
        assert_eq!(state.profile_id.as_deref(), Some("streaming"));
        assert!(state.snapshot_id.is_some());

        engine.deactivate().await.ok(); // restaura plano de energia
    }

    #[tokio::test]
    async fn activate_records_activation_entry() {
        let (engine, db, _dir) = make_engine().await;
        engine.activate("balanced").await.unwrap();

        let acts = ProfileRepo::new(db).list_activations(10).await.unwrap();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts[0].profile_id, "balanced");
        assert_eq!(acts[0].source, session_source::PROFILE_ACTIVATION);

        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_stores_machine_fingerprint_in_activation() {
        let (engine, db, _dir) = make_engine().await;
        engine.activate("power_saver").await.unwrap();

        let fp_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM profile_activations WHERE machine_fingerprint IS NOT NULL",
        )
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(fp_count, 1);

        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_sets_from_profile_id_when_switching() {
        let (engine, db, _dir) = make_engine().await;
        engine.activate("streaming").await.unwrap();
        engine.activate("balanced").await.unwrap();

        let acts = ProfileRepo::new(db).list_activations(10).await.unwrap();
        let balanced_act = &acts[0]; // mais recente primeiro
        assert_eq!(balanced_act.from_profile_id.as_deref(), Some("streaming"));

        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_creates_snapshot() {
        let (engine, db, _dir) = make_engine().await;
        engine.activate("power_saver").await.unwrap();

        let snap_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM snapshots WHERE reason LIKE 'profile:%'",
        )
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(snap_count, 1);

        engine.deactivate().await.ok();
    }

    // ── Activate — executor: applied vs skipped ──────────────────────────────
    // Balanced (381b4222-...) é garantido em toda instalação Windows.
    // Os testes usam `streaming` (power_plan_balanced + memory_flush) como base.
    // Para outros planos, usamos scheme_exists como guard.

    #[tokio::test]
    async fn activate_streaming_applies_power_plan_balanced() {
        // streaming: power_plan_balanced (sempre disponível). memory_standby_flush
        // foi removido — não deve mais aparecer em applied nem em skipped.
        let (engine, _, _dir) = make_engine().await;
        let result = engine.activate("streaming").await.unwrap();
        assert!(
            result.applied_configs.contains(&"power_plan_balanced".to_string()),
            "power_plan_balanced deve estar em applied (sempre disponível)"
        );
        assert!(
            !result.applied_configs.contains(&"memory_standby_flush".to_string())
                && !result.skipped_configs.contains(&"memory_standby_flush".to_string()),
            "memory_standby_flush não deve mais ser config de perfil"
        );
        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_balanced_applies_power_plan() {
        // balanced: gpu_hardware_scheduling (HKLM — depende de elevação) + power_plan_balanced.
        // power_plan_balanced é sempre aplicável; o destino de HAGS depende de admin.
        let (engine, _, _dir) = make_engine().await;
        let result = engine.activate("balanced").await.unwrap();
        assert!(
            result.applied_configs.contains(&"power_plan_balanced".to_string()),
            "power_plan_balanced deve ser aplicado"
        );
        if tk_platform_win::elevation::is_elevated() {
            assert!(
                result.applied_configs.contains(&"gpu_hardware_scheduling".to_string()),
                "elevado: HAGS deve ser aplicado"
            );
        } else {
            assert!(
                result.skipped_configs.contains(&"gpu_hardware_scheduling".to_string()),
                "sem admin: HAGS deve ser skipped"
            );
        }
        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_competitive_handles_hags_by_elevation() {
        // P2.4: competitive = gpu_hardware_scheduling + power_plan_high_performance
        // (timer_resolution removido). HAGS é HKLM → applied só com admin, senão skipped.
        let (engine, _, _dir) = make_engine().await;
        let result = engine.activate("competitive").await.unwrap();
        if tk_platform_win::elevation::is_elevated() {
            assert!(
                result.applied_configs.contains(&"gpu_hardware_scheduling".to_string()),
                "elevado: HAGS deve ser aplicado"
            );
        } else {
            assert!(
                result.skipped_configs.contains(&"gpu_hardware_scheduling".to_string()),
                "sem admin: HAGS deve ser skipped"
            );
        }
        assert!(
            !result.applied_configs.contains(&"timer_resolution".to_string())
                && !result.skipped_configs.contains(&"timer_resolution".to_string()),
            "timer_resolution não deve mais fazer parte de Competitive"
        );
        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_power_saver_succeeds_regardless_of_scheme_availability() {
        // power_saver pode não estar listado em algumas instalações Windows 11
        let (engine, _, _dir) = make_engine().await;
        let result = engine.activate("power_saver").await;
        assert!(result.is_ok(), "deve ativar com sucesso (Unsupported é válido para plan ausente)");
        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_power_saver_applied_when_scheme_exists() {
        use tk_platform_win::power;
        use super::super::executor::GUID_POWER_SAVER;
        if !power::scheme_exists(GUID_POWER_SAVER) { return; } // plano não disponível → skip

        let (engine, _, _dir) = make_engine().await;
        let result = engine.activate("power_saver").await.unwrap();
        assert!(
            result.applied_configs.contains(&"power_plan_power_saver".to_string()),
            "power_plan_power_saver deve ser aplicado quando esquema existe"
        );
        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn activate_competitive_pending_reboot_matches_elevation() {
        let (engine, _, _dir) = make_engine().await;
        let result = engine.activate("competitive").await.unwrap();
        // competitive tem gpu_hardware_scheduling (requires_reboot=true), mas HAGS só
        // é APLICADO com admin. pending_reboot reflete o que foi de fato aplicado.
        if tk_platform_win::elevation::is_elevated() {
            assert!(result.pending_reboot, "elevado: HAGS aplicado → pending_reboot");
        } else {
            assert!(!result.pending_reboot, "sem admin: HAGS skipped → sem pending_reboot");
        }
        engine.deactivate().await.ok();
    }

    // ── Deactivate ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn deactivate_clears_profile_state() {
        let (engine, db, _dir) = make_engine().await;
        engine.activate("streaming").await.unwrap();
        engine.deactivate().await.unwrap();

        let state = ProfileRepo::new(db).get_state("default").await.unwrap();
        assert!(state.profile_id.is_none());
        assert!(state.snapshot_id.is_none());
    }

    #[tokio::test]
    async fn deactivate_without_active_profile_returns_error() {
        let (engine, _, _dir) = make_engine().await;
        let result = engine.deactivate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn deactivate_marks_snapshot_as_restored() {
        let (engine, db, _dir) = make_engine().await;
        let act = engine.activate("power_saver").await.unwrap();
        engine.deactivate().await.unwrap();

        let status: String =
            sqlx::query_scalar("SELECT status FROM snapshots WHERE id = ?")
                .bind(act.snapshot_id)
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(status, "restored");
    }

    #[tokio::test]
    async fn activate_streaming_snapshot_has_power_plan_entry() {
        // streaming: memory_flush (unsupported) + power_plan_balanced (disponível)
        // → snapshot deve ter exatamente 1 entrada (power_plan_balanced)
        let (engine, db, _dir) = make_engine().await;
        let act = engine.activate("streaming").await.unwrap();

        let entry_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM snapshot_entries WHERE snapshot_id = ?")
                .bind(act.snapshot_id)
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(entry_count, 1, "streaming tem 1 ação executável (power_plan_balanced)");

        engine.deactivate().await.ok();
    }

    // ── Rollback em falha ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn activate_unknown_profile_does_not_modify_state() {
        let (engine, db, _dir) = make_engine().await;
        let before = ProfileRepo::new(db.clone()).get_state("default").await.unwrap();

        let result = engine.activate("nonexistent_profile").await;
        assert!(result.is_err(), "deve retornar erro para perfil desconhecido");

        let after = ProfileRepo::new(db).get_state("default").await.unwrap();
        assert_eq!(
            before.profile_id, after.profile_id,
            "estado não deve ser alterado em caso de falha"
        );
    }

    #[tokio::test]
    async fn activate_unknown_profile_creates_no_snapshot() {
        let (engine, db, _dir) = make_engine().await;
        let _ = engine.activate("nonexistent").await;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snapshots")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 0, "falha antes do snapshot não deve criar snapshots");
    }

    // ── Evidence repo integração ──────────────────────────────────────────────

    #[tokio::test]
    async fn evidence_repo_accessible_from_engine() {
        let (engine, _, _dir) = make_engine().await;
        engine.activate("power_saver").await.unwrap();

        // record_success simulando benchmark Keep externo
        let fp = machine::fingerprint();
        engine
            .evidence_repo()
            .record_success("power_saver", &fp, 4.2)
            .await
            .unwrap();

        let rec = engine
            .evidence_repo()
            .get("power_saver", &fp)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(rec.successful_executions, 1);
        assert!((rec.average_gain - 4.2).abs() < 0.001);

        engine.deactivate().await.ok();
    }

    #[tokio::test]
    async fn evidence_repo_record_failure_from_engine() {
        let (engine, _, _dir) = make_engine().await;
        let fp = machine::fingerprint();

        engine
            .evidence_repo()
            .record_failure("streaming", &fp)
            .await
            .unwrap();

        let rec = engine
            .evidence_repo()
            .get("streaming", &fp)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(rec.executions, 1);
        assert_eq!(rec.successful_executions, 0);
    }
}
