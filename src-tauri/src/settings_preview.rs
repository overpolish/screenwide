// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A window's live pictures of what a setting does (`screenwide_settings_
//! preview`), wired to the page: the tools drawn by their own overlay
//! renderers here, the recordings composed by the crate. The page paints
//! the pictures, keeps each preview's placement current and drives a tool
//! with its pointer.

mod annotate;
mod tools;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use screenwide_imaging::CapturedImage;
use screenwide_osc::geometry::Size;
use screenwide_settings_preview::placement::PreviewPlacement;
use screenwide_settings_preview::scene::{PreviewInput, PreviewScene};
use screenwide_settings_preview::{HostWindow, Message, Preview, Renderer, Stacking};
use tauri::ipc::{InvokeBody, Request};
use tauri::WebviewWindow;

struct Open {
  preview: Preview,
  scene: PreviewScene,
}

/// Every open preview, by window label and the page's id for it.
static PREVIEWS: LazyLock<Mutex<HashMap<(String, String), Open>>> = LazyLock::new(Default::default);

fn key(window: &WebviewWindow, id: &str) -> (String, String) {
  (window.label().to_owned(), id.to_owned())
}

fn header<'a>(request: &'a Request<'_>, name: &str) -> Result<&'a str, String> {
  request
    .headers()
    .get(name)
    .and_then(|value| value.to_str().ok())
    .ok_or_else(|| format!("The settings preview request has no {name} header"))
}

fn number<T: std::str::FromStr>(request: &Request<'_>, name: &str) -> Result<T, String> {
  header(request, name)?
    .parse()
    .map_err(|_| format!("The settings preview's {name} header is not a number"))
}

#[cfg(target_os = "macos")]
fn host_window(window: &WebviewWindow) -> Result<HostWindow, String> {
  Ok(HostWindow {
    content_view: window.ns_view().map_err(|error| error.to_string())? as usize,
    find_page: crate::app_windows::platform::webview_view,
  })
}

#[cfg(target_os = "windows")]
fn host_window(window: &WebviewWindow) -> Result<HostWindow, String> {
  Ok(HostWindow {
    hwnd: window.hwnd().map_err(|error| error.to_string())?.0 as isize,
  })
}

/// The body's pictures: `count` of them, RGBA, each `width` by `height`.
fn pictures(request: &Request<'_>) -> Result<Vec<CapturedImage>, String> {
  let width: u32 = number(request, "x-picture-width")?;
  let height: u32 = number(request, "x-picture-height")?;
  let count: usize = number(request, "x-picture-count")?;
  let InvokeBody::Raw(rgba) = request.body() else {
    return Err("The settings preview's pictures are not raw bytes".to_owned());
  };
  let length = width as usize * height as usize * 4;
  if length == 0 || count == 0 || rgba.len() != length * count {
    return Err("The settings preview's pictures do not match their size".to_owned());
  }
  Ok(
    rgba
      .chunks_exact(length)
      .map(|rgba| CapturedImage {
        rgba: rgba.to_vec(),
        width,
        height,
      })
      .collect(),
  )
}

/// Opens a preview of the scene in `x-preview-scene`, as JSON, with the id
/// `x-preview-id`, on the pictures in the body: `x-picture-count` of them,
/// RGBA, `x-picture-width` by `x-picture-height` pixels, shown at
/// `x-size-width` by `x-size-height` points. It stays hidden until placed.
#[tauri::command]
pub async fn open_settings_preview(
  window: WebviewWindow,
  request: Request<'_>,
) -> Result<(), String> {
  let id = header(&request, "x-preview-id")?.to_owned();
  let scene: PreviewScene = serde_json::from_str(header(&request, "x-preview-scene")?)
    .map_err(|error| error.to_string())?;
  let size = Size {
    width: number(&request, "x-size-width")?,
    height: number(&request, "x-size-height")?,
  };
  let mut pictures = pictures(&request)?;
  let host = host_window(&window)?;
  let preview = if scene.is_tool() {
    let picture = pictures.swap_remove(0);
    let tool = scene.clone();
    let stacking = if scene.over_page() {
      Stacking::Above
    } else {
      Stacking::Below
    };
    Preview::open(&host, stacking, move |drawable| {
      tool_renderer(tool, picture, size, drawable)
    })?
  } else {
    Preview::open_recording(&host, scene.clone(), pictures, (size.width, size.height))?
  };
  let replaced = PREVIEWS
    .lock()
    .map_err(|error| error.to_string())?
    .insert(key(&window, &id), Open { preview, scene });
  drop(replaced);
  Ok(())
}

