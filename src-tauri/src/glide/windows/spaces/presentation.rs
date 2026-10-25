// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn publish(app: &AppHandle, s: &mut Session, phase: &'static str) {
  let source = s.snapshot.membership[0].clone();
  let selected = s.selected.as_deref().unwrap_or(&source);
  let mut cards = Vec::new();
  for group in &s.snapshot.groups {
    for desktop in group.desktops.iter().filter(|desktop| desktop.regular) {
      cards.push(picker_cards::PickerCard {
        session_id: s.id,
        kind: picker_cards::PickerKind::Spaces,
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
  let offsets = picker_cards::row(count);
  picker_cards::show_arranged(
    app,
    cards,
    Anchor::Physical(f64::from(s.anchor.x), f64::from(s.anchor.y)),
    &offsets,
  );
}
