// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

/** A stretch of source time, `[start, end)` in milliseconds. */
export type RecordingPinStretch = [number, number];

/**
 * How a pinned annotation's path is coming along, as the native tracker
 * reports it: `progress` from 0 to 1 while it is worked out, and once it has
 * landed, where the content was lost (`weak`) or off the frame (`hidden`).
 */
export type RecordingPinStatus = {
  hidden: RecordingPinStretch[];
  progress: number | null;
  weak: RecordingPinStretch[];
};

type PinStatusPayload = RecordingPinStatus & { annotationId: string };
type PinStatusEvent = PinStatusPayload & { sessionId: number };

const EMPTY = new Map<string, RecordingPinStatus>();

/** Each pinned annotation's status in the preview session, by annotation id. */
export function useRecordingPinStatus(
  sessionId: number | null,
): ReadonlyMap<string, RecordingPinStatus> {
  // Kept with the session it was heard in, so a new session starts empty
  // without clearing anything by hand.
  const [heard, setHeard] = useState<{
    session: number | null;
    statuses: ReadonlyMap<string, RecordingPinStatus>;
  }>({ session: null, statuses: EMPTY });
  useEffect(() => {
    if (sessionId === null) return;
    let disposed = false;
    let stop: (() => void) | null = null;
    const update = (
      change: (
        statuses: Map<string, RecordingPinStatus>,
      ) => Map<string, RecordingPinStatus>,
    ) => {
      setHeard((previous) => ({
        session: sessionId,
        statuses: change(
          new Map(previous.session === sessionId ? previous.statuses : EMPTY),
        ),
      }));
    };
    // A retrack starts by reporting no stretches, so the last path's never
    // stand over the new one while it is worked out.
    const take = ({ annotationId, ...status }: PinStatusPayload) => {
      update((statuses) => statuses.set(annotationId, status));
    };
    void listen<PinStatusEvent>(
      "editor://recording-pin-status",
      ({ payload }) => {
        if (!disposed && payload.sessionId === sessionId) take(payload);
      },
    )
      .then((unlisten) => {
        if (disposed) unlisten();
        else stop = unlisten;
      })
      .catch((cause: unknown) => {
        console.error("Could not listen for pinned annotations", cause);
      });
    // Paths that landed before this listener was in place.
    void invoke<PinStatusPayload[]>("recording_preview_pin_statuses", {
      sessionId,
    })
      .then((landed) => {
        if (disposed) return;
        update((statuses) => {
          for (const { annotationId, ...status } of landed)
            if (!statuses.has(annotationId)) statuses.set(annotationId, status);
          return statuses;
        });
      })
      .catch((cause: unknown) => {
        console.error("Could not read pinned annotations", cause);
      });
    return () => {
      disposed = true;
      stop?.();
    };
  }, [sessionId]);
  return heard.session === sessionId ? heard.statuses : EMPTY;
}
