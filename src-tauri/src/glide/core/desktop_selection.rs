// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::desktops::{destination, Direction, MoveError, Snapshot};

/// Navigate a preview without changing the actual window's desktop membership.
pub(crate) fn step(
  snapshot: &Snapshot,
  selected: Option<&str>,
  direction: Direction,
) -> Result<String, MoveError> {
  let [source] = snapshot.membership.as_slice() else {
    return Err(MoveError::AmbiguousWindow);
  };
  let mut preview = snapshot.clone();
  if let Some(selected) = selected {
    let group = preview
      .groups
      .iter_mut()
      .find(|group| {
        group.current == *source
          && group
            .desktops
            .iter()
            .any(|desktop| desktop.id == selected && desktop.regular)
      })
      .ok_or(MoveError::AmbiguousWindow)?;
    group.current = selected.to_owned();
    preview.membership = vec![selected.to_owned()];
  }
  destination(&preview, direction).map(|(_, id)| id)
}

#[cfg(test)]
mod tests {
  use super::*;
  fn snapshot() -> Snapshot {
    serde_json::from_str(r#"{"membership":["a"],"groups":[{"id":"display","current":"a","transitioning":false,"desktops":[{"id":"a","regular":true},{"id":"full","regular":false},{"id":"b","regular":true},{"id":"c","regular":true}]}]}"#).unwrap()
  }
  #[test]
  fn previews_multiple_steps_and_return_to_source_without_moving_window() {
    let state = snapshot();
    let selected = step(&state, None, Direction::Next).unwrap();
    assert_eq!(selected, "b");
    assert_eq!(step(&state, Some(&selected), Direction::Next).unwrap(), "c");
    assert_eq!(
      step(&state, Some(&selected), Direction::Previous).unwrap(),
      "a"
    );
    assert_eq!(state.membership, ["a"]);
    assert_eq!(state.groups[0].current, "a");
  }
  #[test]
  fn selection_never_wraps_or_enters_fullscreen() {
    let state = snapshot();
    assert_eq!(
      step(&state, Some("c"), Direction::Next),
      Err(MoveError::NoAdjacentDesktop)
    );
    assert_eq!(
      step(&state, Some("full"), Direction::Next),
      Err(MoveError::AmbiguousWindow)
    );
  }
}
