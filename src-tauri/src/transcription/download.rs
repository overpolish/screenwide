// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::catalogue::{Model, ModelFile};
use super::models;

struct Download {
  progress: f32,
  cancelled: Arc<AtomicBool>,
}

static DOWNLOADS: LazyLock<Mutex<HashMap<&'static str, Download>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

fn downloads() -> std::sync::MutexGuard<'static, HashMap<&'static str, Download>> {
  DOWNLOADS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// How far the download of `id` is, 0 to 1, while one runs.
pub(super) fn progress(id: &str) -> Option<f32> {
  downloads().get(id).map(|download| download.progress)
}

pub(super) fn cancel(id: &str) {
  if let Some(download) = downloads().get(id) {
    download.cancelled.store(true, Ordering::Relaxed);
  }
}

/// Fetches `model`, returning once it is in place, cancelled, or failed.
/// Progress goes out as change events meanwhile. Asked again while it runs,
/// it leaves the first download to finish.
pub(super) async fn run(app: &AppHandle, model: &'static Model) -> Result<(), String> {
  let cancelled = {
    let mut downloads = downloads();
    if downloads.contains_key(model.id) {
      return Ok(());
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    downloads.insert(
      model.id,
      Download {
        cancelled: cancelled.clone(),
        progress: 0.0,
      },
    );
    cancelled
  };
  models::changed(app);
  let result = fetch(app, model, &cancelled).await;
  downloads().remove(model.id);
  models::changed(app);
  if cancelled.load(Ordering::Relaxed) {
    return Ok(());
  }
  if result.is_ok() {
    // Notes waiting for a model can be transcribed now.
    super::notes::changed(app);
  }
  result
}

/// Downloads each file beside its place and moves it in only once its size
/// and checksum match, so a file under its own name is always whole. Files
/// already in place are kept.
async fn fetch(app: &AppHandle, model: &Model, cancelled: &AtomicBool) -> Result<(), String> {
  let total = model.size_bytes();
  let mut before: u64 = 0;
  for file in model.files {
    let destination = models::file_path(app, file)?;
    let whole =
      std::fs::metadata(&destination).is_ok_and(|metadata| metadata.len() == file.size_bytes);
    if !whole {
      let partial = destination.with_extension("part");
      if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
      }
      let progress = Progress {
        app,
        id: model.id,
        before,
        total,
      };
      let result = receive(&progress, file, cancelled, &partial)
        .await
        .and_then(|()| std::fs::rename(&partial, &destination).map_err(|error| error.to_string()));
      if result.is_err() {
        let _ = std::fs::remove_file(&partial);
        return result;
      }
    }
    before += file.size_bytes;
  }
  Ok(())
}

/// Where a model's download stands across all its files.
struct Progress<'a> {
  app: &'a AppHandle,
  id: &'static str,
  /// Bytes of the files before this one.
  before: u64,
  total: u64,
}

async fn receive(
  progress: &Progress<'_>,
  file: &ModelFile,
  cancelled: &AtomicBool,
  partial: &Path,
) -> Result<(), String> {
  let failed = |error: reqwest::Error| format!("Could not download the model: {error}");
  let mut response = client()?
    .get(file.url)
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(failed)?;
  let mut output = File::create(partial).map_err(|error| error.to_string())?;
  let mut hasher = Sha256::new();
  let mut received: u64 = 0;
  let mut shown = 0;
  while let Some(chunk) = response.chunk().await.map_err(failed)? {
    if cancelled.load(Ordering::Relaxed) {
      return Err("Cancelled".to_owned());
    }
    output
      .write_all(&chunk)
      .map_err(|error| format!("Could not save the model: {error}"))?;
    hasher.update(&chunk);
    received += chunk.len() as u64;
    // Every half percent: smooth enough to watch, few enough events.
    let per_mille = ((progress.before + received) * 1000 / progress.total.max(1)).min(1000);
    if per_mille >= shown + 5 {
      shown = per_mille;
      if let Some(download) = downloads().get_mut(progress.id) {
        #[expect(clippy::cast_precision_loss, reason = "at most 1000")]
        let fraction = per_mille as f32 / 1000.0;
        download.progress = fraction;
      }
      models::changed(progress.app);
    }
  }
  output
    .sync_all()
    .map_err(|error| format!("Could not save the model: {error}"))?;
  if received != file.size_bytes || hex(&hasher.finalize()) != file.sha256 {
    return Err("The model arrived damaged. Try downloading it again.".to_owned());
  }
  Ok(())
}

fn client() -> Result<reqwest::Client, String> {
  // reqwest is built without a TLS provider of its own; the updater installs
  // the same one when it checks, whichever runs first.
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }
  reqwest::Client::builder()
    .user_agent(concat!("Screenwide/", env!("CARGO_PKG_VERSION")))
    .connect_timeout(Duration::from_secs(30))
    // Between chunks rather than for the whole file, which takes as long as
    // the connection needs.
    .read_timeout(Duration::from_secs(60))
    .build()
    .map_err(|error| error.to_string())
}

pub(super) fn hex(bytes: &[u8]) -> String {
  use std::fmt::Write as _;
  bytes.iter().fold(String::new(), |mut text, byte| {
    let _ = write!(text, "{byte:02x}");
    text
  })
}
