// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fetching a file found on Openverse, telling the island how far it has
//! come. Freesound, where nearly every sound comes from, sends each
//! connection slowly (a 0.9 MB effect took 11 seconds) but not several at
//! once, so where the host takes ranges the file is fetched in parts side
//! by side (the same file in 1.3 seconds).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{CONTENT_RANGE, RANGE};
use reqwest::{Client, StatusCode};
use tauri::ipc::Channel;
use tokio::task::JoinSet;

use super::client;

/// The most a file found may weigh, in bytes: well past any song or picture
/// the library is for, short of filling the disk.
const MOST_BYTES: u64 = 100 << 20;

/// The most parts a file is fetched in at once.
const MOST_PARTS: u64 = 8;

/// The least a part holds, in bytes, so a small file is not cut into parts
/// that each cost more to ask for than to fetch.
const LEAST_PART_BYTES: u64 = 128 << 10;

/// How much of a file has come, sent on as a fraction whenever another
/// percent has.
struct Progress {
  channel: Channel<f64>,
  received: AtomicU64,
  sent_percent: AtomicU64,
  total: u64,
}

impl Progress {
  fn add(&self, bytes: u64) {
    let received = self.received.fetch_add(bytes, Ordering::Relaxed) + bytes;
    if self.total == 0 {
      return;
    }
    let percent = (received * 100 / self.total).min(100);
    // Parts arrive side by side, so only a percent past every one sent goes
    // on, and the ring never steps back.
    if self.sent_percent.fetch_max(percent, Ordering::Relaxed) < percent {
      let _ = self.channel.send(percent as f64 / 100.0);
    }
  }
}

fn too_large(url: &str) -> String {
  format!("{url} is too large to add")
}

/// The whole of `url`'s file as the host says, from the answer to a request
/// for its first byte.
fn total_of(answer: &reqwest::Response) -> Option<u64> {
  if answer.status() != StatusCode::PARTIAL_CONTENT {
    return None;
  }
  let range = answer.headers().get(CONTENT_RANGE)?.to_str().ok()?;
  range.rsplit_once('/')?.1.parse().ok()
}

/// Reads `answer` to its end into `into`, counting it towards `progress`.
async fn read_into(
  mut answer: reqwest::Response,
  into: &mut Vec<u8>,
  progress: &Progress,
) -> Result<(), reqwest::Error> {
  while let Some(chunk) = answer.chunk().await? {
    progress.add(chunk.len() as u64);
    into.extend_from_slice(&chunk);
  }
  Ok(())
}

/// Bytes `start` to `end` of `url`'s file.
async fn part(
  client: Client,
  url: String,
  start: u64,
  end: u64,
  progress: Arc<Progress>,
) -> Result<(u64, Vec<u8>), String> {
  let failed = |error: reqwest::Error| format!("Could not download {url}: {error}");
  let answer = client
    .get(&url)
    .header(RANGE, format!("bytes={start}-{end}"))
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(failed)?;
  if answer.status() != StatusCode::PARTIAL_CONTENT {
    return Err(format!("{url} did not send the part asked for"));
  }
  let mut bytes = Vec::with_capacity((end - start + 1) as usize);
  read_into(answer, &mut bytes, &progress)
    .await
    .map_err(failed)?;
  if bytes.len() as u64 != end - start + 1 {
    return Err(format!("{url} sent a part short"));
  }
  Ok((start, bytes))
}

/// The file at `url`, which must be on the web and no heavier than
/// `MOST_BYTES`, telling `channel` how far it has come.
pub(super) async fn download(url: &str, channel: Channel<f64>) -> Result<Vec<u8>, String> {
  if !url.starts_with("https://") && !url.starts_with("http://") {
    return Err(format!("{url} is not on the web"));
  }
  let failed = |error: reqwest::Error| format!("Could not download {url}: {error}");
  let client = client(Duration::from_secs(120))?;
  // Asked for its first byte, a host that takes ranges says how large the
  // whole is; one that does not sends the whole, which is read as it is.
  let first = client
    .get(url)
    .header(RANGE, "bytes=0-0")
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(failed)?;
  let Some(total) = total_of(&first) else {
    let total = first.content_length().unwrap_or(0);
    if total > MOST_BYTES {
      return Err(too_large(url));
    }
    let progress = progress_of(channel, total);
    let mut bytes = Vec::with_capacity(total as usize);
    read_into(first, &mut bytes, &progress)
      .await
      .map_err(failed)?;
    if bytes.len() as u64 > MOST_BYTES {
      return Err(too_large(url));
    }
    return Ok(bytes);
  };
  if total > MOST_BYTES {
    return Err(too_large(url));
  }
  let progress = Arc::new(progress_of(channel, total));
  let parts = total.div_ceil(LEAST_PART_BYTES).clamp(1, MOST_PARTS);
  let part_bytes = total.div_ceil(parts);
  let mut fetching = JoinSet::new();
  for index in 0..parts {
    let start = index * part_bytes;
    let end = (start + part_bytes).min(total) - 1;
    fetching.spawn(part(
      client.clone(),
      url.to_owned(),
      start,
      end,
      progress.clone(),
    ));
  }
  let mut whole = vec![0; total as usize];
  while let Some(fetched) = fetching.join_next().await {
    let (start, bytes) = fetched.map_err(|error| error.to_string())??;
    whole[start as usize..start as usize + bytes.len()].copy_from_slice(&bytes);
  }
  Ok(whole)
}

fn progress_of(channel: Channel<f64>, total: u64) -> Progress {
  Progress {
    channel,
    received: AtomicU64::new(0),
    sent_percent: AtomicU64::new(0),
    total,
  }
}
