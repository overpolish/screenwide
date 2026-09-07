// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! OS-neutral desktop navigation. IDs are opaque and valid only for a live
//! topology snapshot. A move completes only when both window and user arrive.
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(crate) enum Direction {
  Previous,
  Next,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Desktop {
  pub id: String,
  pub regular: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DesktopGroup {
  pub id: String,
  pub current: String,
  pub transitioning: bool,
  pub desktops: Vec<Desktop>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Snapshot {
  pub groups: Vec<DesktopGroup>,
  pub membership: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum MoveError {
  Unsupported,
  Unavailable(String),
  AmbiguousWindow,
  NoAdjacentDesktop,
  MoveUnconfirmed,
  FollowUnconfirmed,
}

pub(crate) trait DesktopAdapter {
  fn snapshot(&self) -> Result<Snapshot, MoveError>;
  /// Own the native choreography as one operation. Platforms may carry the
  /// window through a transition rather than first sending it off screen.
  fn carry_window(&self, group: &str, destination: &str) -> Result<(), MoveError>;
}

pub(crate) fn destination(
  snapshot: &Snapshot,
  direction: Direction,
) -> Result<(String, String), MoveError> {
  let [source] = snapshot.membership.as_slice() else {
    return Err(MoveError::AmbiguousWindow);
  };
  let group = snapshot
    .groups
    .iter()
    .find(|group| {
      group
        .desktops
        .iter()
        .any(|desktop| &desktop.id == source && desktop.regular)
    })
    .ok_or(MoveError::AmbiguousWindow)?;
  // Never act on a stale/hidden target captured before a desktop switch.
  if &group.current != source || group.transitioning {
    return Err(MoveError::AmbiguousWindow);
  }
  let regular: Vec<_> = group
    .desktops
    .iter()
    .filter(|desktop| desktop.regular)
    .collect();
  let index = regular
    .iter()
    .position(|desktop| &desktop.id == source)
    .unwrap();
  let next = match direction {
    Direction::Previous => index.checked_sub(1),
    Direction::Next => index.checked_add(1),
  }
  .and_then(|index| regular.get(index))
  .ok_or(MoveError::NoAdjacentDesktop)?;
  Ok((group.id.clone(), next.id.clone()))
}

pub(crate) fn move_and_follow(
  adapter: &impl DesktopAdapter,
  direction: Direction,
) -> Result<(), MoveError> {
  let snapshot = adapter.snapshot()?;
  let (group, destination) = destination(&snapshot, direction)?;
  move_to_and_follow(adapter, &group, &snapshot.membership[0], &destination)
}

/// Commit the previewed ID, never a newly calculated neighbour on release.
pub(crate) fn move_to_and_follow(
  adapter: &impl DesktopAdapter,
  group: &str,
  source: &str,
  destination: &str,
) -> Result<(), MoveError> {
  let snapshot = adapter.snapshot()?;
  if snapshot.membership != [source]
    || !snapshot.groups.iter().any(|item| {
      item.id == group
        && item.current == source
        && !item.transitioning
        && item
          .desktops
          .iter()
          .any(|desktop| desktop.id == destination && desktop.regular)
    })
  {
    return Err(MoveError::AmbiguousWindow);
  }
  if source == destination {
    return Ok(());
  }
  adapter.carry_window(group, destination)?;
  // Some adapters announce the new current desktop before its animation has
  // finished. Confirm completion only when arrival remains quiescent across
  // several display frames, rather than ending it on that first notification.
  let mut arrived_since = None;
  wait_until(|| {
    let state = adapter.snapshot()?;
    let arrived = state.membership == [destination]
      && state
        .groups
        .iter()
        .any(|item| item.id == group && item.current == destination && !item.transitioning);
    if !arrived {
      arrived_since = None;
    }
    Ok(
      arrived
        && arrived_since.get_or_insert_with(Instant::now).elapsed() >= Duration::from_millis(50),
    )
  })
  .map_err(|_| MoveError::FollowUnconfirmed)
}

fn wait_until(mut ready: impl FnMut() -> Result<bool, MoveError>) -> Result<(), MoveError> {
  let deadline = Instant::now() + Duration::from_secs(2);
  loop {
    if ready()? {
      return Ok(());
    }
    if Instant::now() >= deadline {
      return Err(MoveError::MoveUnconfirmed);
    }
    std::thread::sleep(Duration::from_millis(25));
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn snapshot() -> Snapshot {
    serde_json::from_str(
      r#"{"membership":["a"],"groups":[
      {"id":"screen1","current":"a","transitioning":false,"desktops":[
        {"id":"a","regular":true},{"id":"fullscreen","regular":false},{"id":"b","regular":true}]},
      {"id":"screen2","current":"c","transitioning":false,"desktops":[{"id":"c","regular":true}]}]}"#,
    )
    .unwrap()
  }
  #[test]
  fn skips_fullscreen_and_stays_on_source_display() {
    assert_eq!(
      destination(&snapshot(), Direction::Next).unwrap(),
      ("screen1".into(), "b".into())
    );
    assert_eq!(
      destination(&snapshot(), Direction::Previous),
      Err(MoveError::NoAdjacentDesktop)
    );
    let mut state = snapshot();
    state.membership = vec!["b".into()];
    state.groups[0].current = "b".into();
    assert_eq!(
      destination(&state, Direction::Next),
      Err(MoveError::NoAdjacentDesktop)
    );
  }
  #[test]
  fn rejects_starting_during_a_desktop_transition() {
    let mut state = snapshot();
    state.groups[0].transitioning = true;
    assert_eq!(
      destination(&state, Direction::Next),
      Err(MoveError::AmbiguousWindow)
    );
  }
  #[test]
  fn rejects_sticky_fullscreen_and_stale_targets() {
    for membership in [vec!["a", "b"], vec!["fullscreen"], vec!["b"], vec![]] {
      let mut state = snapshot();
      state.membership = membership.into_iter().map(String::from).collect();
      assert_eq!(
        destination(&state, Direction::Next),
        Err(MoveError::AmbiguousWindow)
      );
    }
  }
}

#[cfg(test)]
mod sequencing_tests {
  use super::*;
  use std::cell::Cell;

  struct Adapter {
    stage: Cell<u8>,
    follow_fails: bool,
    transition_reads: Cell<u8>,
  }
  impl DesktopAdapter for Adapter {
    fn snapshot(&self) -> Result<Snapshot, MoveError> {
      let stage = self.stage.get();
      let transitioning = stage == 2 && self.transition_reads.get() > 0;
      if transitioning {
        self.transition_reads.set(self.transition_reads.get() - 1);
      }
      Ok(Snapshot {
        groups: vec![DesktopGroup {
          id: "global".into(),
          current: if stage == 2 { "b" } else { "a" }.into(),
          transitioning,
          desktops: ["a", "b"]
            .map(|id| Desktop {
              id: id.into(),
              regular: true,
            })
            .into(),
        }],
        membership: vec![if stage > 0 { "b" } else { "a" }.into()],
      })
    }
    fn carry_window(&self, group: &str, destination: &str) -> Result<(), MoveError> {
      assert_eq!(group, "global");
      assert_eq!(destination, "b");
      assert_eq!(self.stage.get(), 0);
      self.stage.set(1);
      if self.follow_fails {
        return Err(MoveError::FollowUnconfirmed);
      }
      self.stage.set(2);
      Ok(())
    }
  }
  #[test]
  fn completes_only_after_window_and_user_arrive() {
    let adapter = Adapter {
      stage: Cell::new(0),
      follow_fails: false,
      transition_reads: Cell::new(2),
    };
    assert_eq!(move_and_follow(&adapter, Direction::Next), Ok(()));
    assert_eq!(adapter.stage.get(), 2);
    assert_eq!(adapter.transition_reads.get(), 0);
  }
  #[test]
  fn return_to_source_and_stale_destination_never_carry() {
    let adapter = Adapter {
      stage: Cell::new(0),
      follow_fails: false,
      transition_reads: Cell::new(0),
    };
    assert_eq!(move_to_and_follow(&adapter, "global", "a", "a"), Ok(()));
    assert_eq!(adapter.stage.get(), 0);
    assert_eq!(
      move_to_and_follow(&adapter, "global", "a", "removed"),
      Err(MoveError::AmbiguousWindow)
    );
    assert_eq!(
      move_to_and_follow(&adapter, "global", "changed", "b"),
      Err(MoveError::AmbiguousWindow)
    );
    assert_eq!(adapter.stage.get(), 0);
  }
  #[test]
  fn move_success_does_not_hide_follow_failure() {
    let adapter = Adapter {
      stage: Cell::new(0),
      follow_fails: true,
      transition_reads: Cell::new(0),
    };
    assert_eq!(
      move_and_follow(&adapter, Direction::Next),
      Err(MoveError::FollowUnconfirmed)
    );
  }
}
