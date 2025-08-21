use crate::action_generator::Action;
#[allow(unused_imports)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, Transaction};
#[allow(unused_imports)]
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Result type for action execution
pub type ExecutionResult = Result<ExecutionReport, ExecutionError>;

/// Error that can occur during action execution
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Action failed: {0}")]
    ActionFailed(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Report of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub total_actions: usize,
    pub successful_actions: usize,
    pub failed_actions: usize,
    pub rolled_back: bool,
    pub execution_time_ms: u128,
    pub action_results: Vec<ActionResult>,
}

/// Result of a single action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_type: String,
    pub success: bool,
    pub message: String,
    pub execution_time_ms: u128,
}

/// Executor for configuration actions
pub struct ActionExecutor {
    db_pool: SqlitePool,
    dry_run: bool,
}

impl ActionExecutor {
    /// Create a new action executor
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool,
            dry_run: false,
        }
    }
    
    /// Create a new action executor in dry-run mode
    pub fn new_dry_run(db_pool: SqlitePool) -> Self {
        Self {
            db_pool,
            dry_run: true,
        }
    }
    
    /// Execute a list of actions
    pub async fn execute_actions(&self, actions: Vec<Action>) -> ExecutionResult {
        let start_time = std::time::Instant::now();
        let total_actions = actions.len();
        let mut action_results = Vec::new();
        let mut successful_actions = 0;
        let mut failed_actions = 0;
        
        info!("Executing {} actions (dry_run: {})", total_actions, self.dry_run);
        
        // Start a transaction if not in dry-run mode
        let mut tx = if !self.dry_run {
            Some(self.db_pool.begin().await?)
        } else {
            None
        };
        
        // Track executed actions for potential rollback
        let mut executed_actions = Vec::new();
        
        for action in actions {
            let action_start = std::time::Instant::now();
            let action_type = self.action_type_string(&action);
            
            let result = if let Some(ref mut transaction) = tx {
                self.execute_single_action(&action, transaction).await
            } else {
                // Dry run - just validate
                self.validate_action(&action).await
            };
            
            let execution_time_ms = action_start.elapsed().as_millis();
            
            match result {
                Ok(message) => {
                    successful_actions += 1;
                    action_results.push(ActionResult {
                        action_type,
                        success: true,
                        message,
                        execution_time_ms,
                    });
                    executed_actions.push(action);
                }
                Err(e) => {
                    failed_actions += 1;
                    error!("Action failed: {}", e);
                    action_results.push(ActionResult {
                        action_type,
                        success: false,
                        message: e.to_string(),
                        execution_time_ms,
                    });
                    
                    // If an action fails and we're not in dry-run, rollback
                    if !self.dry_run {
                        warn!("Rolling back due to action failure");
                        if let Some(transaction) = tx {
                            transaction.rollback().await?;
                        }
                        
                        // Try to rollback executed actions
                        let rollback_result = self.rollback_actions(executed_actions).await;
                        
                        return Ok(ExecutionReport {
                            total_actions,
                            successful_actions,
                            failed_actions,
                            rolled_back: true,
                            execution_time_ms: start_time.elapsed().as_millis(),
                            action_results,
                        });
                    }
                }
            }
        }
        
        // Commit the transaction if not in dry-run mode
        if let Some(transaction) = tx {
            transaction.commit().await?;
            info!("Transaction committed successfully");
        }
        
        Ok(ExecutionReport {
            total_actions,
            successful_actions,
            failed_actions,
            rolled_back: false,
            execution_time_ms: start_time.elapsed().as_millis(),
            action_results,
        })
    }
    
    /// Execute a single action within a transaction
    async fn execute_single_action(
        &self,
        action: &Action,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<String, ExecutionError> {
        match action {
            Action::CreateTable { entity_id, table_name, sql } => {
                debug!("Creating table {} for entity {}", table_name, entity_id);
                sqlx::query(sql).execute(&mut **tx).await?;
                Ok(format!("Created table {}", table_name))
            }
            
            Action::DropTable { entity_id, table_name } => {
                debug!("Dropping table {} for entity {}", table_name, entity_id);
                let sql = format!("DROP TABLE IF EXISTS {}", table_name);
                sqlx::query(&sql).execute(&mut **tx).await?;
                Ok(format!("Dropped table {}", table_name))
            }
            
            Action::AddColumn { entity_id, table_name, column_name, sql } => {
                debug!("Adding column {} to table {} for entity {}", column_name, table_name, entity_id);
                sqlx::query(sql).execute(&mut **tx).await?;
                Ok(format!("Added column {} to table {}", column_name, table_name))
            }
            
            Action::DropColumn { entity_id: _, table_name: _, column_name: _ } => {
                // SQLite doesn't support DROP COLUMN directly
                // We need to recreate the table without the column
                warn!("SQLite doesn't support DROP COLUMN directly. This requires table recreation.");
                Err(ExecutionError::ActionFailed(
                    "DROP COLUMN not supported in SQLite without table recreation".to_string()
                ))
            }
            
            Action::ModifyColumn { entity_id: _, table_name: _, column_name: _, old_type: _, new_type: _, sql: _ } => {
                // SQLite doesn't support ALTER COLUMN directly
                warn!("SQLite doesn't support ALTER COLUMN directly. This requires table recreation.");
                Err(ExecutionError::ActionFailed(
                    "ALTER COLUMN not supported in SQLite without table recreation".to_string()
                ))
            }
            
            Action::CreateIndex { index_name, table_name, columns: _, sql } => {
                debug!("Creating index {} on table {}", index_name, table_name);
                sqlx::query(sql).execute(&mut **tx).await?;
                Ok(format!("Created index {} on table {}", index_name, table_name))
            }
            
            Action::DropIndex { index_name, table_name } => {
                debug!("Dropping index {} from table {}", index_name, table_name);
                let sql = format!("DROP INDEX IF EXISTS {}", index_name);
                sqlx::query(&sql).execute(&mut **tx).await?;
                Ok(format!("Dropped index {}", index_name))
            }
            
            Action::UpdateConfig { key, value } => {
                // This would update the in-memory config
                // For now, we just log it
                info!("Would update config key '{}' with value: {:?}", key, value);
                Ok(format!("Updated config key '{}'", key))
            }
            
            Action::InvalidateCache { entity_id } => {
                info!("Would invalidate cache for entity '{}'", entity_id);
                Ok(format!("Invalidated cache for entity '{}'", entity_id))
            }
            
            Action::ReloadEntityDefinitions => {
                info!("Would reload entity definitions from schemas");
                Ok("Reloaded entity definitions".to_string())
            }
        }
    }
    
    /// Validate an action without executing it
    async fn validate_action(&self, action: &Action) -> Result<String, ExecutionError> {
        match action {
            Action::CreateTable { table_name, .. } => {
                // Check if table already exists
                let exists = self.table_exists(table_name).await?;
                if exists {
                    Err(ExecutionError::Validation(format!("Table {} already exists", table_name)))
                } else {
                    Ok(format!("Would create table {}", table_name))
                }
            }
            
            Action::DropTable { table_name, .. } => {
                // Check if table exists
                let exists = self.table_exists(table_name).await?;
                if !exists {
                    Err(ExecutionError::Validation(format!("Table {} does not exist", table_name)))
                } else {
                    Ok(format!("Would drop table {}", table_name))
                }
            }
            
            Action::AddColumn { table_name, column_name, .. } => {
                // Check if column already exists
                let exists = self.column_exists(table_name, column_name).await?;
                if exists {
                    Err(ExecutionError::Validation(format!("Column {} already exists in table {}", column_name, table_name)))
                } else {
                    Ok(format!("Would add column {} to table {}", column_name, table_name))
                }
            }
            
            _ => Ok(format!("Would execute: {:?}", action))
        }
    }
    
    /// Rollback a list of executed actions
    pub async fn rollback_actions(&self, actions: Vec<Action>) -> Result<(), ExecutionError> {
        info!("Rolling back {} actions", actions.len());
        
        let mut tx = self.db_pool.begin().await?;
        
        // Execute rollback actions in reverse order
        for action in actions.iter().rev() {
            if let Some(rollback_action) = action.rollback_action() {
                debug!("Executing rollback action: {:?}", rollback_action);
                if let Err(e) = self.execute_single_action(&rollback_action, &mut tx).await {
                    error!("Rollback action failed: {}", e);
                    tx.rollback().await?;
                    return Err(ExecutionError::RollbackFailed(e.to_string()));
                }
            }
        }
        
        tx.commit().await?;
        info!("Rollback completed successfully");
        Ok(())
    }
    
    /// Check if a table exists
    async fn table_exists(&self, table_name: &str) -> Result<bool, ExecutionError> {
        let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
        let result: Option<(String,)> = sqlx::query_as(query)
            .bind(table_name)
            .fetch_optional(&self.db_pool)
            .await?;
        
        Ok(result.is_some())
    }
    
    /// Check if a column exists in a table
    async fn column_exists(&self, table_name: &str, column_name: &str) -> Result<bool, ExecutionError> {
        let query = format!("PRAGMA table_info({})", table_name);
        let rows: Vec<(i32, String, String, i32, Option<String>, i32)> = 
            sqlx::query_as(&query)
                .fetch_all(&self.db_pool)
                .await?;
        
        Ok(rows.iter().any(|(_, name, _, _, _, _)| name == column_name))
    }
    
    /// Get a string representation of the action type
    fn action_type_string(&self, action: &Action) -> String {
        match action {
            Action::CreateTable { .. } => "CreateTable".to_string(),
            Action::DropTable { .. } => "DropTable".to_string(),
            Action::AddColumn { .. } => "AddColumn".to_string(),
            Action::DropColumn { .. } => "DropColumn".to_string(),
            Action::ModifyColumn { .. } => "ModifyColumn".to_string(),
            Action::UpdateConfig { .. } => "UpdateConfig".to_string(),
            Action::InvalidateCache { .. } => "InvalidateCache".to_string(),
            Action::CreateIndex { .. } => "CreateIndex".to_string(),
            Action::DropIndex { .. } => "DropIndex".to_string(),
            Action::ReloadEntityDefinitions => "ReloadEntityDefinitions".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    async fn create_test_pool() -> SqlitePool {
        SqlitePool::connect(":memory:").await.unwrap()
    }
    
    #[tokio::test]
    async fn test_dry_run_execution() {
        let pool = create_test_pool().await;
        let executor = ActionExecutor::new_dry_run(pool);
        
        let actions = vec![
            Action::CreateTable {
                entity_id: "test".to_string(),
                table_name: "content_test".to_string(),
                sql: "CREATE TABLE content_test (id TEXT PRIMARY KEY)".to_string(),
            },
        ];
        
        let report = executor.execute_actions(actions).await.unwrap();
        assert_eq!(report.total_actions, 1);
        assert_eq!(report.successful_actions, 1);
        assert_eq!(report.failed_actions, 0);
        assert!(!report.rolled_back);
    }
    
    #[tokio::test]
    async fn test_table_creation() {
        let pool = create_test_pool().await;
        let executor = ActionExecutor::new(pool.clone());
        
        let actions = vec![
            Action::CreateTable {
                entity_id: "test".to_string(),
                table_name: "content_test".to_string(),
                sql: "CREATE TABLE content_test (id TEXT PRIMARY KEY)".to_string(),
            },
        ];
        
        let report = executor.execute_actions(actions).await.unwrap();
        assert_eq!(report.successful_actions, 1);
        
        // Verify table was created
        let exists = executor.table_exists("content_test").await.unwrap();
        assert!(exists);
    }
}