// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A tool shown at work inside a window, for a page that shows what the
//! tool does: the Ruler, OCR or Annotate drawn by its own overlay renderer
//! over a picture the page paints, driven by pointer input the page sends.
//! The page keeps the drawing's placement current, and leaves a transparent
//! box where a tool that draws its own screen shows through.

mod annotate;
mod macos;
mod worker;

use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::sync::{LazyLock, Mutex};
use std::thread::JoinHandle;

use dispatch2::MainThreadBound;
use screenwide_imaging::CapturedImage;
use screenwide_osc::geometry::{Point, Size};
use serde::Deserialize;
use tauri::ipc::{InvokeBody, Request};
use tauri::WebviewWindow;

use macos::Slot;
use worker::{Message, Setup};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PreviewTool {
  Ruler,
  TextRecognition,
  Annotate,
}

impl PreviewTool {
  /// Annotate draws on glass over the screen, as the overlay does; the
  /// others draw the screen themselves and sit under the page.
  fn over_page(self) -> bool {
    self == Self::Annotate
  }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PreviewInput {
  Hover,
  Down,
  Drag,
  Up,
  /// Back to how the tool starts: no measurements, no area chosen, nothing
  /// drawn.
  Reset,
  /// Annotate: what the next press draws.
  PickArrow,
  PickCounter,
  /// Annotate: the session ends, and the next starts.
  Exit,
  Enter,
}

/// A rectangle in the page's points.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(crate) struct Area {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

/// A scroll area's fade at its ends, which the page draws with a CSS mask
/// the layer under it cannot see.
#[derive(Clone, Copy, Debug, Deserialize)]
pub(crate) struct Fade {
  /// The scroll area's viewport.
  pub area: Area,
  /// Fades along the vertical axis, rather than the horizontal.
  pub vertical: bool,
  /// Points faded at the top or left.
  pub start: f64,
  /// Points faded at the bottom or right.
  pub end: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Placement {
  /// The page's box the picture shows in.
  pub frame: Area,
  /// Where the picture is, wider than the box and centred on it, so a
  /// resize moves it rather than redrawing it.
  pub picture: Area,
  /// The part of the page it may be seen in.
  pub clip: Area,
  pub fade: Option<Fade>,
  /// How round the box's corners are, in points.
  pub radius: f64,
  pub visible: bool,
  /// Physical pixels per point, from the window rather than the page.
  #[serde(skip)]
  pub scale: f64,
}

impl Placement {
  fn hidden() -> Self {
    Self {
      frame: Area::default(),
      picture: Area::default(),
      clip: Area::default(),
      fade: None,
      radius: 0.0,
      visible: false,
      scale: 1.0,
    }
  }
}

struct Preview {
  slot: Option<MainThreadBound<Slot>>,
  messages: Sender<Message>,
  worker: Option<JoinHandle<()>>,
}

impl Drop for Preview {
  /// The worker draws into the slot's layer, so it stops before the layer
  /// goes.
  fn drop(&mut self) {
    let _ = self.messages.send(Message::Stop);
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
    drop(self.slot.take());
  }
}

/// Every open preview, by window label and the page's id for it.
static PREVIEWS: LazyLock<Mutex<HashMap<(String, String), Preview>>> =
  LazyLock::new(Default::default);

fn key(window: &WebviewWindow, id: &str) -> (String, String) {
  (window.label().to_owned(), id.to_owned())
}

fn header<'a>(request: &'a Request<'_>, name: &str) -> Result<&'a str, String> {
  request
    .headers()
    .get(name)
    .and_then(|value| value.to_str().ok())
    .ok_or_else(|| format!("The tool preview request has no {name} header"))
}

fn number<T: std::str::FromStr>(request: &Request<'_>, name: &str) -> Result<T, String> {
  header(request, name)?
    .parse()
    .map_err(|_| format!("The tool preview's {name} header is not a number"))
}

/// Opens a preview of `x-preview-tool` with the id `x-preview-id`, on the
/// picture in the body: RGBA, `x-picture-width` by `x-picture-height`
/// pixels, shown at `x-size-width` by `x-size-height` points. It stays
/// hidden until placed.
#[tauri::command]
pub async fn open_tool_preview(window: WebviewWindow, request: Request<'_>) -> Result<(), String> {
  let id = header(&request, "x-preview-id")?.to_owned();
  let tool: PreviewTool = serde_json::from_value(serde_json::Value::String(
    header(&request, "x-preview-tool")?.to_owned(),
  ))
  .map_err(|error| error.to_string())?;
  let width: u32 = number(&request, "x-picture-width")?;
  let height: u32 = number(&request, "x-picture-height")?;
  let size = Size {
    width: number(&request, "x-size-width")?,
    height: number(&request, "x-size-height")?,
  };
  let InvokeBody::Raw(rgba) = request.body() else {
    return Err("The tool preview's picture is not raw bytes".to_owned());
  };
  if width == 0 || height == 0 || rgba.len() != width as usize * height as usize * 4 {
    return Err("The tool preview's picture does not match its size".to_owned());
  }
  let picture = CapturedImage {
    rgba: rgba.clone(),
    width,
    height,
  };
  let scale = window.scale_factor().map_err(|error| error.to_string())?;
  let (slot, drawable) = macos::create(&window, scale, tool.over_page())?;
  let (messages, receiver) = mpsc::channel();
  let setup = Setup {
    tool,
    picture,
    size,
    drawable,
  };
  let worker = std::thread::Builder::new()
    .name("tool-preview".to_owned())
    .spawn(move || worker::run(setup, receiver))
    .map_err(|error| error.to_string())?;
  let preview = Preview {
    slot: Some(slot),
    messages,
    worker: Some(worker),
  };
  let replaced = PREVIEWS
    .lock()
    .map_err(|error| error.to_string())?
    .insert(key(&window, &id), preview);
  drop(replaced);
  Ok(())
}

/// Moves, clips, rounds, shows or hides a preview.
#[tauri::command]
pub fn place_tool_preview(
  window: WebviewWindow,
  id: String,
  mut placement: Placement,
) -> Result<(), String> {
  placement.scale = window.scale_factor().map_err(|error| error.to_string())?;
  let previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
  let Some(slot) = previews
    .get(&key(&window, &id))
    .and_then(|preview| preview.slot.as_ref())
  else {
    return Ok(());
  };
  dispatch2::run_on_main(|mtm| slot.get(mtm).place(placement));
  Ok(())
}

/// Pointer input at (`x`, `y`), in the picture's points.
#[tauri::command]
pub fn tool_preview_input(
  window: WebviewWindow,
  id: String,
  input: PreviewInput,
  x: f64,
  y: f64,
) -> Result<(), String> {
  let previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
  if let Some(preview) = previews.get(&key(&window, &id)) {
    let _ = preview.messages.send(Message::Input(input, Point { x, y }));
  }
  Ok(())
}

/// Annotate's colour, `#rrggbb`, and whether its annotations outlast the
/// session, as the page's settings have them.
#[tauri::command]
pub fn set_tool_preview_look(
  window: WebviewWindow,
  id: String,
  color: String,
  keep: bool,
) -> Result<(), String> {
  let previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
  if let Some(preview) = previews.get(&key(&window, &id)) {
    let _ = preview.messages.send(Message::Look { color, keep });
  }
  Ok(())
}

#[tauri::command]
pub fn close_tool_preview(window: WebviewWindow, id: String) -> Result<(), String> {
  let closed = PREVIEWS
    .lock()
    .map_err(|error| error.to_string())?
    .remove(&key(&window, &id));
  drop(closed);
  Ok(())
}

/// Closes every preview in the window, such as those a page left behind
/// when it was loaded again.
#[tauri::command]
pub fn close_tool_previews(window: WebviewWindow) -> Result<(), String> {
  let closed: Vec<Preview> = {
    let mut previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
    let keys: Vec<_> = previews
      .keys()
      .filter(|(label, _)| label == window.label())
      .cloned()
      .collect();
    keys.iter().filter_map(|key| previews.remove(key)).collect()
  };
  drop(closed);
  Ok(())
}
