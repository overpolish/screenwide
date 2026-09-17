// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { keyboardSelectionGeometry } from "../keyboard-effect-geometry";
import { recordingKeyboardShortcutPositions } from "../recording-keyboard-timeline-edit";
import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";
import { KeyboardEffectSettings } from "../types";

import { layoutTimedLaneItems } from "./timed-lane-layout";
import {
  KEYBOARD_LAYER_ID,
  useRecordingKeyboardCanvasEditing,
} from "./use-recording-keyboard-canvas-gesture";
import { useRecordingKeyboardTimelineEditing } from "./use-recording-keyboard-timeline-editing";

export function useRecordingKeyboardPreviewEditing({
  artifactId,
  canvasTool,
  durationMs,
  edit,
  enabled,
  keyboardEffects,
  maximumWidthUnits,
  onChange,
  onKeyboardEffectsChange,
  onSelectionStart,
  output,
  positionMs,
}: {
  artifactId: number;
  canvasTool: string | null;
  durationMs: number;
  edit: RecordingTimelineEdit | null | undefined;
  enabled: boolean;
  keyboardEffects: KeyboardEffectSettings;
  output: { height: number; width: number };
  positionMs: number;
  maximumWidthUnits?: number | null;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onSelectionStart?: () => void;
}) {
  const effectiveEdit = useMemo(
    () => edit ?? createRecordingTimelineEdit(artifactId),
    [artifactId, edit],
  );
  const timeline = useRecordingKeyboardTimelineEditing({
    artifactId,
    edit: effectiveEdit,
    enabled,
    onChange,
    onSelectionStart,
    sourceDurationMs: durationMs,
  });
  const fragments = useMemo(
    () =>
      layoutTimedLaneItems({
        edit: effectiveEdit,
        items: timeline.items,
        sourceDurationMs: durationMs,
      }),
    [durationMs, effectiveEdit, timeline.items],
  );
  const retainedDurationMs =
    durationMs * recordingTimelineRetainedDuration(effectiveEdit);
  const outputPosition =
    retainedDurationMs > 0 ? positionMs / retainedDurationMs : 0;
  const visibleFragment = useMemo(
    () =>
      fragments
        .filter(
          (fragment) =>
            !timeline.hiddenFragmentIds.has(fragment.fragmentId) &&
            !timeline.hiddenItemIds.has(fragment.item.id) &&
            outputPosition >= fragment.outputStart &&
            outputPosition < fragment.outputEnd,
        )
        .sort((a, b) => b.item.startMs - a.item.startMs)
        .find((_, index) => index === 0) ?? null,
    [
      fragments,
      outputPosition,
      timeline.hiddenFragmentIds,
      timeline.hiddenItemIds,
    ],
  );
  const position = visibleFragment
    ? (recordingKeyboardShortcutPositions(effectiveEdit).find(
        (candidate) =>
          candidate.shortcutId === visibleFragment.item.id &&
          candidate.segmentId === visibleFragment.segmentId,
      ) ?? null)
    : null;
  const geometry = keyboardSelectionGeometry({
    height: output.height,
    maximumWidthUnits,
    position,
    positionXPercent: keyboardEffects.positionXPercent,
    positionYPercent: keyboardEffects.positionYPercent,
    sizePercent: keyboardEffects.sizePercent,
    width: output.width,
  });
  const canvas = useRecordingKeyboardCanvasEditing({
    edit: effectiveEdit,
    geometry,
    keyboardEffects,
    onChange,
    onKeyboardEffectsChange,
    selection: timeline.selection,
    visibleFragment,
  });
  const selection =
    canvasTool === "select" &&
    keyboardEffects.bake &&
    visibleFragment &&
    geometry
      ? {
          layerId: KEYBOARD_LAYER_ID,
          maximumScale: geometry.maximumSizePercent / geometry.sizePercent,
          minimumScale: geometry.minimumSizePercent / geometry.sizePercent,
          paneIndex: 0,
          radiusPercent: 0,
          rect: geometry.rect,
        }
      : null;
  return {
    canvas,
    effectiveEdit,
    geometry,
    selection,
    timeline,
    visibleFragment,
  };
}
