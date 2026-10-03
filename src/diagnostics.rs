use crate::app_paths::AppPaths;
use anyhow::Result;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{Builder, Rotation},
};
use tracing_subscriber::EnvFilter;

/// Bounded asynchronous diagnostics, daily rotation, seven files retained.
/// Guard is process-owned and dropped after application workers stop.
pub fn init(paths: Option<&AppPaths>) -> Result<Option<WorkerGuard>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if let Some(paths) = paths {
        let appender = Builder::new()
            .rotation(Rotation::DAILY)
            .filename_prefix("process")
            .filename_suffix("jsonl")
            .max_log_files(7)
            .build(paths.root.join("logs"))?;
        let (writer, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(1024)
            .lossy(true)
            .finish(appender);
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_writer(writer)
            .try_init()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(Some(guard))
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .try_init()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(None)
    }
}
