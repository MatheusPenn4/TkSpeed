#[cfg(test)]
mod migration_tests {
    use crate::open;

    #[tokio::test]
    async fn test_v4_migration_runs_clean() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = open(db_path.to_str().unwrap()).await
            .expect("migration deve rodar sem erros");

        // Confirmar que as novas colunas existem em benchmark_sessions
        sqlx::query(
            "INSERT INTO benchmark_sessions \
             (ts, kind, suite_version, duration_ms, runs, confidence, contaminated, \
              source, machine_fingerprint, rendering_device_name) \
             VALUES (1, 'test', 'v1', 100, 1, 80, 0, \
                     'profile_activation', 'fp_test', 'RTX 4080')"
        )
        .execute(&pool)
        .await
        .expect("INSERT com colunas V4 em benchmark_sessions deve funcionar");

        // Confirmar source padrão
        let source: String = sqlx::query_scalar(
            "SELECT source FROM benchmark_sessions WHERE kind = 'test'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(source, "profile_activation");

        // Confirmar que snapshots tem machine_fingerprint
        sqlx::query(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status, machine_fingerprint) \
             VALUES (1, 'test', 'abc', 'active', 'fp_snap')"
        )
        .execute(&pool)
        .await
        .expect("INSERT com machine_fingerprint em snapshots deve funcionar");

        // Confirmar profile_definitions
        sqlx::query(
            "INSERT INTO profile_definitions \
             (id, name, suite_id, bundle_version, created_at, updated_at) \
             VALUES ('competitive', 'Competitivo', 'fps', 1, 1, 1)"
        )
        .execute(&pool)
        .await
        .expect("INSERT em profile_definitions deve funcionar");

        // Confirmar profile_state com user_context
        let ctx: String = sqlx::query_scalar(
            "SELECT user_context FROM profile_state"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(ctx, "default");

        // Confirmar profile_activations
        let snap_id: i64 = sqlx::query_scalar(
            "SELECT id FROM snapshots WHERE reason = 'test'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO profile_activations \
             (ts, profile_id, snapshot_id, machine_fingerprint, rendering_device_name) \
             VALUES (1, 'competitive', ?, 'fp_test', 'RTX 4080')"
        )
        .bind(snap_id)
        .execute(&pool)
        .await
        .expect("INSERT em profile_activations deve funcionar");

        pool.close().await;
    }
}

#[cfg(test)]
mod session_source_tests {
    use crate::{open, session_source, PerfRepo, OptRepo, SnapshotRepo};
    use tk_contracts::{BenchmarkResult, OptDecision, OptimizationRunInfo, PerfComparison};

    fn dummy_result(kind: &str) -> BenchmarkResult {
        BenchmarkResult {
            kind: kind.into(),
            suite_version: "test-1.0".into(),
            duration_ms: 100,
            runs: 3,
            metrics: vec![],
            confidence: 80,
            stable: true,
            contaminated: false,
            temp_start_c: None,
            temp_end_c: None,
        }
    }

    fn dummy_run_info(opt_id: &str) -> OptimizationRunInfo {
        OptimizationRunInfo {
            id: 0,
            ts: 1_000_000,
            optimization_id: opt_id.into(),
            name: opt_id.into(),
            status: "kept".into(),
            decision: OptDecision::Keep,
            confidence: 90,
            before_session: None,
            after_session: None,
            comparison: None,
            message: "test".into(),
        }
    }

    async fn make_db() -> (crate::Db, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = open(path.to_str().unwrap()).await.unwrap();
        (db, dir)
    }

    // ── source padrão da coluna é 'manual' para benchmark_sessions ──
    #[tokio::test]
    async fn benchmark_sessions_default_source_is_manual() {
        let (db, _dir) = make_db().await;
        sqlx::query(
            "INSERT INTO benchmark_sessions \
             (ts, kind, suite_version, duration_ms, runs, confidence, contaminated) \
             VALUES (1, 'cpu', 'v1', 100, 3, 80, 0)",
        )
        .execute(&db)
        .await
        .unwrap();

        let src: String = sqlx::query_scalar("SELECT source FROM benchmark_sessions WHERE kind = 'cpu'")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(src, session_source::MANUAL, "DEFAULT deve ser 'manual'");
    }

