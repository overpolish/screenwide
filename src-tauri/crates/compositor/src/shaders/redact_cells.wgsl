// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The cells pass: one pixel of the cells target a cell of a classically
// pixelated box, holding that cell's exact average. The twin of
// `redact_cells_rgba`.

// Every pixel in the cell, each laid over the surface first so a transparent
// one counts as the surface it shows against.
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let cell = vec2<u32>(position.xy);
  let box = redact.bounds.zw - redact.bounds.xy;
  let size = max(redact.size, 1.0);
  let origin = redact_grid_origin();
  // Every pixel the division could put in this cell, one either side of
  // where it nominally starts and ends.
  let low = vec2<u32>(max(floor(vec2<f32>(cell) * size + origin) - 1.0, vec2<f32>(0.0)));
  let high = vec2<u32>(clamp(ceil(vec2<f32>(cell + 1u) * size + origin) + 1.0, vec2<f32>(0.0),
                             vec2<f32>(box)));
  let span = select(vec2<u32>(0u), high - low, high > low);
  let surface = vec3<u32>(round(saturate(redact.color.rgb) * 255.0));
  var sum = vec3<u32>(0u);
  var count = 0u;
  for (var y = 0u; y < span.y; y++) {
    for (var x = 0u; x < span.x; x++) {
      let local = low + vec2<u32>(x, y);
      if (any(redact_cell_of(local) != cell)) {
        continue;
      }
      let texel = vec2<i32>(redact.bounds.xy + local);
      let pixel = vec4<u32>(round(saturate(textureLoad(redact_source, texel, 0)) * 255.0));
      sum += (pixel.rgb * pixel.a + surface * (255u - pixel.a) + 127u) / 255u;
      count += 1u;
    }
  }
  var average = surface;
  if (count != 0u) {
    average = (sum + count / 2u) / count;
  }
  return vec4<f32>(vec3<f32>(average) / 255.0, 1.0);
}
