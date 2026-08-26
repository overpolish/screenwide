// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { getRecordingKeyboardTimeline } from "./recording-keyboard-timeline-api";
import { RecordingKeyboardTimelineItem } from "./types";

const EMPTY_ITEMS: RecordingKeyboardTimelineItem[] = [];
export function useRecordingKeyboardTimeline(
  artifactId: number,
  enabled: boolean,
) {
  const [items, setItems] = useState(EMPTY_ITEMS);

  useEffect(() => {
    let active = true;
    if (!enabled) {
      return () => {
        active = false;
      };
    }
    void getRecordingKeyboardTimeline(artifactId)
      .then((next) => {
        if (active) setItems(next);
      })
      .catch((cause: unknown) => {
        console.error("Could not load keyboard shortcut timeline", cause);
      });
    return () => {
      active = false;
    };
  }, [artifactId, enabled]);

  return items;
}
