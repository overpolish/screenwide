// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::time::Duration;

use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::models;
use screenwide_speech::catalogue::{Model, ModelFile};

struct Download {
  progress: f32,
  /// The bytes this download fetches: the model's files not yet whole, or
  /// every file when it is downloaded again.
  total: u64,
  cancelled: Arc<AtomicBool>,
}

static DOWNLOADS: LazyLock<Mutex<HashMap<&'static str, Download>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// The bytes received of each file being fetched, by its name. Models share
/// files, and two downloads writing one file at once corrupt it or rename it
/// from under each other, so each file is fetched by one download at a time;
/// another needing it waits, counting the bytes as its own.
static FETCHING: LazyLock<Mutex<HashMap<&'static str, u64>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// How often a download waiting for another's file looks again.
const WAIT: Duration = Duration::from_millis(250);

fn locked<T>(mutex: &'static Mutex<T>) -> MutexGuard<'static, T> {
  mutex
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// How far the download of `id` is, 0 to 1, and the bytes it fetches, while
/// one runs.
pub(super) fn progress(id: &str) -> Option<(f32, u64)> {
  locked(&DOWNLOADS)
    .get(id)
    .map(|download| (download.progress, download.total))
}

pub(super) fn cancel(id: &str) {
  if let Some(download) = locked(&DOWNLOADS).get(id) {
    download.cancelled.store(true, Ordering::Relaxed);
  }
}

/// Fetches `model`, returning once it is in place, cancelled, or failed:
/// its missing files, or with `again` every file, each replacing the one in
/// place only once it arrives whole, so the model keeps working meanwhile.
/// Progress goes out as change events. Asked again while it runs, it leaves
/// the first download to finish.
pub(super) async fn run(app: &AppHandle, model: &'static Model, again: bool) -> Result<(), String> {
  let cancelled = {
    let mut downloads = locked(&DOWNLOADS);
    if downloads.contains_key(model.id) {
      return Ok(());
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    downloads.insert(
      model.id,
      Download {
        cancelled: cancelled.clone(),
        progress: 0.0,
        total: 0,
      },
    );
    cancelled
  };
  let result = fetch(app, model, again, &cancelled).await;
  locked(&DOWNLOADS).remove(model.id);
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

/// A file this download fetches, given back however the fetch ends.
struct Claim(&'static str);

impl Drop for Claim {
  fn drop(&mut self) {
    locked(&FETCHING).remove(self.0);
  }
}

/// Where a model's download stands across the files it fetches.
struct Progress<'a> {
  app: &'a AppHandle,
  id: &'static str,
  /// Bytes of the files before this one.
  before: u64,
  total: u64,
  /// Per mille last told, so a change event goes out every half percent.
  shown: u64,
}

impl Progress<'_> {
  fn tell(&mut self, received: u64) {
    let per_mille = ((self.before + received) * 1000 / self.total.max(1)).min(1000);
    if per_mille < self.shown + 5 && per_mille != 0 {
      return;
    }
    self.shown = per_mille;
    if let Some(download) = locked(&DOWNLOADS).get_mut(self.id) {
      #[expect(clippy::cast_precision_loss, reason = "at most 1000")]
      let fraction = per_mille as f32 / 1000.0;
      download.progress = fraction;
      download.total = self.total;
    }
    models::changed(self.app);
  }
}

/// Downloads each file beside its place and moves it in only once its size
/// and checksum match, so a file under its own name is always whole.
async fn fetch(
  app: &AppHandle,
  model: &'static Model,
  again: bool,
  cancelled: &AtomicBool,
) -> Result<(), String> {
  let files: Vec<&'static ModelFile> = model
    .files
    .iter()
    .filter(|file| again || !models::is_whole(app, file))
    .collect();
  let mut progress = Progress {
    app,
    id: model.id,
    before: 0,
    total: files.iter().map(|file| file.size_bytes).sum(),
    shown: 0,
  };
  progress.tell(0);
  for file in files {
    let mut waited = false;
    let claim = loop {
      if cancelled.load(Ordering::Relaxed) {
        return Err("Cancelled".to_owned());
      }
      let other = {
        let mut fetching = locked(&FETCHING);
        match fetching.get(file.name) {
          Some(&received) => Some(received),
          None => {
            fetching.insert(file.name, 0);
            None
          }
        }
      };
      match other {
        None => break Claim(file.name),
        Some(received) => {
          waited = true;
          progress.tell(received);
          tokio::time::sleep(WAIT).await;
        }
      }
    };
    // A file another download has just brought in is not fetched twice,
    // even when downloading again.
    if !((waited || !again) && models::is_whole(app, file)) {
      let destination = models::file_path(app, file)?;
      if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
      }
      let partial = destination.with_extension("part");
      let result = receive(&mut progress, file, cancelled, &partial)
        .await
        .and_then(|()| std::fs::rename(&partial, &destination).map_err(|error| error.to_string()));
      if result.is_err() {
        let _ = std::fs::remove_file(&partial);
        return result;
      }
    }
    drop(claim);
    progress.before += file.size_bytes;
  }
  Ok(())
}

async fn receive(
  progress: &mut Progress<'_>,
  file: &'static ModelFile,
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
  while let Some(chunk) = response.chunk().await.map_err(failed)? {
    if cancelled.load(Ordering::Relaxed) {
      return Err("Cancelled".to_owned());
    }
    output
      .write_all(&chunk)
      .map_err(|error| format!("Could not save the model: {error}"))?;
    hasher.update(&chunk);
    received += chunk.len() as u64;
    if let Some(fetched) = locked(&FETCHING).get_mut(file.name) {
      *fetched = received;
    }
    progress.tell(received);
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
