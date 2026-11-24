// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

mod destination {
  use super::super::*;
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
  fn rejects_sticky_and_stale_targets() {
    for membership in [vec!["a", "b"], vec!["fullscreen"], vec!["b"], vec![]] {
      let mut state = snapshot();
      state.membership = membership.into_iter().map(String::from).collect();
      assert_eq!(
        destination(&state, Direction::Next),
        Err(MoveError::AmbiguousWindow)
      );
    }
  }
  #[test]
  fn steps_out_of_the_current_fullscreen_space_to_either_side() {
    let mut state = snapshot();
    state.membership = vec!["fullscreen".into()];
    state.groups[0].current = "fullscreen".into();
    assert_eq!(
      destination(&state, Direction::Previous).unwrap(),
      ("screen1".into(), "a".into())
    );
    assert_eq!(
      destination(&state, Direction::Next).unwrap(),
      ("screen1".into(), "b".into())
    );
  }
}

mod sequencing {
  use super::super::*;
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
