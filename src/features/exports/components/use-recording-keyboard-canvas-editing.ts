// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useRef } from "react";

import { keyboardSelectionGeometry } from "../keyboard-effect-geometry";
import {
  moveRecordingKeyboardShortcutFragments,
  recordingKeyboardShortcutPositions,
  resetAllRecordingKeyboardShortcutPositions,
  resetRecordingKeyboardShortcutPositions,
  resizeRecordingKeyboardShortcutFragments,
} from "../recording-keyboard-timeline-edit";
import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";
import { KeyboardEffectSettings } from "../types";
import { useExportEditGesture } from "../use-export-edit-history";

import { layoutTimedLaneItems } from "./timed-lane-layout";
import { useRecordingKeyboardTimelineEditing } from "./use-recording-keyboard-timeline-editing";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";
import type { TimedLaneFragment, TimedLaneItem } from "./timed-lane-layout";
import type { TimelineItemSelection } from "./timeline-item-selection";

export const KEYBOARD_LAYER_ID = 0xfffffffe;

export function useRecordingKeyboardCanvasEditing<Item extends TimedLaneItem>({
  edit,
  geometry,
  keyboardEffects,
  onChange,
  onKeyboardEffectsChange,
  selection,
  visibleFragment,
}: {
  edit: RecordingTimelineEdit;
  geometry: NonNullable<ReturnType<typeof keyboardSelectionGeometry>> | null;
  keyboardEffects: KeyboardEffectSettings;
  selection: TimelineItemSelection<string>;
  visibleFragment: TimedLaneFragment<Item> | null;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
}) {
  const editGesture = useExportEditGesture();
  const selectedIdsRef = useRef(selection.ids);
  selectedIdsRef.current = selection.ids;
  const gestureRef = useRef<{
    edit: RecordingTimelineEdit;
    ids: ReadonlySet<string>;
    leaderCenter: { x: number; y: number };
    leaderSizePercent: number;
  } | null>(null);
  const selectVisible = useCallback(() => {
    if (!visibleFragment) return;
    if (!selectedIdsRef.current.has(visibleFragment.fragmentId)) {
      const next = new Set([visibleFragment.fragmentId]);
      selectedIdsRef.current = next;
      selection.onSelect(visibleFragment.fragmentId, false);
    }
  }, [selection, visibleFragment]);
  const reset = useCallback(() => {
    if (!onChange || selectedIdsRef.current.size === 0) return;
    const next = resetRecordingKeyboardShortcutPositions(
      edit,
      selectedIdsRef.current,
    );
    if (next === edit) return;
    editGesture.beginGesture();
    onChange(next);
    editGesture.endGesture();
  }, [edit, editGesture, onChange]);
  const applyToAll = useCallback(() => {
    if (!geometry || !onChange || !onKeyboardEffectsChange) return;
    editGesture.beginGesture();
    onKeyboardEffectsChange({
      ...keyboardEffects,
      positionXPercent: geometry.center.x * 100,
      positionYPercent: geometry.center.y * 100,
      sizePercent: geometry.sizePercent,
    });
    onChange(resetAllRecordingKeyboardShortcutPositions(edit));
    editGesture.endGesture();
  }, [
    edit,
    editGesture,
    geometry,
    keyboardEffects,
    onChange,
    onKeyboardEffectsChange,
  ]);
  const applyGesture = useCallback(
    (event: RecordingSelectionGestureEvent) => {
      if (event.paneIndex !== KEYBOARD_LAYER_ID) return false;
      if (event.operation === "resetAction") {
        if (event.phase === "begin") reset();
        return true;
      }
      if (event.operation === "applyToAllAction") {
        if (event.phase === "begin") applyToAll();
        return true;
      }
      if (
        (event.operation !== "move" && event.operation !== "resize") ||
        !geometry ||
        !onChange
      )
        return true;
      if (event.phase === "begin") {
        selectVisible();
        const ids = selectedIdsRef.current.has(
          visibleFragment?.fragmentId ?? "",
        )
          ? selectedIdsRef.current
          : new Set(visibleFragment ? [visibleFragment.fragmentId] : []);
        if (ids.size === 0) return true;
        gestureRef.current = {
          edit,
          ids,
          leaderCenter: geometry.center,
          leaderSizePercent: geometry.sizePercent,
        };
        editGesture.beginGesture();
        return true;
      }
      const active = gestureRef.current;
      if (!active) return true;
      if (event.phase === "cancel") {
        gestureRef.current = null;
        requestAnimationFrame(editGesture.endGesture);
        return true;
      }
      if (event.phase === "end") {
        const next =
          event.operation === "resize"
            ? resizeRecordingKeyboardShortcutFragments({
                center: {
                  x:
                    geometry.rect.x +
                    event.deltaX +
                    (geometry.rect.width * event.scale) / 2,
                  y:
                    geometry.rect.y +
                    event.deltaY +
                    (geometry.rect.height * event.scale) / 2,
                },
                edit: active.edit,
                fragmentIds: active.ids,
                maximumSizePercent: geometry.maximumSizePercent,
                minimumSizePercent: geometry.minimumSizePercent,
                sizePercent: active.leaderSizePercent * event.scale,
              })
            : moveRecordingKeyboardShortcutFragments({
                bounds: geometry.rect,
                delta: { x: event.deltaX, y: event.deltaY },
                edit: active.edit,
                fragmentIds: active.ids,
                leaderCenter: active.leaderCenter,
              });
        onChange(next);
        gestureRef.current = null;
        requestAnimationFrame(editGesture.endGesture);
      }
      return true;
    },
    [
      edit,
      editGesture,
      geometry,
      onChange,
      applyToAll,
      reset,
      selectVisible,
      visibleFragment,
    ],
  );
  return { applyGesture, applyToAll, reset, selectVisible };
}

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
          recenterMode: false,
          rect: geometry.rect,
        }
      : null;
  return { canvas, effectiveEdit, selection, timeline, visibleFragment };
}
