// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { GlidePreview } from "./glide-preview";
import { type GlideRegion } from "./glide-regions";

export type GlideSpacePreviewEvent = {
  count: number;
  desktop: string;
  iconPath: string | null;
  index: number;
  origin: boolean;
  phase: string;
  selected: boolean;
  sessionId: number;
};

const selectedRegion = (event: GlideSpacePreviewEvent): GlideRegion | null =>
  event.selected
    ? { colSpan: 2, colStart: 0, gridCols: 2, rowSpan: 2, rowStart: 0 }
    : null;

/** The small native Spaces host reports its own desktop state to this view. */
export function GlideSpaceWindow() {
  const [event, setEvent] = useState<GlideSpacePreviewEvent | null>(null);
  const [pulse, setPulse] = useState(0);
  const readySessionRef = useRef<number | null>(null);

  useEffect(() => {
    let disposed = false;
    const window = getCurrentWindow();
    let stopListening: (() => void) | undefined;

    // Register first: native may have the initial state queued as soon as the
    // webview exists, so readiness must never race the listener installation.
    // Global listeners receive even events addressed to other windows. Each
    // panel must only consume the state addressed to its own native window.
    void window
      .listen<GlideSpacePreviewEvent>(
        "glide://space-preview",
        ({ payload }) => {
          setEvent(payload);
          if (payload.phase !== "ready" || !payload.selected) {
            if (payload.phase !== "ready") readySessionRef.current = null;
            return;
          }
          if (readySessionRef.current !== payload.sessionId) {
            readySessionRef.current = payload.sessionId;
            setPulse((value) => value + 1);
          }
        },
      )
      .then(async (unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        stopListening = unlisten;
        await window.emit("glide://space-ready", { label: window.label });
      });

    return () => {
      disposed = true;
      stopListening?.();
    };
  }, []);

  return (
    <main className="flex h-screen w-screen items-center justify-center">
      <GlideSpacePreview event={event} pulse={pulse} />
    </main>
  );
}

export function GlideSpacePreview({
  event,
  pulse,
}: {
  event: GlideSpacePreviewEvent | null;
  pulse: number;
}) {
  const iconSrc =
    event?.selected && event.iconPath
      ? event.iconPath.startsWith("data:")
        ? event.iconPath
        : convertFileSrc(event.iconPath)
      : null;

  return (
    <GlidePreview
      fit={null}
      iconSrc={iconSrc}
      neutralFill={event?.origin === true && !event.selected}
      pending={null}
      pulse={pulse}
      region={event ? selectedRegion(event) : null}
    />
  );
}
