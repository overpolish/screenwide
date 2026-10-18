// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  screenwide_wgsl::assemble(&[
    (
      "workspace_backdrop",
      &["src/shaders/workspace_backdrop.wgsl"],
    ),
    (
      "workspace_magnifier",
      &[
        // First: it carries the module's directive.
        "src/shaders/workspace_magnifier.wgsl",
        "../osc/src/gpu/shaders/lens.wgsl",
      ],
    ),
  ]);
}
