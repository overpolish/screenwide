// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scene as the native surface reaches it. Every entry takes the scene
//! pointer the surface was created with, which the surface releases when it
//! is deallocated, after the last block that could use it.

use std::ffi::c_void;
use std::sync::Arc;

use crate::editor::preview_platform::workspace_backdrop::BackdropStyle;
use crate::editor::preview_platform::workspace_scene::{
  FrameChange, NativeWorkspacePlacement, WorkspaceScene,
};
use screenwide_osc::gpu::magnifier::NativeMagnifier;

/// Where the overlays drawn over the picture show, and the caption's words.
mod overlays;

/// # Safety
/// `scene` is null or a pointer the surface was handed by `Arc::into_raw`.
unsafe fn scene<'a>(scene: *const c_void) -> Option<&'a WorkspaceScene> {
  unsafe { scene.cast::<WorkspaceScene>().as_ref() }
}

fn status(result: bool) -> i32 {
  i32::from(result)
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_release(scene: *const c_void) {
  if !scene.is_null() {
    drop(unsafe { Arc::from_raw(scene.cast::<WorkspaceScene>()) });
  }
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_ready(scene: *const c_void, count: u32) -> i32 {
  status(unsafe { self::scene(scene) }.is_some_and(|scene| scene.ready(count as usize)))
}

/// Draws a caret and selection into the text box at `index` in layer
/// `layer`'s list, from byte `start` to byte `end` of its text, the caret
/// shown where `caret` is non-zero; with `typing` zero, into no box. Answers
/// whether that changes what is drawn.
#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_set_typing(
  scene: *const c_void,
  typing: i32,
  layer: u32,
  index: u32,
  start: u32,
  end: u32,
  caret: i32,
) -> i32 {
  let marks = (typing != 0).then_some((
    layer,
    index as usize,
    screenwide_text::engine::typing::TypingMarks {
      start: start as usize,
      end: end as usize,
      caret: caret != 0,
    },
  ));
  status(unsafe { self::scene(scene) }.is_some_and(|scene| scene.set_typing(marks)))
}

/// Draws the scene into `texture`, a BGRA `id<MTLTexture>` the size of the
/// workspace drawable, with one placement for each layer, in drawable pixels,
/// over a checkerboard sized for `scale` drawable pixels a point and coloured
/// for the light appearance where `light` is non-zero. Non-zero `reuse` shows
/// the picture last composed again where the scene has not changed since:
/// the redraw is for the selection chrome alone.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_workspace_scene_draw(
  scene: *const c_void,
  texture: *mut c_void,
  placements: *const NativeWorkspacePlacement,
  count: u32,
  magnifier: *const NativeMagnifier,
  scale: f64,
  light: i32,
  reuse: i32,
) -> i32 {
  let Some(scene) = (unsafe { self::scene(scene) }) else {
    return 0;
  };
  if placements.is_null() || count == 0 {
    return 0;
  }
  let placements = unsafe { std::slice::from_raw_parts(placements, count as usize) };
  let magnifier = unsafe { magnifier.as_ref() };
  let backdrop = BackdropStyle {
    scale,
    light: light != 0,
  };
  let drawn = screenwide_gpu::shared()
    .and_then(|gpu| unsafe { screenwide_gpu::macos::bgra_render_target(gpu, texture) })
    .and_then(|target| scene.draw(&target, placements, magnifier, backdrop, reuse != 0));
  match drawn {
    Ok(()) => 1,
    Err(error) => {
      eprintln!("The editor workspace could not be drawn: {error}");
      0
    }
  }
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_begin_resize(scene: *const c_void) -> i32 {
  status(unsafe { self::scene(scene) }.is_some_and(WorkspaceScene::begin_resize))
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_end_resize(scene: *const c_void, commit: i32) {
  if let Some(scene) = unsafe { self::scene(scene) } {
    scene.end_resize(commit != 0);
  }
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_update_resize(
  scene: *const c_void,
  origin_x: f64,
  origin_y: f64,
  width: f64,
  height: f64,
) -> i32 {
  let change = FrameChange {
    origin: (origin_x, origin_y),
    size: (width, height),
    moved: (0.0, 0.0),
  };
  status(unsafe { self::scene(scene) }.is_some_and(|scene| scene.update_resize(None, change)))
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_workspace_scene_update_auto_fit_move(
  scene: *const c_void,
  selected_layer: u32,
  move_x: f64,
  move_y: f64,
  origin_x: f64,
  origin_y: f64,
  width: f64,
  height: f64,
) -> i32 {
  let change = FrameChange {
    origin: (origin_x, origin_y),
    size: (width, height),
    moved: (move_x, move_y),
  };
  status(
    unsafe { self::scene(scene) }
      .is_some_and(|scene| scene.update_resize(Some(selected_layer), change)),
  )
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_workspace_scene_update_recording_auto_fit_move(
  scene: *const c_void,
  selected_pane: u32,
  move_x: f64,
  move_y: f64,
  origin_x: f64,
  origin_y: f64,
  width: f64,
  height: f64,
) -> i32 {
  let change = FrameChange {
    origin: (origin_x, origin_y),
    size: (width, height),
    moved: (move_x, move_y),
  };
  status(
    unsafe { self::scene(scene) }
      .is_some_and(|scene| scene.update_recording_auto_fit_move(selected_pane, change)),
  )
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_update_selected_resize(
  scene: *const c_void,
  selected_pane: u32,
  origin_x: f64,
  origin_y: f64,
  width: f64,
  height: f64,
) -> i32 {
  let change = FrameChange {
    origin: (origin_x, origin_y),
    size: (width, height),
    moved: (0.0, 0.0),
  };
  status(
    unsafe { self::scene(scene) }
      .is_some_and(|scene| scene.update_selected_resize(selected_pane, change)),
  )
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_update_selected_radius(
  scene: *const c_void,
  selected_pane: u32,
  radius_percent: f64,
  frame: i32,
) -> i32 {
  status(
    unsafe { self::scene(scene) }
      .is_some_and(|scene| scene.update_selected_radius(selected_pane, radius_percent, frame != 0)),
  )
}

/// Copies `rgba`, straight RGBA `width` by `height`, into `texture`, a BGRA
/// drawable's `id<MTLTexture>` of that size.
#[no_mangle]
pub unsafe extern "C" fn screenwide_preview_upload_rgba(
  texture: *mut c_void,
  rgba: *const u8,
  width: u32,
  height: u32,
) -> i32 {
  if rgba.is_null() || width == 0 || height == 0 {
    return 0;
  }
  let length = width as usize * height as usize * 4;
  let rgba = unsafe { std::slice::from_raw_parts(rgba, length) };
  let uploaded = screenwide_gpu::shared().and_then(|gpu| {
    let target = unsafe { screenwide_gpu::macos::bgra_render_target(gpu, texture) }?;
    if (target.width(), target.height()) != (width, height) {
      return Err("The drawable is not the frame's size".to_owned());
    }
    let mut bgra = rgba.to_vec();
    for pixel in bgra.as_chunks_mut::<4>().0 {
      pixel.swap(0, 2);
    }
    gpu.queue.write_texture(
      target.as_image_copy(),
      &bgra,
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(width * 4),
        rows_per_image: Some(height),
      },
      target.size(),
    );
    gpu.queue.submit([]);
    Ok(())
  });
  match uploaded {
    Ok(()) => 1,
    Err(error) => {
      eprintln!("The preview frame could not be uploaded: {error}");
      0
    }
  }
}