/// How big the page's desktop for a tool is beside the one it stands for.
/// The page paints a small window on a strip of desktop, so the tools, drawn
/// at their real sizes, dwarfed it: Annotate's arrow and counter at the
/// sizes the person set came out as big as the window. The tool works on a
/// desktop this much bigger instead, so everything it draws, and what the
/// Ruler measures, comes out at the picture's scale.
const TOOL_SCALE: f64 = 0.5;

/// A length in the page's picture as the tool measures it.
pub(crate) fn tool_points(points: f64) -> f64 {
  points / TOOL_SCALE
}

fn tool_renderer(
  scene: PreviewScene,
  picture: CapturedImage,
  size: Size,
  drawable: screenwide_gpu::surface::Surface,
) -> Result<Box<dyn Renderer>, String> {
  let size = Size {
    width: tool_points(size.width),
    height: tool_points(size.height),
  };
  if let Some(look) = annotate::look(scene.clone()) {
    // The page paints the screen under it; only the glass is drawn here.
    let pixels = (picture.width, picture.height);
    let scale = f64::from(picture.width) / size.width.max(1.0);
    return Ok(Box::new(annotate::AnnotateRenderer::new(
      drawable, pixels, scale, look,
    )?));
  }
  Ok(Box::new(tools::ToolRenderer::new(
    scene == PreviewScene::Ruler,
    picture,
    size,
    drawable,
  )?))
}

/// Moves, clips, rounds, shows or hides a preview.
#[tauri::command]
pub fn place_settings_preview(
  window: WebviewWindow,
  id: String,
  mut placement: PreviewPlacement,
) -> Result<(), String> {
  placement.scale = window.scale_factor().map_err(|error| error.to_string())?;
  let mut previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
  if let Some(open) = previews.get_mut(&key(&window, &id)) {
    open.preview.place(placement);
  }
  Ok(())
}

/// A tool's pointer at (`x`, `y`), in the picture's points.
#[tauri::command]
pub fn settings_preview_input(
  window: WebviewWindow,
  id: String,
  input: PreviewInput,
  x: f64,
  y: f64,
) -> Result<(), String> {
  let previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
  if let Some(open) = previews.get(&key(&window, &id)) {
    open.preview.send(Message::Input {
      input,
      x: tool_points(x),
      y: tool_points(y),
    });
  }
  Ok(())
}

/// The preview's scene with other values, such as Annotate's colour or a
/// frame rate; another kind of scene needs a preview of its own.
#[tauri::command]
pub fn set_settings_preview_scene(
  window: WebviewWindow,
  id: String,
  scene: PreviewScene,
) -> Result<(), String> {
  let mut previews = PREVIEWS.lock().map_err(|error| error.to_string())?;
  let Some(open) = previews.get_mut(&key(&window, &id)) else {
    return Ok(());
  };
  if !open.scene.same_kind(&scene) {
    return Err("A settings preview can only take values for its own scene".to_owned());
  }
  open.scene = scene.clone();
  open.preview.send(Message::Scene(scene));
  Ok(())
}

#[tauri::command]
pub fn close_settings_preview(window: WebviewWindow, id: String) -> Result<(), String> {
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
pub fn close_settings_previews(window: WebviewWindow) -> Result<(), String> {
  close_window_previews(window.label());
  Ok(())
}

/// Closes every preview in the window `label`, for a window destroyed with
/// them open: its page can no longer close them, and each holds the device.
pub(crate) fn close_window_previews(label: &str) {
  let closed: Vec<Open> = {
    let Ok(mut previews) = PREVIEWS.lock() else {
      return;
    };
    let keys: Vec<_> = previews
      .keys()
      .filter(|(held, _)| held == label)
      .cloned()
      .collect();
    keys.iter().filter_map(|key| previews.remove(key)).collect()
  };
  drop(closed);
}
