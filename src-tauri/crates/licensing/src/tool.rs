// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/// A paid tool. Each is sold as its own product, so a key unlocks the tools
/// its product grants; a bundle buyer holds one key per tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
  Ruler,
  References,
  Glide,
  /// The live Annotate overlay, for drawing on the screen. Annotations in
  /// the editor stay free.
  Annotate,
}

impl Tool {
  pub const ALL: [Tool; 4] = [Tool::Ruler, Tool::References, Tool::Glide, Tool::Annotate];

  /// The name the licence server signs for the tool in a record's
  /// entitlements. Part of the server's contract: never rename one.
  pub fn entitlement(self) -> &'static str {
    match self {
      Tool::Ruler => "ruler",
      Tool::References => "references",
      Tool::Glide => "glide",
      Tool::Annotate => "annotate",
    }
  }

  pub fn from_entitlement(name: &str) -> Option<Tool> {
    Tool::ALL
      .into_iter()
      .find(|tool| tool.entitlement() == name)
  }
}
