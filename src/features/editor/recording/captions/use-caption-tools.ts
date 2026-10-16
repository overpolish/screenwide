// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useRef } from "react";

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { EditorArtifact } from "../../types";

import {
  CaptionsPatch,
  DEFAULT_CAPTION_SETTINGS,
  recordingCaptions,
  withCaptions,
} from "./recording-captions";
import {
  CaptionDefaultsPatch,
  captionDefaultsSettings,
  defaultCaptions,
  useTextTemplates,
} from "./use-text-templates";

import type { GeneralSettings } from "../../../../bindings/GeneralSettings";
import type { TextLayout } from "../../../../bindings/TextLayout";
import type { TextStyle } from "../../../../bindings/TextStyle";

/** No caption template saved as a default: one list, so the templates are
 * only worked out again when the settings change. */
const NO_CAPTION_STYLES: TextStyle[] = [];
/** No layout option kept for new captions, as one object for the same
 * reason. */
const NO_CAPTION_LAYOUT: TextLayout = {};

/**
 * The Captions tool for the recording open, where it has a microphone to
 * caption: what the panel shows, and its changes. They join the edit history
 * as any other setting does, so a slider's run of changes is one step.
 * Captions the edit has not set yet start from the defaults: the template,
 * its look and the rules kept in the general settings.
 */
export function useCaptionTools({
  artifact,
  edit,
  general,
  onEditChange,
  onGeneralChange,
}: {
  artifact: EditorArtifact | null;
  edit: RecordingTimelineEdit | null | undefined;
  general: GeneralSettings | null | undefined;
  onGeneralChange: (change: Partial<GeneralSettings>) => void;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
}) {
  const editRef = useRef(edit);
  editRef.current = edit;
  const savedStyles = general?.captionStyles ?? NO_CAPTION_STYLES;
  const templates = useTextTemplates(savedStyles, {
    layout: general?.captionLayout ?? NO_CAPTION_LAYOUT,
    settings: general?.captionSettings ?? DEFAULT_CAPTION_SETTINGS,
    template: general?.captionTemplate ?? "box",
  });
  const hasMicrophone =
    artifact?.kind === "recording" &&
    artifact.audioTracks.some((track) => track.kind === "microphone");
  const defaults = useMemo(
    () => (templates ? defaultCaptions(templates) : null),
    [templates],
  );
  const snapshot = useMemo(
    () =>
      hasMicrophone && edit && defaults
        ? recordingCaptions(edit, defaults)
        : null,
    [defaults, edit, hasMicrophone],
  );
  const change = (patch: CaptionsPatch) => {
    const current = editRef.current;
    if (!onEditChange || !current || !defaults) return;
    // Read back from the ref, not the render, so two changes sent together
    // build on each other rather than the second undoing the first.
    const next = withCaptions(current, patch, defaults);
    editRef.current = next;
    onEditChange(next);
  };
  const changeDefaults = (patch: CaptionDefaultsPatch) => {
    onGeneralChange(captionDefaultsSettings(savedStyles, patch));
  };
  return { change, changeDefaults, snapshot, templates };
}
