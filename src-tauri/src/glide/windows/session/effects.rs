// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The monitor picker's cards as the detector steps through them.

use super::*;

pub(super) fn update_monitor(app: &AppHandle, id: u64, direction: Option<(i8, i8)>, ready: bool) {
  crate::glide::core::trace::input(
    "windows-monitor",
    format!("update id={id} direction={direction:?} ready={ready}"),
  );
  let Some(Some((anchor, offsets, cards, hide))) =
    shared::with_session::<WindowsGlide, _>(|session| {
      if session.id != id {
        return None;
      }
      let selection = session.monitors.as_mut()?;
      if let Some(direction) = direction {
        if let Some(next) = crate::glide::core::monitors::neighbour(
          &selection.displays,
          selection.selected,
          direction,
        ) {
          selection.selected = next;
        }
      }
      let hide = !session.revealed;
      session.revealed = true;
      let offsets = crate::glide::core::monitors::card_offsets(&selection.displays);
      let cards = selection
        .displays
        .iter()
        .enumerate()
        .map(|(index, monitor)| picker_cards::PickerCard {
          session_id: id,
          kind: picker_cards::PickerKind::Displays,
          desktop: monitor.id.clone(),
          selected: index == selection.selected,
          origin: index == selection.source,
          index,
          count: selection.displays.len(),
          phase: if ready { "ready" } else { "settling" },
          icon_path: session.extra.icon_path.clone(),
        })
        .collect();
      Some((point(session.anchor), offsets, cards, hide))
    })
  else {
    return;
  };
  if hide {
    cursor::hide_cursor();
  }
  picker_cards::show_arranged(
    app,
    cards,
    picker_cards::Anchor::Physical(f64::from(anchor.x), f64::from(anchor.y)),
    &offsets,
  );
  // Reconcile after show_arranged installs the cards. Keep the session lock
  // while reading and applying the icon so an icon lookup cannot interleave
  // between the read and setter and then be overwritten by stale None.
  if let Ok(state) = STATE.lock() {
    if let Some(session) = state.as_ref().filter(|session| session.id == id) {
      picker_cards::set_icon(app, id, session.extra.icon_path.clone());
    }
  }
}