    // ── source padrão da coluna é 'optimization_catalog' para optimization_runs ──
    #[tokio::test]
    async fn optimization_runs_default_source_is_optimization_catalog() {
        let (db, _dir) = make_db().await;
        let snap_id: i64 = sqlx::query_scalar(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status) \
             VALUES (1, 'r', 'h', 'active') RETURNING id",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        let info = dummy_run_info("power_plan");
        let json = serde_json::to_string(&info).unwrap();
        sqlx::query(
            "INSERT INTO optimization_runs \
             (ts, optimization_id, snapshot_id, status, decision, confidence, evidence_json) \
             VALUES (1, 'power_plan', ?, 'kept', 'keep', 90, ?)",
        )
        .bind(snap_id)
        .bind(&json)
        .execute(&db)
        .await
        .unwrap();

        let src: String =
            sqlx::query_scalar("SELECT source FROM optimization_runs WHERE optimization_id = 'power_plan'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(src, session_source::OPTIMIZATION_CATALOG, "DEFAULT deve ser 'optimization_catalog'");
    }

    // ── source padrão de profile_activations é 'profile_activation' ──
    #[tokio::test]
    async fn profile_activations_default_source_is_profile_activation() {
        let (db, _dir) = make_db().await;
        let snap_id: i64 = sqlx::query_scalar(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status) \
             VALUES (1, 'r', 'h', 'active') RETURNING id",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO profile_activations (ts, profile_id, snapshot_id) \
             VALUES (1, 'competitive', ?)",
        )
        .bind(snap_id)
        .execute(&db)
        .await
        .unwrap();

        let src: String =
            sqlx::query_scalar("SELECT source FROM profile_activations WHERE profile_id = 'competitive'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(src, session_source::PROFILE_ACTIVATION, "DEFAULT deve ser 'profile_activation'");
    }

    // ── PerfRepo::save_session persiste source corretamente ──
    #[tokio::test]
    async fn save_session_persists_source() {
        let (db, _dir) = make_db().await;
        let repo = PerfRepo::new(db.clone());

        let result = dummy_result("ram");
        repo.save_session("bench-manual", &result, None, session_source::MANUAL)
            .await
            .unwrap();

        let src: String =
            sqlx::query_scalar("SELECT source FROM benchmark_sessions WHERE target = 'bench-manual'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(src, session_source::MANUAL);
    }

    // ── Cada source é persistido sem confusão ──
    #[tokio::test]
    async fn save_session_all_sources_persist_correctly() {
        let (db, _dir) = make_db().await;
        let repo = PerfRepo::new(db.clone());

        let sources = [
            session_source::MANUAL,
            session_source::OPTIMIZATION_CATALOG,
            session_source::PROFILE_ACTIVATION,
            session_source::AUTOMATIC,
        ];
        for src in sources {
            let label = format!("bench-{src}");
            repo.save_session(&label, &dummy_result("cpu"), None, src)
                .await
                .unwrap();
            let stored: String = sqlx::query_scalar(
                "SELECT source FROM benchmark_sessions WHERE target = ?",
            )
            .bind(&label)
            .fetch_one(&db)
            .await
            .unwrap();
            assert_eq!(stored, src);
        }
    }

    // ── sessions_by_source filtra corretamente ──
    #[tokio::test]
    async fn sessions_by_source_filters_correctly() {
        let (db, _dir) = make_db().await;
        let repo = PerfRepo::new(db.clone());

        repo.save_session("s1", &dummy_result("cpu"), None, session_source::MANUAL).await.unwrap();
        repo.save_session("s2", &dummy_result("ram"), None, session_source::MANUAL).await.unwrap();
        repo.save_session("s3", &dummy_result("io"), None, session_source::OPTIMIZATION_CATALOG).await.unwrap();

        let manual = repo.sessions_by_source(session_source::MANUAL, 50).await.unwrap();
        let catalog = repo.sessions_by_source(session_source::OPTIMIZATION_CATALOG, 50).await.unwrap();

        assert_eq!(manual.len(), 2, "deve retornar apenas sessões 'manual'");
        assert_eq!(catalog.len(), 1, "deve retornar apenas sessões 'optimization_catalog'");
    }

    // ── fallback legado: INSERT sem source explícito usa DEFAULT 'manual' ──
    // Simula banco pré-V4.1-E onde a coluna existe mas o código não passava source.
    #[tokio::test]
    async fn sessions_by_source_legacy_default_is_manual() {
        let (db, _dir) = make_db().await;

        // INSERT via SQL direto sem incluir source → usa DEFAULT 'manual'.
        sqlx::query(
            "INSERT INTO benchmark_sessions \
             (ts, kind, suite_version, duration_ms, runs, confidence, contaminated) \
             VALUES (1, 'cpu', 'v0', 100, 3, 70, 0)",
        )
        .execute(&db)
        .await
        .unwrap();

        // DEFAULT 'manual' deve fazer esta sessão aparecer no filtro.
        let repo = PerfRepo::new(db.clone());
        let manual = repo.sessions_by_source(session_source::MANUAL, 50).await.unwrap();
        assert_eq!(manual.len(), 1, "INSERT sem source deve usar DEFAULT 'manual'");
    }

    // ── OptRepo::save_run persiste source corretamente ──
    #[tokio::test]
    async fn save_run_persists_source() {
        let (db, _dir) = make_db().await;
        let snap_id: i64 = sqlx::query_scalar(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status) \
             VALUES (1, 'r', 'h', 'active') RETURNING id",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        let repo = OptRepo::new(db.clone());
        let info = dummy_run_info("timer_resolution");
        repo.save_run(&info, snap_id, None, session_source::OPTIMIZATION_CATALOG)
            .await
            .unwrap();

        let src: String = sqlx::query_scalar(
            "SELECT source FROM optimization_runs WHERE optimization_id = 'timer_resolution'",
        )
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(src, session_source::OPTIMIZATION_CATALOG);
    }

    // ── runs_by_source filtra corretamente ──
    #[tokio::test]
    async fn runs_by_source_filters_correctly() {
        let (db, _dir) = make_db().await;
        let snap_id: i64 = sqlx::query_scalar(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status) \
             VALUES (1, 'r', 'h', 'active') RETURNING id",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        let repo = OptRepo::new(db.clone());
        repo.save_run(&dummy_run_info("opt_a"), snap_id, None, session_source::OPTIMIZATION_CATALOG).await.unwrap();
        repo.save_run(&dummy_run_info("opt_b"), snap_id, None, session_source::OPTIMIZATION_CATALOG).await.unwrap();
        repo.save_run(&dummy_run_info("opt_c"), snap_id, None, session_source::PROFILE_ACTIVATION).await.unwrap();

        let catalog = repo.runs_by_source(session_source::OPTIMIZATION_CATALOG, 50).await.unwrap();
        let profile = repo.runs_by_source(session_source::PROFILE_ACTIVATION, 50).await.unwrap();

        assert_eq!(catalog.len(), 2, "deve retornar apenas runs 'optimization_catalog'");
        assert_eq!(profile.len(), 1, "deve retornar apenas runs 'profile_activation'");
    }

    // ── fallback legado: INSERT sem source explícito usa DEFAULT 'optimization_catalog' ──
    // Simula banco pré-V4.1-E onde a coluna existe mas o código não passava source.
    #[tokio::test]
    async fn runs_by_source_legacy_default_is_optimization_catalog() {
        let (db, _dir) = make_db().await;
        let snap_id: i64 = sqlx::query_scalar(
            "INSERT INTO snapshots (ts, reason, integrity_hash, status) \
             VALUES (1, 'r', 'h', 'active') RETURNING id",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        let info = dummy_run_info("gpu_scheduling");
        let json = serde_json::to_string(&info).unwrap();
        // INSERT via SQL direto sem incluir source → usa DEFAULT 'optimization_catalog'.
        sqlx::query(
            "INSERT INTO optimization_runs \
             (ts, optimization_id, snapshot_id, status, decision, confidence, evidence_json) \
             VALUES (1, 'gpu_scheduling', ?, 'kept', 'keep', 90, ?)",
        )
        .bind(snap_id)
        .bind(&json)
        .execute(&db)
        .await
        .unwrap();

        let repo = OptRepo::new(db.clone());
        let catalog = repo.runs_by_source(session_source::OPTIMIZATION_CATALOG, 50).await.unwrap();
        assert_eq!(catalog.len(), 1, "INSERT sem source deve usar DEFAULT 'optimization_catalog'");
    }

    // ── source 'automatic' é persistido (reservado para uso futuro) ──
    #[tokio::test]
    async fn automatic_source_persists() {
        let (db, _dir) = make_db().await;
        let repo = PerfRepo::new(db.clone());
        repo.save_session("auto-bench", &dummy_result("complete"), None, session_source::AUTOMATIC)
            .await
            .unwrap();

        let sessions = repo.sessions_by_source(session_source::AUTOMATIC, 50).await.unwrap();
        assert_eq!(sessions.len(), 1);
    }
}
