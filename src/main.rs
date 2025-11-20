//
// Copyright (c) 2025 murilo ijanc' <murilo@ijanc.org>
//
// Permission to use, copy, modify, and distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use anyhow::{Context, anyhow, bail};
use clap::{ArgAction, Parser};
use ignore::{WalkBuilder, WalkState};
use log::{LevelFilter, debug, error, info, warn};

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_HASH", "unknown"),
    " ",
    env!("BUILD_DATE", "unknown"),
    ")",
);

/// Simple Image metadata cleaner.
///
/// Recursively walks an input directory, removes metadata from JPEG files
/// and writes the cleaned copies into an output directory, preserving the
/// directory structure.
#[derive(Debug, Parser)]
#[command(
    name = "imgst",
    about = "Simple Image metadata cleaner",
    version = VERSION,
    author,
    propagate_version = true
)]
struct Args {
    /// Input directory containing original images
    #[arg(short, long)]
    input: PathBuf,

    /// Ouput directoryu where cleaned images will be written
    #[arg(short, long)]
    output: PathBuf,

    /// Number of worker threads for directory walking
    #[arg(long, default_value_t = 0)]
    num_threads: usize,

    /// Only print what would be done, do not write files
    #[arg(long)]
    dry_run: bool,

    /// Increase verbosity (use -v, -vv, ...).
    ///
    /// When no RUST_LOG is set, a single -v switches the log level to DEBUG.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_logger(args.verbose);

    if !args.input.is_dir() {
        bail!("input path '{}' is not directory", args.input.display());
    }

    if !args.output.exists() {
        fs::create_dir_all(&args.output).with_context(|| {
            format!("failed to create output dir '{}'", args.output.display())
        })?;
    } else if !args.output.is_dir() {
        bail!(
            "output path '{}' exists but is not directory",
            args.input.display()
        );
    }

    info!("input directory: {}", args.input.display());
    info!("output directory: {}", args.output.display());
    info!("threads : {}", args.num_threads);
    if args.dry_run {
        info!("running in DRY_RUN mode");
    }

    let input_root = Arc::new(args.input);
    let output_root = Arc::new(args.output);
    let dry_run = args.dry_run;

    // counter
    let processed = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));

    let walker = WalkBuilder::new(&*input_root)
        .hidden(false)
        .follow_links(false)
        .standard_filters(true)
        .threads(args.num_threads)
        .build_parallel();

    walker.run(|| {
        let input_root = Arc::clone(&input_root);
        let output_root = Arc::clone(&output_root);
        let processed = Arc::clone(&processed);
        let skipped = Arc::clone(&skipped);
        let failed = Arc::clone(&failed);

        Box::new(move |result| {
            match result {
                Ok(entry) => {
                    let path = entry.path();

                    // regular file
                    if !entry
                        .file_type()
                        .map(|ft| ft.is_file())
                        .unwrap_or(false)
                    {
                        return WalkState::Continue;
                    }

                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_ascii_lowercase());

                    let is_jpeg =
                        matches!(ext.as_deref(), Some("jpg" | "jpeg"));

                    if !is_jpeg {
                        skipped.fetch_add(1, Ordering::Relaxed);
                        return WalkState::Continue;
                    }

                    match process_img(&input_root, &output_root, path, dry_run)
                    {
                        Ok(()) => {
                            processed.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(err) => {
                            failed.fetch_add(1, Ordering::Relaxed);
                            error!(
                                "failed to process '{}': {err:#}",
                                path.display()
                            );
                        }
                    }
                }
                Err(err) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    error!("walk error: {err}");
                }
            }

            WalkState::Continue
        })
    });

    info!(
        "done: processed={} skipped={} failed={}",
        processed.load(Ordering::Relaxed),
        skipped.load(Ordering::Relaxed),
        failed.load(Ordering::Relaxed),
    );

    if failed.load(Ordering::Relaxed) > 0 {
        warn!("some files failed to process");
    }

    Ok(())
}

fn process_img(
    input_root: &Path,
    output_root: &Path,
    src: &Path,
    dry_run: bool,
) -> anyhow::Result<()> {
    let rel_path = match src.strip_prefix(input_root) {
        Ok(rel) => rel.to_path_buf(),
        Err(_) => {
            src.file_name().map(PathBuf::from).ok_or_else(|| anyhow!(""))?
        }
    };

    let dst = output_root.join(rel_path);

    if dry_run {
        debug!(
            "dry-run: would clean '{}' -> '{}'",
            src.display(),
            dst.display()
        );
    }

    Ok(())
}

fn init_logger(verbose: u8) {
    use std::io::Write;

    if std::env::var_os("RUST_LOG").is_some() {
        env_logger::builder()
            .format(|buf, record| {
                writeln!(buf, "[{}]: {}", record.level(), record.args())
            })
            .init();
        return;
    }

    let level =
        if verbose > 0 { LevelFilter::Debug } else { LevelFilter::Info };

    env_logger::builder()
        .filter(None, level)
        .format(|buf, record| {
            writeln!(buf, "[{}]: {}", record.level(), record.args())
        })
        .init();
}
