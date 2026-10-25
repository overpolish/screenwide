// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { GlidePickerCard } from "./glide-picker-card";
import { describePick, type GlidePickerProps } from "./glide-picker-types";

/** What the native picker tells one card's window (`picker_cards.rs`). */
type GlidePickerEvent = {
  count: number;
  desktop: string;
  iconPath: string | null;
  index: number;
  kind: GlidePickerProps["kind"];
  origin: boolean;
  phase: string;
  selected: boolean;
  sessionId: number;
};

/** One card of the Spaces or display picker, in a window of its own. */
export function GlidePickerWindow() {
  const [event, setEvent] = useState<GlidePickerEvent | null>(null);
  const [pulse, setPulse] = useState(0);
  const readySessionRef = useRef<number | null>(null);

  useEffect(() => {
    let disposed = false;
    const window = getCurrentWindow();
    let stopListening: (() => void) | undefined;

    // Register first: native may have the initial state queued as soon as the
    // webview exists, so readiness must never race the listener installation.
    // Global listeners receive even events addressed to other windows. Each
    // card must only consume the state addressed to its own native window.
    void window
      .listen<GlidePickerEvent>("glide://picker-card", ({ payload }) => {
        setEvent(payload);
        if (payload.phase !== "ready" || !payload.selected) {
          if (payload.phase !== "ready") readySessionRef.current = null;
          return;
        }
        if (readySessionRef.current !== payload.sessionId) {
          readySessionRef.current = payload.sessionId;
          setPulse((value) => value + 1);
        }
      })
      .then(async (unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        stopListening = unlisten;
        await window.emit("glide://picker-ready", { label: window.label });
      });

    return () => {
      disposed = true;
      stopListening?.();
    };
  }, []);

  return (
    <main className="flex size-full items-center justify-center">
      {event ? (
        // Each pick comes out afresh, rather than carrying on from the last.
        <GlidePickerCard
          iconSrc={
            event.selected && event.iconPath
              ? convertFileSrc(event.iconPath)
              : null
          }
          isOrigin={event.origin}
          isSelected={event.selected}
          isShown
          key={event.sessionId}
          label={
            event.selected
              ? describePick(event.kind, event.index, event.count)
              : undefined
          }
          pulse={pulse}
        />
      ) : null}
    </main>
  );
}
