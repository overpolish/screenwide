// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! OS-neutral desktop navigation. IDs are opaque and valid only for a live
//! topology snapshot. A move completes only when both window and user arrive.
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Snapshot {
  /// Whether the window is on a full-screen Space alone, which only one of
  /// Screenwide's own windows can be moved off.
  pub fn on_full_screen_space(&self) -> bool {
    let [source] = self.membership.as_slice() else {
      return false;
    };
    self
      .groups
      .iter()
      .flat_map(|group| &group.desktops)
      .any(|desktop| &desktop.id == source && !desktop.regular)
  }
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

/// The regular desktop next to the window's in `direction`. A full-screen
/// Space is never a destination, but it can be the source: a window another
/// app's full-screen Space took in steps out to the desktops either side.
pub(crate) fn destination(
  snapshot: &Snapshot,
  direction: Direction,
) -> Result<(String, String), MoveError> {
  let [source] = snapshot.membership.as_slice() else {
    return Err(MoveError::AmbiguousWindow);
  };
  let (group, position) = snapshot
    .groups
    .iter()
    .find_map(|group| {
      let position = group
        .desktops
        .iter()
        .position(|desktop| &desktop.id == source)?;
      Some((group, position))
    })
    .ok_or(MoveError::AmbiguousWindow)?;
  // Never act on a stale/hidden target captured before a desktop switch.
  if &group.current != source || group.transitioning {
    return Err(MoveError::AmbiguousWindow);
  }
  let next = match direction {
    Direction::Previous => group.desktops[..position]
      .iter()
      .rev()
      .find(|desktop| desktop.regular),
    Direction::Next => group.desktops[position + 1..]
      .iter()
      .find(|desktop| desktop.regular),
  }
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
mod tests;
