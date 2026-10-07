// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const MOMENT_PLACED_EVENT = "moments://placed";
const MOMENT_RELEASED_EVENT = "moments://released";

/** Long enough to read a kind's name at a glance, short enough that the
 * timer is back before the next moment is likely. Counted from the release,
 * so a held key keeps its moment on show. */
const SHOWN_MS = 1_500;

/** The twin of `NOTE_THRESHOLD` in `recording/note_gate.rs`: held this long,
 * a press is a voice note rather than a moment alone. */
const NOTE_THRESHOLD_MS = 300;

/** How often a note's timer is redrawn while the key is held. */
const TICK_MS = 250;

/** What a press does about a note: `recording` listens, `unavailable` has no
 * microphone to listen with, `off` has notes switched off. */
export type MomentVoice = "off" | "recording" | "unavailable";

export type PlacedMoment = {
  color: string;
  /** Tells two moments of one kind in a row apart. */
  key: number;
  name: string;
  /** Held long enough for a note, with no microphone to take it. */
  noMicrophone: boolean;
  /** How long the note has run, once the hold is long enough to be one; null
   * for a moment alone. */
  noteMs: number | null;
};

type Press = {
  color: string;
  key: number;
  name: string;
  pressedAt: number;
  releasedAt: number | null;
  voice: MomentVoice;
};

const asShown = (press: Press, now: number): PlacedMoment => {
  const heldMs = (press.releasedAt ?? now) - press.pressedAt;
  const isNote = heldMs >= NOTE_THRESHOLD_MS;
  return {
    color: press.color,
    key: press.key,
    name: press.name,
    noMicrophone: isNote && press.voice === "unavailable",
    noteMs: isNote && press.voice === "recording" ? heldMs : null,
  };
};

/** The moment a kind shortcut placed last, for as long as its key is held
 * and a moment after. The dock is the only feedback a moment gets: a sound
 * would be heard in the recording's system audio. */
export function usePlacedMoment() {
  const [press, setPress] = useState<Press | null>(null);
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    let disposed = false;
    const unlisteners: (() => void)[] = [];
    let hide = 0;
    let count = 0;
    let current: Press | null = null;
    const show = (next: Press | null) => {
      current = next;
      setPress(next);
    };
    const keep = (stop: () => void) => {
      if (disposed) stop();
      else unlisteners.push(stop);
    };
    void listen<{ color: string; name: string; voice: MomentVoice }>(
      MOMENT_PLACED_EVENT,
      ({ payload }) => {
        count += 1;
        window.clearTimeout(hide);
        show({
          ...payload,
          key: count,
          pressedAt: Date.now(),
          releasedAt: null,
        });
      },
    ).then(keep);
    void listen(MOMENT_RELEASED_EVENT, () => {
      if (!current || current.releasedAt !== null) return;
      const releasedAt = Date.now();
      window.clearTimeout(hide);
      // A hold showed its note, or that there was no microphone, for as long
      // as the key was down; letting go is the end of it, so the timer comes
      // back at once. A tap only flashed its kind, which stays a moment
      // longer to be read.
      if (releasedAt - current.pressedAt >= NOTE_THRESHOLD_MS) {
        show(null);
        return;
      }
      show({ ...current, releasedAt });
      hide = window.setTimeout(() => {
        show(null);
      }, SHOWN_MS);
    }).then(keep);
    return () => {
      disposed = true;
      window.clearTimeout(hide);
      for (const unlisten of unlisteners) unlisten();
    };
  }, []);

  // Only while a key is held does anything shown change with time.
  const isHeld = press !== null && press.releasedAt === null;
  useEffect(() => {
    if (!isHeld) return;
    const tick = window.setInterval(() => {
      setNow(Date.now());
    }, TICK_MS);
    return () => {
      window.clearInterval(tick);
    };
  }, [isHeld]);

  return press ? asShown(press, now) : null;
}
