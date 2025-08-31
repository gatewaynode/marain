//! Test module to verify secure log behavior after restart
//! This test ensures that the log chain verification works correctly
//! when the logger is restarted and new entries are added.

#[cfg(test)]
mod tests {
    use crate::secure_log::{SecureLogConfig, SecureLogger};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_log_chain_after_restart() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("secure.log");

        let config = SecureLogConfig {
            log_path: log_path.clone(),
            max_size_mb: 10,
            max_rotations: 5,
            enable_verification: true,
        };

        // First logger instance - create initial entries
        {
            let logger = SecureLogger::new(config.clone()).unwrap();

            // Log some initial actions
            for i in 0..3 {
                logger
                    .log_action(i, &format!("initial_action_{}", i), None, None, None, true)
                    .await
                    .unwrap();
            }

            // Verify the chain is valid
            assert!(logger.verify_log_chain().await.unwrap());
        }

        // Second logger instance - simulating a restart
        {
            let logger = SecureLogger::new(config.clone()).unwrap();

            // Add more entries after "restart"
            for i in 3..6 {
                logger
                    .log_action(
                        i,
                        &format!("post_restart_action_{}", i),
                        None,
                        None,
                        None,
                        true,
                    )
                    .await
                    .unwrap();
            }

            // Verify the entire chain is still valid
            assert!(
                logger.verify_log_chain().await.unwrap(),
                "Log chain verification failed after restart and new entries"
            );
        }

        // Third logger instance - verify chain integrity one more time
        {
            let logger = SecureLogger::new(config).unwrap();

            // Just verify without adding new entries
            assert!(
                logger.verify_log_chain().await.unwrap(),
                "Log chain verification failed on third restart"
            );
        }
    }
}
