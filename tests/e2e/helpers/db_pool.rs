use anyhow::Result;
use parking_lot::RwLock;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::VecDeque;
use std::sync::Arc;
use uuid::Uuid;

/// A pool that manages isolated test databases within a single PostgreSQL container
pub struct DatabasePool {
    /// The host port where the PostgreSQL container is exposed
    container_port: u16,
    /// Available databases ready to be used
    available: Arc<RwLock<VecDeque<String>>>,
    /// Databases currently in use
    in_use: Arc<RwLock<Vec<String>>>,
}

impl DatabasePool {
    /// Create a new database pool connected to the PostgreSQL container
    pub fn new(container_port: u16) -> Self {
        Self {
            container_port,
            available: Arc::new(RwLock::new(VecDeque::new())),
            in_use: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a database from the pool, creating a new one if needed
    pub async fn get_database(&self) -> Result<PooledDatabase> {
        // Try to get an available database first
        let db_name = {
            let mut available = self.available.write();
            available.pop_front()
        };

        let db_name = if let Some(name) = db_name {
            // Reuse existing database
            name
        } else {
            // Create a new database
            self.create_new_database().await?
        };

        // Mark as in use
        {
            let mut in_use = self.in_use.write();
            in_use.push(db_name.clone());
        }

        // Create connection pool for this database
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/{}",
            self.container_port, db_name
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(PooledDatabase {
            db_name: db_name.clone(),
            database_url,
            pool,
            pool_ref: Arc::new(DatabasePoolRef {
                container_port: self.container_port,
                available: self.available.clone(),
                in_use: self.in_use.clone(),
            }),
        })
    }

    /// Create a new isolated test database with migrations
    async fn create_new_database(&self) -> Result<String> {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());

        // Connect to postgres database to create new database
        let admin_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            self.container_port
        );

        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&admin_url)
            .await?;

        // Create the database (need to use raw SQL, can't use prepared statements)
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await?;

        // Close admin connection
        admin_pool.close().await;

        // Connect to the new database and run migrations
        let new_db_url = format!(
            "postgresql://postgres:postgres@localhost:{}/{}",
            self.container_port, db_name
        );

        let new_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&new_db_url)
            .await?;

        sqlx::migrate!("./migrations").run(&new_pool).await?;

        new_pool.close().await;

        Ok(db_name)
    }

    /// Return a database back to the pool after cleaning it
    #[allow(dead_code)]
    async fn return_database(&self, db_name: String) {
        // Remove from in-use
        {
            let mut in_use = self.in_use.write();
            in_use.retain(|name| name != &db_name);
        }

        // Clean the database and return to available pool
        if let Ok(_) = self.cleanup_database(&db_name).await {
            let mut available = self.available.write();
            available.push_back(db_name);
        }
        // If cleanup fails, just drop the database (don't reuse)
    }

    /// Clean all tables in a database to prepare it for reuse
    #[allow(dead_code)]
    async fn cleanup_database(&self, db_name: &str) -> Result<()> {
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/{}",
            self.container_port, db_name
        );

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;

        // Truncate all tables to clean the database
        sqlx::query("TRUNCATE TABLE feeds, users, refresh_tokens, usage_tracking CASCADE")
            .execute(&pool)
            .await?;

        pool.close().await;

        Ok(())
    }
}

/// Reference to the database pool for cleanup on drop
struct DatabasePoolRef {
    container_port: u16,
    available: Arc<RwLock<VecDeque<String>>>,
    in_use: Arc<RwLock<Vec<String>>>,
}

impl DatabasePoolRef {
    /// Return a database back to the pool
    fn return_database(&self, db_name: String) {
        // Remove from in-use
        {
            let mut in_use = self.in_use.write();
            in_use.retain(|name| name != &db_name);
        }

        // Spawn a task to clean and return the database
        let container_port = self.container_port;
        let available = self.available.clone();

        tokio::spawn(async move {
            // Clean the database
            let database_url = format!(
                "postgresql://postgres:postgres@localhost:{}/{}",
                container_port, db_name
            );

            if let Ok(pool) = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
            {
                // Try to clean - if it fails, just don't reuse the database
                if sqlx::query(
                    "TRUNCATE TABLE feeds, users, refresh_tokens, usage_tracking CASCADE",
                )
                .execute(&pool)
                .await
                .is_ok()
                {
                    // Successfully cleaned, return to pool
                    let mut available = available.write();
                    available.push_back(db_name);
                }

                pool.close().await;
            }
        });
    }
}

/// A database leased from the pool
pub struct PooledDatabase {
    pub db_name: String,
    pub database_url: String,
    pub pool: PgPool,
    pool_ref: Arc<DatabasePoolRef>,
}

impl Drop for PooledDatabase {
    fn drop(&mut self) {
        // Return the database to the pool when dropped
        self.pool_ref.return_database(self.db_name.clone());
    }
}
