// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWebview } from "@tauri-apps/api/webview";
import { type RefObject, useEffect, useRef, useState } from "react";

import { isSoundPath } from "../../sound-track/sound-library";
import { type TimelineScale } from "../timeline-scale";

// Songs and sound effects dragged in from the Finder or File Explorer. Over
// the strip, the moment under the pointer is where they land, which the
// strip marks while they are carried; dropped anywhere else in the window,
// they land as sounds added off the strip do (`sound-track/audio-clips.ts`).
// Files that are not sounds are left to whatever else takes them.

/** Where sounds carried over the strip would land, in seconds of the edited
 * video, while some are; null otherwise. */
export function useAudioDrop({
  onDrop,
  scale,
  trackRef,
}: {
  /** Told the sounds dropped, and where on the strip, if on it. */
  onDrop: ((paths: string[], at: number | null) => void) | undefined;
  scale: TimelineScale;
  trackRef: RefObject<HTMLElement | null>;
}) {
  const [landsAt, setLandsAt] = useState<number | null>(null);
  const latestRef = useRef({ onDrop, scale });
  latestRef.current = { onDrop, scale };
  const takesDrops = onDrop !== undefined;
  useEffect(() => {
    if (!takesDrops) return;
    let carriesSounds = false;
    /** The moment under a point the webview reports, in physical px. */
    const momentAt = (position: { x: number; y: number }) => {
      const box = trackRef.current?.getBoundingClientRect();
      const x = position.x / window.devicePixelRatio;
      const y = position.y / window.devicePixelRatio;
      if (
        !box ||
        x < box.left ||
        x > box.right ||
        y < box.top ||
        y > box.bottom
      )
        return null;
      const { scale } = latestRef.current;
      return scale.map.toPlayed(scale.toSeconds((x - box.left) / box.width));
    };
    let isDisposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        switch (payload.type) {
          case "enter":
            carriesSounds = payload.paths.some(isSoundPath);
            setLandsAt(carriesSounds ? momentAt(payload.position) : null);
            return;
          case "over":
            if (carriesSounds) setLandsAt(momentAt(payload.position));
            return;
          case "drop": {
            setLandsAt(null);
            const sounds = payload.paths.filter(isSoundPath);
            if (sounds.length > 0)
              latestRef.current.onDrop?.(sounds, momentAt(payload.position));
            return;
          }
          case "leave":
            setLandsAt(null);
        }
      })
      .then((stop) => {
        if (isDisposed) stop();
        else unlisten = stop;
      });
    return () => {
      isDisposed = true;
      unlisten?.();
    };
  }, [takesDrops, trackRef]);
  return landsAt;
}
