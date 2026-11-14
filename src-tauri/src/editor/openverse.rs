// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Songs, sound effects and pictures found on Openverse, for the Library
//! island: searched through Screenwide's Worker (`screenwide_openverse`) and
//! added to the library with who made them, as though added from a file, so
//! the library and the timeline treat them as any other. Each is looked up
//! again as it is added, so one whose licence changed since it was found is
//! not, and it is credited as it stands now. The island streams a sound to
//! hear it before it is added, straight from where it is published.

mod download;

use std::sync::OnceLock;
use std::time::Duration;

use screenwide_images::store::IMAGE_PREFIX;
use screenwide_openverse::{
  install_id, page_from, search_url, work_from, work_url, Failure, Found, Kind, MusicFilters, Page,
  WorkFailure, INSTALL_HEADER, WORKER_URL,
};
use screenwide_sound_track::store::{self as sounds, Sound};
use screenwide_sound_track::AudioClipKind;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

use download::download;

/// Where the app asks: the Worker, or another while it is worked on locally
/// (`SCREENWIDE_OPENVERSE_URL`, such as `wrangler dev`'s address).
fn worker_base() -> String {
  std::env::var("SCREENWIDE_OPENVERSE_URL").unwrap_or_else(|_| WORKER_URL.to_owned())
}

fn client(timeout: Duration) -> Result<reqwest::Client, String> {
  // reqwest is built without a TLS provider of its own; the updater and the
  // model downloads install the same one, whichever runs first.
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }
  reqwest::Client::builder()
    .user_agent(concat!("Screenwide/", env!("CARGO_PKG_VERSION")))
    .connect_timeout(Duration::from_secs(15))
    .timeout(timeout)
    .build()
    .map_err(|error| error.to_string())
}

/// A question for the Worker at `url`, naming this installation
/// (`screenwide_openverse::install_id`). Only the Worker is told it: a file
/// is fetched from where it is published without it.
fn ask_worker(app: &AppHandle, url: String) -> Result<reqwest::RequestBuilder, String> {
  static INSTALL: OnceLock<Option<String>> = OnceLock::new();
  let install = INSTALL.get_or_init(|| install_id(&app.path().app_data_dir().ok()?));
  let request = client(Duration::from_secs(30))?.get(url);
  Ok(match install {
    Some(id) => request.header(INSTALL_HEADER, id),
    None => request,
  })
}

/// `page` of what Openverse holds of `kind` for `query`, music narrowed by
/// `filters`.
#[tauri::command]
pub async fn search_openverse(
  app: AppHandle,
  kind: Kind,
  query: String,
  filters: MusicFilters,
  page: u32,
) -> Result<Page, Failure> {
  let answer = ask_worker(
    &app,
    search_url(&worker_base(), kind, &query, &filters, page),
  )
  .map_err(|_| Failure::Unavailable)?
  .send()
  .await
  .map_err(|error| {
    eprintln!("Could not search Openverse: {error}");
    Failure::Unavailable
  })?;
  let status = answer.status().as_u16();
  let body = answer.bytes().await.map_err(|_| Failure::Unavailable)?;
  page_from(status, &body)
}

/// `found` as it stands now, looked up again: refused with `withdrawn`
/// where it is gone or no longer under a licence the app may use, which the
/// island says.
async fn as_it_stands(app: &AppHandle, kind: Kind, found: &Found) -> Result<Found, String> {
  let answer = ask_worker(app, work_url(&worker_base(), kind, &found.id))?
    .send()
    .await
    .map_err(|error| format!("Could not look {} up again: {error}", found.id))?;
  let status = answer.status().as_u16();
  let body = answer.bytes().await.map_err(|error| error.to_string())?;
  work_from(status, &body).map_err(|failure| {
    match failure {
      WorkFailure::Withdrawn => "withdrawn",
      WorkFailure::Busy => "busy",
      WorkFailure::Unavailable => "unavailable",
    }
    .to_owned()
  })
}

/// The kind of file `found` is, from what Openverse says or its address.
fn extension_of(found: &Found) -> String {
  found
    .filetype
    .clone()
    .or_else(|| {
      let path = found.url.split(['?', '#']).next()?;
      path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_owned())
    })
    .unwrap_or_else(|| "mp3".to_owned())
    .to_ascii_lowercase()
}

/// Fetches the song or sound effect `found` on Openverse into the library,
/// listed as `kind`, credited to who made it, telling `on_progress` how far
/// it has come, and says what it is.
#[tauri::command]
pub async fn add_openverse_sound(
  app: AppHandle,
  found: Found,
  kind: AudioClipKind,
  on_progress: Channel<f64>,
) -> Result<Sound, String> {
  let found = as_it_stands(
    &app,
    match kind {
      AudioClipKind::Music => Kind::Music,
      AudioClipKind::Effect => Kind::Effect,
    },
    &found,
  )
  .await?;
  let bytes = download(&found.url, on_progress).await?;
  let sound = tauri::async_runtime::spawn_blocking(move || {
    let file = std::env::temp_dir().join(format!(
      "screenwide-openverse-{}.{}",
      found
        .id
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', ""),
      extension_of(&found)
    ));
    std::fs::write(&file, &bytes).map_err(|error| error.to_string())?;
    let added = sounds::import_found(&file, &found.title, found.credit(), found.tags.clone())
      .and_then(|sound| {
        sounds::set_kind(&sound.name, kind)?;
        Ok(Sound {
          kind: Some(kind),
          ..sound
        })
      });
    let _ = std::fs::remove_file(&file);
    added
  })
  .await
  .map_err(|error| error.to_string())??;
  let _ = app.emit(super::sound_library::LIBRARY_EVENT, ());
  Ok(sound)
}

/// Fetches the picture `found` on Openverse into the library, credited to
/// who made it, telling `on_progress` how far it has come, and says the
/// library's name for it.
#[tauri::command]
pub async fn add_openverse_picture(
  app: AppHandle,
  found: Found,
  on_progress: Channel<f64>,
) -> Result<String, String> {
  let found = as_it_stands(&app, Kind::Image, &found).await?;
  let bytes = download(&found.url, on_progress).await?;
  let art = tauri::async_runtime::spawn_blocking(move || {
    screenwide_images::store::import_found(&bytes, &found.title, found.credit())
  })
  .await
  .map_err(|error| error.to_string())??;
  super::images::library::changed(&app);
  Ok(
    art
      .asset
      .strip_prefix(IMAGE_PREFIX)
      .unwrap_or(&art.asset)
      .to_owned(),
  )
}
