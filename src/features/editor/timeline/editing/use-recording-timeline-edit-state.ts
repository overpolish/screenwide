// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useRef, useState } from "react";

import { withSnappedScenes } from "../../recording/recording-captures";
import {
  recordOverlayTimeline,
  RecordingOverlayTimelines,
  retimeRecordingOverlays,
} from "../../recording/recording-overlay-retime";
import { EditorArtifact } from "../../types";

import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";
import { persistRecordingTimelineEdit } from "./recording-timeline-edit-api";

/**
 * The recording's timeline edit and two ways to change it: `update` stores an
 * edit as it is, for undo and redo, which bring back clips already timed on
 * their own timeline; `commit` stores an edit made from the current one, with
 * its annotations and scenes carried onto the timeline it lays out.
 */
export function useRecordingTimelineEditState(artifact: EditorArtifact | null) {
  const [storedEdit, setStoredEdit] = useState<RecordingTimelineEdit | null>(
    null,
  );
  const artifactId = artifact?.kind === "recording" ? artifact.id : null;
  const timelinesRef = useRef<RecordingOverlayTimelines>(new WeakMap());
  const sourceDurationMs =
    artifact?.kind === "recording" ? artifact.durationMs : 0;
  const initialEdit = useMemo(() => {
    if (artifact?.kind !== "recording") return null;
    // Paddings and corners saved as percents open as the looks in points
    // they were nearest.
    const opened = artifact.timelineEdit
      ? withSnappedScenes(artifact.timelineEdit, artifact)
      : createRecordingTimelineEdit(artifact.id);
    // A project saved with a scene over a gap opens with it cut back to
    // either side, as an edit would leave it.
    return retimeRecordingOverlays(
      { next: opened, previous: opened },
      { sourceDurationMs, timelines: timelinesRef.current },
    );
    // The editor window owns one editing session per artifact. Keeping this
    // fallback stable is important: downstream timeline effects use identity
    // to distinguish a real edit from an ordinary React render.
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [artifactId]);
  const revisionRef = useRef({
    artifactId,
    value:
      artifact?.kind === "recording" ? (artifact.timelineEditRevision ?? 0) : 0,
  });
  if (revisionRef.current.artifactId !== artifactId) {
    revisionRef.current = {
      artifactId,
      value:
        artifact?.kind === "recording"
          ? (artifact.timelineEditRevision ?? 0)
          : 0,
    };
  }
  const edit =
    artifact?.kind === "recording"
      ? storedEdit?.artifactId === artifact.id
        ? storedEdit
        : initialEdit
      : null;
  const latestRef = useRef(edit);
  latestRef.current = edit;
  const update = useCallback((next: RecordingTimelineEdit | null) => {
    latestRef.current = next;
    setStoredEdit(next);
    if (!next) return;
    recordOverlayTimeline(next, timelinesRef.current);
    revisionRef.current.value += 1;
    persistRecordingTimelineEdit(
      next.artifactId,
      revisionRef.current.value,
      next,
    );
  }, []);
  const commit = useCallback(
    (next: RecordingTimelineEdit) => {
      const previous = latestRef.current;
      update(
        previous
          ? retimeRecordingOverlays(
              { next, previous },
              { sourceDurationMs, timelines: timelinesRef.current },
            )
          : next,
      );
    },
    [sourceDurationMs, update],
  );
  return [edit, update, commit] as const;
}
