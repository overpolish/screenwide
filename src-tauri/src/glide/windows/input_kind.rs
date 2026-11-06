// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputKind {
  Mouse,
  TrackpadContacts,
  TrackpadScroll,
}

impl InputKind {
  pub(super) const fn is_trackpad(self) -> bool {
    matches!(self, Self::TrackpadContacts | Self::TrackpadScroll)
  }
}
