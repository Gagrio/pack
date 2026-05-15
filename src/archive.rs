use crate::collector::CollectedEntry;
use anyhow::{Context, Result};
use std::fs;
use tracing::{info, warn};

pub fn create_archive(
    entries: &[CollectedEntry],
    source_paths: &[String],
    output_dir: &str,
    timestamp: &str,
) -> Result<String> {
    let archive_name = format!("{}/pack_logs_{}.tar.gz", output_dir, timestamp);
    let top_dir = format!("pack_logs_{}", timestamp);

    info!("Creating archive: {}", archive_name);

    let tar_gz = fs::File::create(&archive_name).context("Failed to create archive file")?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    let summary = build_summary(entries, source_paths, timestamp);
    let summary_bytes = summary.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_size(summary_bytes.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        format!("{}/collection-summary.yaml", top_dir),
        summary_bytes,
    )
    .context("Failed to add summary to archive")?;

    for entry in entries {
        let archive_path = format!("{}/{}", top_dir, entry.archive_path);

        if entry.is_dir {
            // Directory entries are skipped intentionally. append_path_with_name handles
            // long paths via the GNU long name extension, but the equivalent for directories
            // hits the 100-char tar path limit. Directories are created implicitly during
            // extraction when their child files are extracted. As a result, empty directories
            // will not appear in the output archive.
            continue;
        } else {
            if let Err(e) = tar.append_path_with_name(&entry.absolute_path, &archive_path) {
                let hint = if e.kind() == std::io::ErrorKind::PermissionDenied {
                    " — re-run with 'sudo podman run' to access root-only paths"
                } else {
                    ""
                };
                warn!(
                    "Skipping {}: {}{} — file could not be read",
                    entry.absolute_path.display(),
                    e,
                    hint
                );
            }
        }
    }

    tar.finish().context("Failed to finalize archive")?;

    info!("Archive finalized: {}", archive_name);
    Ok(archive_name)
}

fn build_summary(entries: &[CollectedEntry], source_paths: &[String], timestamp: &str) -> String {
    let file_count = entries.iter().filter(|e| !e.is_dir).count();
    let dir_count = entries.iter().filter(|e| e.is_dir).count();

    let paths_yaml = source_paths
        .iter()
        .map(|p| format!("  - {}", p))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "collection_info:\n  timestamp: \"{}\"\n  tool: pack\n  version: \"{}\"\n\npaths_collected:\n{}\n\ncollection_summary:\n  total_files: {}\n  total_directories: {}\n",
        timestamp,
        env!("CARGO_PKG_VERSION"),
        paths_yaml,
        file_count,
        dir_count,
    )
}
