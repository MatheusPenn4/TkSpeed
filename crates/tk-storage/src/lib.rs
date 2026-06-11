//! TkStorage — camada de persistência (SQLite + WAL).
//! Expõe o pool, roda migrations e fornece repositórios (Repository pattern)
//! com queries em runtime (build hermético, sem DATABASE_URL).

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Db = SqlitePool;

pub mod housekeeping;
pub mod repos;
pub use repos::*;

/// Erro unificado da camada de storage.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Timestamp epoch em milissegundos (UTC).
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Abre (ou cria) o banco e aplica todas as migrations pendentes.
/// Usa `filename()` (não URL) para lidar com caminhos Windows com backslashes.
pub async fn open(db_path: &str) -> std::result::Result<Db, sqlx::Error> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true)
        // Espera até 5s por um lock em vez de falhar com SQLITE_BUSY de imediato
        // (escritas concorrentes: monitor + análise + snapshots).
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
