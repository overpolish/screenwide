// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scene as the native surface reaches it. Every entry takes the scene
//! pointer the surface was created with, which the surface releases when it
//! is deallocated, after the last block that could use it.

use std::ffi::c_void;
use std::sync::Arc;

use super::gestures::FrameChange;
use super::loupe::NativeMagnifier;
use super::WorkspaceScene;
use crate::editor::preview_platform::surface_macos::NativeWorkspacePlacement;

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

/// Draws the scene into `texture`, the workspace drawable's `id<MTLTexture>`,
/// with one placement for each layer, in drawable pixels.
#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_draw(
  scene: *const c_void,
  texture: *mut c_void,
  placements: *const NativeWorkspacePlacement,
  count: u32,
  magnifier: *const NativeMagnifier,
) -> i32 {
  let Some(scene) = (unsafe { self::scene(scene) }) else {
    return 0;
  };
  if placements.is_null() || count == 0 {
    return 0;
  }
  let placements = unsafe { std::slice::from_raw_parts(placements, count as usize) };
  let magnifier = unsafe { magnifier.as_ref() };
  let drawn = crate::gpu::shared()
    .and_then(|gpu| unsafe { crate::gpu::macos::bgra_render_target(gpu, texture) })
    .and_then(|target| scene.draw(&target, placements, magnifier));
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

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_update_keyboard(
  scene: *const c_void,
  pane_index: u32,
  center_x: f64,
  center_y: f64,
  scale_ratio: f64,
) -> i32 {
  status(
    unsafe { self::scene(scene) }
      .is_some_and(|scene| scene.update_keyboard(pane_index, (center_x, center_y), scale_ratio)),
  )
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_keyboard_bounds(
  scene: *const c_void,
  pane_index: u32,
  x: *mut f64,
  y: *mut f64,
  width: *mut f64,
  height: *mut f64,
) -> i32 {
  let Some([left, top, wide, tall]) =
    unsafe { self::scene(scene) }.and_then(|scene| scene.keyboard_bounds(pane_index))
  else {
    return 0;
  };
  for (out, value) in [(x, left), (y, top), (width, wide), (height, tall)] {
    if let Some(out) = unsafe { out.as_mut() } {
      *out = value;
    }
  }
  1
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
  let uploaded = crate::gpu::shared().and_then(|gpu| {
    let target = unsafe { crate::gpu::macos::bgra_render_target(gpu, texture) }?;
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
