// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { isSoundPath } from "../../sound-track/sound-library";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// Sounds dragged in from the system are dropped on the timeline, whose left
// half takes them as sound effects and right half as songs, under the
// islands the island window lays over each half (`sound-kind-chooser.tsx`).
// The islands come out as a sound reaches any part of the window: the page
// follows the drag over itself, as the island window lets it through, and
// the picture, drawn natively, takes no sound but tells of one over it
// (`editor/images/dropped.rs`). Going from one to the other leaves the
// first, so the islands wait a moment before going: long enough to reach
// the other, short enough to go soon after the drag leaves the window or is
// let go anywhere but the timeline.

const LEAVE_GRACE_MS = 1_000;

/** The event the picture tells of a sound dragged over it with. */
const SOUND_DRAG_EVENT = "editor://sound-drag";

type PictureSoundDrag = { paths: string[]; type: "over" } | { type: "left" };

/** The band the islands stand over (`recording-strip-band.tsx`). */
export const soundDropArea = () =>
  document.querySelector<HTMLElement>("[data-sound-drop-area]");

/** The kind a drop at a point in the page, in CSS px, chooses, or none off
 * the timeline. The timeline reads left to right in every language. */
const kindAt = (x: number, y: number): AudioClipKind | null => {
  const box = soundDropArea()?.getBoundingClientRect();
  if (!box || x < box.left || x > box.right || y < box.top || y > box.bottom) {
    return null;
  }
  return x < box.left + box.width / 2 ? "effect" : "music";
};

export function useSoundDrag(handlers: {
  /** Sound files arrived, `over` the kind the drag is over. */
  onCarry: (paths: string[], over: AudioClipKind | null) => void;
  /** Let go over `kind`, or nowhere that takes it, or gone. */
  onDrop: (kind: AudioClipKind | null) => void;
  onOver: (over: AudioClipKind | null) => void;
}) {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;
  useEffect(() => {
    let isDisposed = false;
    const stops: (() => void)[] = [];
    const keep = (stop: () => void) => {
      if (isDisposed) stop();
      else stops.push(stop);
    };
    let leaving = 0;
    const leave = () => {
      window.clearTimeout(leaving);
      leaving = window.setTimeout(() => {
        handlersRef.current.onDrop(null);
      }, LEAVE_GRACE_MS);
    };
    const carry = (paths: string[], over: AudioClipKind | null) => {
      const sounds = paths.filter(isSoundPath);
      if (sounds.length === 0) return;
      // The editor comes to the front, so the timeline is seen.
      void getCurrentWindow().setFocus();
      handlersRef.current.onCarry(sounds, over);
    };
    // Tauri calls the drag's position physical, but on macOS it arrives in
    // the page's own CSS pixels.
    const under = ({ x, y }: { x: number; y: number }) => kindAt(x, y);
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        window.clearTimeout(leaving);
        switch (payload.type) {
          case "enter":
            carry(payload.paths, under(payload.position));
            return;
          case "over":
            handlersRef.current.onOver(under(payload.position));
            return;
          case "drop":
            handlersRef.current.onDrop(under(payload.position));
            return;
          case "leave":
            leave();
        }
      })
      .then(keep);
    void listen<PictureSoundDrag>(SOUND_DRAG_EVENT, ({ payload }) => {
      if (payload.type === "left") {
        leave();
        return;
      }
      window.clearTimeout(leaving);
      carry(payload.paths, null);
    }).then(keep);
    return () => {
      isDisposed = true;
      window.clearTimeout(leaving);
      for (const stop of stops) stop();
    };
  }, []);
}
