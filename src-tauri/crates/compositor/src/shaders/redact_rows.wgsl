// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The rows pass: one pixel of the rows target a pixel of a blurred box,
// holding the Gaussian along its row. The twin of `redact_rows_rgba`.

// The row's taps are the box's own pixels only, so nothing outside the box
// reaches in. A redaction's blur lays each pixel over the surface first, so a
// transparent one counts as the surface it shows against. The spotlights'
// blur has no surface to lay one over - what it softens keeps its own alpha -
// so it weighs each pixel by its alpha instead. Written premultiplied, its
// weight in alpha, for the paint pass to finish. The target holds the rows
// of the box the window holds, from `window.zw`.
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let local = vec2<i32>(redact_pixel(position.xy, redact.window.zw)) -
      vec2<i32>(redact.bounds.xy);
  let wide = i32(redact.bounds.z - redact.bounds.x);
  let reach = redact_reach(redact.size);
  let surface = saturate(redact.color.rgb);
  let origin = vec2<i32>(redact.bounds.xy);
  let held = redact_held();
  var sum = vec4<f32>(0.0);
  var total = 0.0;
  let last = min(min(local.x + reach, wide - 1), held.y);
  for (var x = max(max(local.x - reach, 0), held.x); x <= last; x++) {
    let weight = redact_weight(x - local.x, redact.size);
    let pixel = saturate(redact_read(origin + vec2<i32>(x, local.y)));
    if (redact.mode == 4u) {
      sum += weight * vec4<f32>(pixel.rgb * pixel.a, pixel.a);
    } else {
      sum += weight * vec4<f32>(mix(surface, pixel.rgb, pixel.a), 1.0);
    }
    total += weight;
  }
  return sum / total;
}
