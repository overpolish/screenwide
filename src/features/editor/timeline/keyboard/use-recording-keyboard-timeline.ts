// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { RecordingKeyboardTimelineItem } from "../../types";
import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";

import {
  getRecordingKeyboardTimeline,
  recordingPreviewKeyboardDeletions,
} from "./recording-keyboard-timeline-api";

const EMPTY_ITEMS: RecordingKeyboardTimelineItem[] = [];

/**
 * Deletions and manual placements decide badge continuity and therefore each
 * lane's real exit time, so the lane reloads whenever the edit changes. The
 * edit the items were computed against is returned alongside them: until a
 * reload lands, callers hide by the union of the live and applied edits so
 * an undone deletion cannot flash stale lane geometry.
 */
export function useRecordingKeyboardTimeline({
  artifactId,
  edit,
  enabled,
  sourceDurationMs,
}: {
  artifactId: number;
  edit: RecordingTimelineEdit;
  enabled: boolean;
  sourceDurationMs: number;
}) {
  const [state, setState] = useState<{
    appliedEdit: RecordingTimelineEdit | null;
    items: RecordingKeyboardTimelineItem[];
  }>({ appliedEdit: null, items: EMPTY_ITEMS });

  useEffect(() => {
    let active = true;
    if (!enabled) {
      return () => {
        active = false;
      };
    }
    void getRecordingKeyboardTimeline(
      artifactId,
      recordingPreviewKeyboardDeletions(edit, sourceDurationMs),
    )
      .then((next) => {
        if (active) setState({ appliedEdit: edit, items: next });
      })
      .catch((cause: unknown) => {
        console.error("Could not load keyboard shortcut timeline", cause);
      });
    return () => {
      active = false;
    };
  }, [artifactId, edit, enabled, sourceDurationMs]);

  return state;
}
