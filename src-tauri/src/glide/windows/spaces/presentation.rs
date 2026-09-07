// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn publish(app: &AppHandle, s: &mut Session, phase: &'static str) {
  let source = s.snapshot.membership[0].clone();
  let selected = s.selected.as_deref().unwrap_or(&source);
  let mut cards = Vec::new();
  for group in &s.snapshot.groups {
    for desktop in group.desktops.iter().filter(|desktop| desktop.regular) {
      cards.push(preview_windows::Preview {
        session_id: s.id,
        desktop: desktop.id.clone(),
        selected: desktop.id == selected,
        origin: desktop.id == source,
        index: cards.len(),
        count: 0,
        phase,
        icon_path: s.icon_path.clone(),
      });
    }
  }
  let count = cards.len();
  for card in &mut cards {
    card.count = count;
  }
  if !s.revealed {
    super::super::cursor::hide_cursor();
    s.revealed = true;
  }
  let offsets = (0..count)
    .map(|index| (index as f64 * 56.0, 0.0))
    .collect::<Vec<_>>();
  preview_windows::show_arranged(
    app,
    cards,
    f64::from(s.anchor.x),
    f64::from(s.anchor.y),
    &offsets,
  );
}
