//! Repositório de perfis — acesso a profile_definitions, profile_state e profile_activations.

use sqlx::Row;
use tk_storage::{now_ms, Db};

use super::model::{
    ActivationRow, ActivationSummary, CompositionEntry, ProfileDefinition, ProfileStateRow,
};

type Result<T> = std::result::Result<T, tk_storage::StorageError>;

pub struct ProfileRepo {
    db: Db,
}

impl ProfileRepo {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    // ── profile_definitions ──────────────────────────────────────────────────

    /// Insere ou atualiza uma definição de perfil.
    pub async fn upsert_definition(&self, p: &ProfileDefinition) -> Result<()> {
        let compositions_json = serde_json::to_string(&p.compositions)?;
        let now = now_ms();
        sqlx::query(
            "INSERT INTO profile_definitions \
             (id, name, description, icon, is_custom, compositions_json, suite_id, \
              requires_fps, bundle_version, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, \
               description = excluded.description, \
               icon = excluded.icon, \
               compositions_json = excluded.compositions_json, \
               suite_id = excluded.suite_id, \
               requires_fps = excluded.requires_fps, \
               bundle_version = excluded.bundle_version, \
               updated_at = excluded.updated_at",
        )
        .bind(&p.id)
        .bind(&p.name)
        .bind(&p.description)
        .bind(&p.icon)
        .bind(p.is_custom as i64)
        .bind(&compositions_json)
        .bind(&p.suite_id)
        .bind(p.requires_fps as i64)
        .bind(p.bundle_version as i64)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn get_definition(&self, id: &str) -> Result<Option<ProfileDefinition>> {
        let row = sqlx::query(
            "SELECT id, name, description, icon, is_custom, compositions_json, \
             suite_id, requires_fps, bundle_version \
             FROM profile_definitions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row.as_ref().map(parse_definition))
    }

    pub async fn list_definitions(&self) -> Result<Vec<ProfileDefinition>> {
        let rows = sqlx::query(
            "SELECT id, name, description, icon, is_custom, compositions_json, \
             suite_id, requires_fps, bundle_version \
             FROM profile_definitions ORDER BY is_custom ASC, id ASC",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows.iter().map(parse_definition).collect())
    }

    // ── profile_state ────────────────────────────────────────────────────────

    pub async fn get_state(&self, user_context: &str) -> Result<ProfileStateRow> {
        let row = sqlx::query(
            "SELECT user_context, profile_id, activated_at, snapshot_id, pending_reboot \
             FROM profile_state WHERE user_context = ?",
        )
        .bind(user_context)
        .fetch_one(&self.db)
        .await?;
        Ok(ProfileStateRow {
            user_context: row.get("user_context"),
            profile_id: row.get("profile_id"),
            activated_at: row.get("activated_at"),
            snapshot_id: row.get("snapshot_id"),
            pending_reboot: row.get::<i64, _>("pending_reboot") != 0,
        })
    }

    pub async fn set_state(
        &self,
        user_context: &str,
        profile_id: Option<&str>,
        snapshot_id: Option<i64>,
        pending_reboot: bool,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE profile_state SET \
             profile_id = ?, activated_at = ?, snapshot_id = ?, pending_reboot = ? \
             WHERE user_context = ?",
        )
        .bind(profile_id)
        .bind(profile_id.map(|_| now_ms()))
        .bind(snapshot_id)
        .bind(pending_reboot as i64)
        .bind(user_context)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn clear_state(&self, user_context: &str) -> Result<()> {
        sqlx::query(
            "UPDATE profile_state SET \
             profile_id = NULL, activated_at = NULL, snapshot_id = NULL, pending_reboot = 0 \
             WHERE user_context = ?",
        )
        .bind(user_context)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    // ── profile_activations ──────────────────────────────────────────────────

    pub async fn record_activation(&self, act: &ActivationRow) -> Result<i64> {
        let id = sqlx::query(
            "INSERT INTO profile_activations \
             (ts, user_context, profile_id, from_profile_id, snapshot_id, \
              machine_fingerprint, pending_reboot, source) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(act.ts)
        .bind(&act.user_context)
        .bind(&act.profile_id)
        .bind(&act.from_profile_id)
        .bind(act.snapshot_id)
        .bind(&act.machine_fingerprint)
        .bind(act.pending_reboot as i64)
        .bind(&act.source)
        .execute(&self.db)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    pub async fn list_activations(&self, limit: i64) -> Result<Vec<ActivationSummary>> {
        let rows = sqlx::query(
            "SELECT id, ts, profile_id, from_profile_id, pending_reboot, source \
             FROM profile_activations ORDER BY ts DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .iter()
            .map(|r| ActivationSummary {
                id: r.get("id"),
                ts: r.get("ts"),
                profile_id: r.get("profile_id"),
                from_profile_id: r.get("from_profile_id"),
                pending_reboot: r.get::<i64, _>("pending_reboot") != 0,
                source: r.get("source"),
            })
            .collect())
    }

    /// Atualiza before_session_id, after_session_id e evidence_json em profile_activations.
    pub async fn update_activation_sessions(
        &self,
        id: i64,
        before_session_id: Option<i64>,
        after_session_id: Option<i64>,
        evidence_json: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE profile_activations \
             SET before_session_id = ?, after_session_id = ?, evidence_json = ? \
             WHERE id = ?",
        )
        .bind(before_session_id)
        .bind(after_session_id)
        .bind(evidence_json)
        .bind(id)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

fn parse_definition(r: &sqlx::sqlite::SqliteRow) -> ProfileDefinition {
    let json: String = r.get("compositions_json");
    let compositions: Vec<CompositionEntry> = serde_json::from_str(&json).unwrap_or_default();
    ProfileDefinition {
        id: r.get("id"),
        name: r.get("name"),
        description: r.get("description"),
        icon: r.get("icon"),
        is_custom: r.get::<i64, _>("is_custom") != 0,
        compositions,
        suite_id: r.get("suite_id"),
        requires_fps: r.get::<i64, _>("requires_fps") != 0,
        bundle_version: r.get::<i64, _>("bundle_version") as u32,
    }
}
