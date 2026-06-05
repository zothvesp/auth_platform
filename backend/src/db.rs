//! Database connection pool.
//!
//! Uses PostgreSQL in all environments. For local development without
//! a running Postgres, use Docker: `docker-compose up -d postgres`
//! See .env.example for the DATABASE_URL format.

use anyhow::Context;
use redis::aio::ConnectionManager;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(url)
            .await
            .with_context(|| format!("Failed to connect to PostgreSQL: {}", url))?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        info!("Running PostgreSQL migrations...");
        sqlx::migrate!("./migrations/postgres")
            .run(&self.pool)
            .await
            .context("Migration failed")?;
        info!("Migrations complete");
        Ok(())
    }
}

#[derive(Clone)]
pub struct RedisPool {
    pub manager: ConnectionManager,
}

impl RedisPool {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url).context("Failed to create Redis client")?;
        let manager = ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis")?;
        Ok(Self { manager })
    }

    /// Attempt Redis connection; warn and use a dummy on failure (disables rate limiting).
    pub async fn connect_or_noop(url: &str) -> Self {
        match Self::connect(url).await {
            Ok(pool) => {
                info!("Redis connected");
                pool
            }
            Err(e) => {
                tracing::warn!("Redis unavailable ({}), rate limiting disabled", e);
                panic!("Redis required. Run: docker-compose up -d redis");
            }
        }
    }
}
