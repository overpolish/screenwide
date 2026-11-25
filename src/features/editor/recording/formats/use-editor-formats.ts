// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useMemo, useRef, useState } from "react";

import {
  type CameraOutput,
  exportedRecordingEdits,
} from "../../export/camera-output";

import { withLookCarried } from "./format-look-carry";
import { formatSaveJobs } from "./format-save-jobs";
import { withOwnHere } from "./format-sharing";
import { formatsOfShape, openedFormats } from "./recording-formats";
import { useFormatEdit } from "./use-format-edit";
import {
  type FrameFormatPatch,
  useRecordingFormats,
} from "./use-recording-formats";

import type { FormatDocument } from "./format-document";
import type { EditorArtifact } from "../../../../bindings/EditorArtifact";
import type {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/** The recording's formats for the editor window: the control the format
 * bar and the Frame island use (`useRecordingFormats`), the timeline worked
 * out for the format in hand with its way back (`useFormatEdit`), and what
 * an export writes: the format in hand, or every format, one file each. */
export function useEditorFormats(
  inputs: Parameters<typeof useRecordingFormats>[0] & {
    artifact: EditorArtifact | null;
    cameraOutput: CameraOutput | null;
    commit: (edit: RecordingTimelineEdit) => void;
    edit: RecordingTimelineEdit | null;
  },
) {
  const { artifact, cameraOutput, recordingOutput } = inputs;
  // Read as a project opens them, so one saved before the main canvas took
  // no platform shows and writes it free of them.
  const formats = useMemo(
    () => openedFormats(inputs.formats),
    [inputs.formats],
  );
  const control = useRecordingFormats({ ...inputs, formats });
  // Every format is written until this is turned off: a second format is
  // made to be published too.
  const [savesAll, setSavesAll] = useState(true);
  const timeline = useFormatEdit({
    canvas: recordingOutput.primary,
    commit: inputs.commit,
    edit: inputs.edit,
    formats,
    onFormatsChange: inputs.onFormatsChange,
    scope: control?.snapshot.scope ?? "all",
  });
  // Memoized: the edits are handed on to the Export island's compare and the
  // export, which would otherwise see new ones on every render.
  const exportEdits = useMemo(
    () =>
      exportedRecordingEdits({
        artifact,
        cameraOutput,
        recordingOutput,
        timelineEdit: timeline.edit,
      }),
    [artifact, cameraOutput, recordingOutput, timeline.edit],
  );
  // The look and the formats as last set, so changes made faster than the
  // window renders (a dial dragged) each start from the one before.
  const latestRef = useRef({
    formats,
    look: { cameraOverlay: inputs.cameraOverlay, recordingOutput },
  });
  useLayoutEffect(() => {
    latestRef.current = {
      formats,
      look: { cameraOverlay: inputs.cameraOverlay, recordingOutput },
    };
  }, [inputs.cameraOverlay, formats, recordingOutput]);
  const scope = control?.snapshot.scope ?? "all";
  /** A change made to the screen or the camera of the shape in hand,
   * carried to the other shapes from the main one (`withLookCarried`); on
   * a platform, any change to its screen or its background sets that its
   * own way there. */
  const editLook = (change: Partial<FormatDocument>) => {
    const { formats, look: before } = latestRef.current;
    const after = { ...before, ...change };
    const carried =
      scope === "shape"
        ? formats && withOwnHere(formats, { after, before })
        : withLookCarried(formats, { after, before }, scope);
    latestRef.current = { formats: carried ?? formats, look: after };
    if (change.recordingOutput)
      inputs.onRecordingOutputChange(change.recordingOutput);
    if (change.cameraOverlay)
      inputs.onCameraOverlayChange(change.cameraOverlay);
    if (carried) inputs.onFormatsChange(carried);
  };
  return {
    control: control && {
      onPatch: (patch: FrameFormatPatch) => {
        if (patch.kind === "sync") timeline.sync(patch.action, patch.id);
        else control.onPatch(patch);
      },
      snapshot: {
        ...control.snapshot,
        differs: timeline.differs,
        syncOf: timeline.syncOf,
        versions: timeline.versions,
      },
    },
    /** The camera's overlay set, as `editLook` carries it. */
    editCamera: (cameraOverlay: FormatDocument["cameraOverlay"]) => {
      editLook({ cameraOverlay });
    },
    /** One of the format in hand's layers set, as `editLook` carries it.
     * A recording's canvas has no rounded corners of its own. */
    editOutput: (
      trackId: keyof RecordingOutputSettings,
      settings: ScreenshotOutputSettings,
    ) => {
      editLook({
        recordingOutput: {
          ...latestRef.current.look.recordingOutput,
          [trackId]: { ...settings, backgroundRadiusPercent: 0 },
        },
      });
    },
    /** The format in hand's canvas and timeline as the export reads them. */
    exportEdits,
    /** The files an export writes now under `fileStem`, each with its
     * canvas and timeline as the export reads them. */
    saveJobs: (fileStem: string) =>
      formatSaveJobs({
        document: { cameraOverlay: inputs.cameraOverlay, recordingOutput },
        edit: timeline.edit,
        fileStem,
        formats,
        isAll: savesAll,
        stored: inputs.edit,
      }).map((job) => ({
        ...job,
        ...exportedRecordingEdits({ artifact, cameraOutput, ...job }),
      })),
    /** Whether an export writes every format; null with one format. */
    savesAll:
      (formats?.list.flatMap(formatsOfShape).length ?? 0) > 1 ? savesAll : null,
    setSavesAll,
    timeline,
  };
}
