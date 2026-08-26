// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useRef, useState } from "react";

import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";
import { persistRecordingTimelineEdit } from "./recording-timeline-edit-api";
import { ExportArtifact } from "./types";

export function useRecordingTimelineEditState(artifact: ExportArtifact | null) {
  const [storedEdit, setStoredEdit] = useState<RecordingTimelineEdit | null>(
    null,
  );
  const artifactId = artifact?.kind === "recording" ? artifact.id : null;
  const initialEdit = useMemo(
    () =>
      artifact?.kind === "recording"
        ? (artifact.timelineEdit ?? createRecordingTimelineEdit(artifact.id))
        : null,
    // The export window owns one editing session per artifact. Keeping this
    // fallback stable is important: downstream timeline effects use identity
    // to distinguish a real edit from an ordinary React render.
    // eslint-disable-next-line @eslint-react/exhaustive-deps
    [artifactId],
  );
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
  const update = useCallback((next: RecordingTimelineEdit | null) => {
    setStoredEdit(next);
    if (!next) return;
    revisionRef.current.value += 1;
    void persistRecordingTimelineEdit(
      next.artifactId,
      revisionRef.current.value,
      next,
    ).catch((cause: unknown) => {
      console.error("Could not persist recording timeline edit", cause);
    });
  }, []);
  return [edit, update] as const;
}
