// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A picture dragged from anywhere with Option held and dropped, kept as a
//! reference: no shortcut, no area drawn. While such a drag is under way,
//! each platform lays a catcher over its displays and the hint island comes
//! out (`drop_island`); what lands is opened as a reference centred where
//! it was dropped, its pixels at that display's scale, so a picture a web
//! page draws at twice its size in points, as most do on a sharp display,
//! comes out the size the page showed it.

use std::path::PathBuf;
use std::time::Duration;

use screenwide_imaging::CapturedImage;
use tauri::{AppHandle, LogicalPosition, LogicalSize, PhysicalPosition, Position};

use super::{Placement, ReferencePicture};

/// One form a dragged picture came in. A drag carries several; they are
/// tried in the order given, the first that opens wins.
pub(crate) enum Dropped {
  /// The picture's own bytes, in any format the `image` crate reads.
  Data(Vec<u8>),
  /// A picture file.
  File(PathBuf),
  /// Where the picture lives on the web.
  Address(String),
}

/// The file extensions a dragged file or address is taken as a picture by.
const PICTURE_EXTENSIONS: [&str; 9] = [
  "png", "jpg", "jpeg", "gif", "webp", "tif", "tiff", "bmp", "heic",
];

pub(super) fn is_picture_path(path: &str) -> bool {
  let path = path.split(['?', '#']).next().unwrap_or(path);
  path.rsplit_once('.').is_some_and(|(_, extension)| {
    PICTURE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
  })
}

/// Whether `address` is a picture's on the web.
pub(super) fn is_picture_address(address: &str) -> bool {
  address.starts_with("http") && is_picture_path(address)
}

/// Keeps the first of `forms` that opens as a reference centred on `at`, on
/// a display of `scale` pixels to the point.
pub(crate) fn keep(app: &AppHandle, forms: Vec<Dropped>, at: Position, scale: f64) {
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    let mut failures = Vec::new();
    for form in forms {
      match load(form).await {
        Ok(image) => {
          if let Err(error) = open(&app, image, at, scale).await {
            eprintln!("Could not keep the dropped picture as a reference: {error}");
          }
          return;
        }
        Err(error) => failures.push(error),
      }
    }
    eprintln!(
      "Nothing dropped opened as a picture: {}",
      failures.join("; ")
    );
  });
}

async fn open(
  app: &AppHandle,
  image: CapturedImage,
  at: Position,
  scale: f64,
) -> Result<(), String> {
  let scale = scale.max(1.0);
  let size = LogicalSize::new(
    f64::from(image.width) / scale,
    f64::from(image.height) / scale,
  );
  let origin = match at {
    Position::Logical(at) => Position::Logical(LogicalPosition::new(
      (at.x - size.width / 2.0).round(),
      (at.y - size.height / 2.0).round(),
    )),
    // The picture's pixels are the display's own.
    Position::Physical(at) => Position::Physical(PhysicalPosition::new(
      at.x - (image.width / 2) as i32,
      at.y - (image.height / 2) as i32,
    )),
  };
  let placement = Placement {
    origin: Some(origin),
    returns: false,
    size,
  };
  super::open(
    app,
    ReferencePicture {
      original: image,
      shown: None,
      annotations: Vec::new(),
      scale,
    },
    placement,
  )
  .await
}

async fn load(form: Dropped) -> Result<CapturedImage, String> {
  let bytes = match form {
    Dropped::Data(bytes) => bytes,
    Dropped::Address(address) => download(&address).await?,
    Dropped::File(path) => {
      return blocking(move || {
        let bytes = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        decode(&bytes)
      })
      .await;
    }
  };
  blocking(move || decode(&bytes)).await
}

/// Runs `work`, which reads and decodes, off the async threads.
async fn blocking(
  work: impl FnOnce() -> Result<CapturedImage, String> + Send + 'static,
) -> Result<CapturedImage, String> {
  tauri::async_runtime::spawn_blocking(work)
    .await
    .map_err(|error| error.to_string())?
}

fn decode(bytes: &[u8]) -> Result<CapturedImage, String> {
  let image = image::load_from_memory(bytes)
    .map_err(|error| error.to_string())?
    .into_rgba8();
  Ok(CapturedImage {
    width: image.width(),
    height: image.height(),
    rgba: image.into_raw(),
  })
}

/// The picture at `address`, which must be on the web.
async fn download(address: &str) -> Result<Vec<u8>, String> {
  if !(address.starts_with("https://") || address.starts_with("http://")) {
    return Err(format!("{address} is not on the web"));
  }
  // reqwest is built without a TLS provider of its own; the updater and the
  // model downloads install the same one, whichever runs first.
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }
  let failed = |error: reqwest::Error| format!("Could not download {address}: {error}");
  reqwest::Client::builder()
    .user_agent(concat!("Screenwide/", env!("CARGO_PKG_VERSION")))
    .connect_timeout(Duration::from_secs(15))
    .timeout(Duration::from_secs(60))
    .build()
    .map_err(failed)?
    .get(address)
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(failed)?
    .bytes()
    .await
    .map(|bytes| bytes.to_vec())
    .map_err(failed)
}
