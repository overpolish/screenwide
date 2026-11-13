// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { isSoundPath } from "../../sound-track/sound-library";

// A drag of sound files in from the system, followed over the editor. The
// page sees the drag over itself as points (Tauri's drag events, whose
// position, though called physical, is in the page's CSS px on macOS); the
// picture, drawn natively over the page, tells of a sound over it, gone
// from it, or let go on it (`editor/images/dropped.rs`). Going from one to
// the other leaves the first, so a leave waits a moment before it counts:
// long enough to reach the other, short enough to end soon after the drag
// leaves the window.

const LEAVE_GRACE_MS = 1_000;

/** The event the picture tells of a sound dragged over it with. */
const SOUND_DRAG_EVENT = "editor://sound-drag";

type PictureSoundDrag =
  { paths: string[]; type: "dropped" | "over" } | { type: "left" };

export type SoundDragHandlers = {
  /** Sound files arrived over the editor. */
  onCarry: (paths: string[]) => void;
  /** Let go on the page at a point, in CSS px. */
  onDrop: (point: { x: number; y: number }) => void;
  /** Gone from the editor, or let go where nothing takes them. */
  onLeave: () => void;
  /** Over the page at a point, in CSS px, or over the picture (null). */
  onMove: (point: { x: number; y: number } | null) => void;
  /** Let go on the picture. */
  onPictureDrop: (paths: string[]) => void;
};

export function useSoundDrag(handlers: SoundDragHandlers) {
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
        handlersRef.current.onLeave();
      }, LEAVE_GRACE_MS);
    };
    const carry = (paths: string[]) => {
      const sounds = paths.filter(isSoundPath);
      if (sounds.length === 0) return;
      // The editor comes to the front, so the timeline is seen.
      void getCurrentWindow().setFocus();
      handlersRef.current.onCarry(sounds);
    };
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        window.clearTimeout(leaving);
        switch (payload.type) {
          case "enter":
            carry(payload.paths);
            handlersRef.current.onMove(payload.position);
            return;
          case "over":
            handlersRef.current.onMove(payload.position);
            return;
          case "drop":
            handlersRef.current.onDrop(payload.position);
            return;
          case "leave":
            leave();
        }
      })
      .then(keep);
    void listen<PictureSoundDrag>(SOUND_DRAG_EVENT, ({ payload }) => {
      window.clearTimeout(leaving);
      switch (payload.type) {
        case "over":
          carry(payload.paths);
          handlersRef.current.onMove(null);
          return;
        case "dropped":
          handlersRef.current.onPictureDrop(payload.paths.filter(isSoundPath));
          return;
        case "left":
          leave();
      }
    }).then(keep);
    return () => {
      isDisposed = true;
      window.clearTimeout(leaving);
      for (const stop of stops) stop();
    };
  }, []);
}
