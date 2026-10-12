// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  screenwide_wgsl::assemble(&[
    ("camera_thumbnail", &["src/shaders/camera_thumbnail.wgsl"]),
    (
      "desktop_compositor",
      &["src/shaders/desktop_compositor.wgsl"],
    ),
  ]);
}
