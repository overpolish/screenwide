// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which handler each key goes to, kept apart from the hook so the rules are
//! tested on every platform.

use std::collections::{HashMap, HashSet};

use crate::{KeyEvent, Modifiers, Phase, Priority, Route};

pub(crate) type Observer = Box<dyn FnMut(&KeyEvent) + Send>;
pub(crate) type Taker = Box<dyn FnMut(&KeyEvent) -> Route + Send>;

const SHIFT: [u32; 3] = [0x10, 0xA0, 0xA1];
const CONTROL: [u32; 3] = [0x11, 0xA2, 0xA3];
const ALT: [u32; 3] = [0x12, 0xA4, 0xA5];
const WIN: [u32; 2] = [0x5B, 0x5C];

struct ObserverEntry {
  id: u64,
  handle: Observer,
}

struct TakerEntry {
  id: u64,
  priority: Priority,
  handle: Taker,
}

/// One key event as the hook reads it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawKey {
  pub(crate) key: u32,
  pub(crate) pressed: bool,
  pub(crate) time: u32,
  pub(crate) extended: bool,
  pub(crate) injected: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Outcome {
  /// Keep the key from the rest of the system.
  pub(crate) take: bool,
  /// Send the mask key: a key was taken while Windows was held, and
  /// releasing Windows with nothing seen between would open Start.
  pub(crate) mask: bool,
}

#[derive(Default)]
pub(crate) struct Router {
  next_id: u64,
  observers: Vec<ObserverEntry>,
  /// In routing order: by priority, then by registration.
  takers: Vec<TakerEntry>,
  /// Each taken key and the handler that took it.
  owners: HashMap<u32, u64>,
  pressed: HashSet<u32>,
  paused: bool,
}

impl Router {
  pub(crate) fn add_observer(&mut self, handle: Observer) -> u64 {
    let id = self.next_id();
    self.observers.push(ObserverEntry { id, handle });
    id
  }

  pub(crate) fn add_taker(&mut self, priority: Priority, handle: Taker) -> u64 {
    let id = self.next_id();
    let at = self
      .takers
      .iter()
      .position(|taker| taker.priority > priority)
      .unwrap_or(self.takers.len());
    self.takers.insert(
      at,
      TakerEntry {
        id,
        priority,
        handle,
      },
    );
    id
  }

  /// Removes a handler. A key it holds goes back to the system: its release
  /// passes.
  pub(crate) fn remove(&mut self, id: u64) {
    self.observers.retain(|observer| observer.id != id);
    self.takers.retain(|taker| taker.id != id);
    self.owners.retain(|_, owner| *owner != id);
  }

  pub(crate) fn set_paused(&mut self, paused: bool) {
    self.paused = paused;
  }

  /// Forgets every held key but `current` that `is_down` says is up: a
  /// release the hook never saw, as when Windows+L moves to the lock screen
  /// with Windows still down. Left held, a lost Windows would turn Shift+S
  /// into Windows+Shift+S. A taken key is kept: the system never saw its
  /// press, so it reads as up the whole time it is held.
  pub(crate) fn forget_released(&mut self, current: u32, is_down: impl Fn(u32) -> bool) {
    let released = self
      .pressed
      .iter()
      .copied()
      .filter(|key| *key != current && !self.owners.contains_key(key) && !is_down(*key))
      .collect::<Vec<_>>();
    for key in released {
      self.pressed.remove(&key);
    }
  }

  pub(crate) fn route(&mut self, raw: RawKey) -> Outcome {
    let phase = match (raw.pressed, self.pressed.contains(&raw.key)) {
      (true, true) => Phase::Repeat,
      (true, false) => Phase::Press,
      (false, _) => Phase::Release,
    };
    let modifiers = self.modifiers();
    if raw.pressed {
      self.pressed.insert(raw.key);
    } else {
      self.pressed.remove(&raw.key);
    }
    let mut event = KeyEvent {
      key: raw.key,
      phase,
      modifiers,
      time: raw.time,
      extended: raw.extended,
      injected: raw.injected,
      may_take: !self.paused,
      owned: false,
    };
    for observer in &mut self.observers {
      (observer.handle)(&event);
    }

    if let Some(owner) = self.owners.get(&raw.key).copied() {
      event.owned = true;
      if let Some(taker) = self.takers.iter_mut().find(|taker| taker.id == owner) {
        (taker.handle)(&event);
      }
      if phase == Phase::Release {
        self.owners.remove(&raw.key);
      }
      return Outcome {
        take: true,
        mask: false,
      };
    }

    if phase == Phase::Release {
      // Every handler hears a release it does not own, as it heard the
      // press it passed on.
      for taker in &mut self.takers {
        (taker.handle)(&event);
      }
      return Outcome::default();
    }

    let mut taken_by = None;
    for taker in &mut self.takers {
      if (taker.handle)(&event) == Route::Take && event.may_take {
        taken_by = Some(taker.id);
        break;
      }
    }
    let Some(owner) = taken_by else {
      return Outcome::default();
    };
    self.owners.insert(raw.key, owner);
    Outcome {
      take: true,
      mask: modifiers.win && !WIN.contains(&raw.key),
    }
  }

  fn modifiers(&self) -> Modifiers {
    let any = |keys: &[u32]| keys.iter().any(|key| self.pressed.contains(key));
    Modifiers {
      shift: any(&SHIFT),
      control: any(&CONTROL),
      alt: any(&ALT),
      win: any(&WIN),
    }
  }

  fn next_id(&mut self) -> u64 {
    self.next_id += 1;
    self.next_id
  }
}

#[cfg(test)]
mod tests;
