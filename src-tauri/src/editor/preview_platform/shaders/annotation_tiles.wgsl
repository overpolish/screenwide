// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The canvas in tiles, each holding which annotations may reach it, so a
// pixel walks only those rather than every annotation the frame shows. A
// compute pass bins every annotation by the bounds its own layer turns pixels
// away at, one thread a tile; the canvas pass then reads its tile's bits in
// index order, which is document order. `canvas.annotation_tiles` is the
// tile's side in canvas pixels, the grid's width and height in tiles, and how
// many 32-bit words each tile's set takes.

@group(0) @binding(14) var<storage, read> annotation_tile_masks: array<u32>;
@group(1) @binding(0) var<storage, read_write> annotation_tiles_out: array<u32>;

fn annotation_next(probe: vec2<f32>, start: u32, last: u32) -> u32 {
  if (start >= last) {
    return last;
  }
  let grid = canvas.annotation_tiles;
  let tile = min(vec2<u32>(max(probe, vec2<f32>(0.0)) / f32(grid.x)),
                 vec2<u32>(grid.y, grid.z) - 1u);
  let base = (tile.y * grid.y + tile.x) * grid.w;
  var word = start >> 5u;
  var bits = annotation_tile_masks[base + word] & (0xffffffffu << (start & 31u));
  while (bits == 0u) {
    word += 1u;
    if (word >= grid.w || word * 32u >= last) {
      return last;
    }
    bits = annotation_tile_masks[base + word];
  }
  return min(word * 32u + countTrailingZeros(bits), last);
}

/// Where `annotation` can reach on the canvas, by its kind. A spotlight's
/// shade reaches everywhere; a kind the canvas draws as an arrow reaches as
/// an arrow does.
fn annotation_reach(annotation: PreviewArrow, feather: f32) -> vec4<f32> {
  let halo = max(annotation.hover, 0.0);
  let kind = annotation.kind;
  if (kind == annotation_spotlight_kind) {
    return annotation_all_bounds;
  }
  if (kind == annotation_magnify_kind) {
    return annotation_magnify_bounds(annotation, feather);
  }
  if (kind == annotation_sticker_kind) {
    return annotation_sticker_bounds(annotation, halo, feather);
  }
  if (kind == annotation_highlight_kind) {
    return annotation_highlight_bounds(annotation, feather);
  }
  if (kind == 3u) {
    return annotation_redact_halo_bounds(annotation, feather);
  }
  if (kind == 1u) {
    return annotation_counter_bounds(annotation, halo, feather);
  }
  if (kind == 2u) {
    return annotation_text_reach(annotation, halo, feather);
  }
  if (kind == annotation_shape_kind) {
    return annotation_shape_bounds(annotation.geometry, halo, feather);
  }
  if (kind == annotation_draw_kind) {
    return annotation_draw_bounds(annotation.geometry, halo, feather);
  }
  return annotation_arrow_bounds(annotation.geometry, halo, feather);
}

@compute @workgroup_size(8, 8)
fn bin_annotations(@builtin(global_invocation_id) id: vec3<u32>) {
  let grid = canvas.annotation_tiles;
  if (id.x >= grid.y || id.y >= grid.z) {
    return;
  }
  let low = vec2<f32>(id.xy) * f32(grid.x);
  let high = low + f32(grid.x);
  // The canvas pass's own feather: half a drawn pixel.
  let feather = max(canvas.motion.z, 1e-4) * 0.5;
  let total = canvas.annotation_options.y;
  let base = (id.y * grid.y + id.x) * grid.w;
  for (var word = 0u; word < grid.w; word++) {
    var bits = 0u;
    for (var bit = 0u; bit < 32u; bit++) {
      let index = word * 32u + bit;
      if (index >= total) {
        break;
      }
      let reach = annotation_reach(annotation_arrows[index], feather);
      // Written so bounds that are not numbers keep the annotation, as the
      // layer's own test would draw it.
      if (!(any(reach.xy > high) || any(reach.zw < low))) {
        bits |= 1u << bit;
      }
    }
    annotation_tiles_out[base + word] = bits;
  }
}
