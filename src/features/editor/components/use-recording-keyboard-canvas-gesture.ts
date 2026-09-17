// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";

import { keyboardSelectionGeometry } from "../keyboard-effect-geometry";
import {
  moveRecordingKeyboardShortcutFragments,
  resetAllRecordingKeyboardShortcutPositions,
  resetRecordingKeyboardShortcutPositions,
  resizeRecordingKeyboardShortcutFragments,
} from "../recording-keyboard-timeline-edit";
import { RecordingTimelineEdit } from "../recording-timeline-edit";
import { KeyboardEffectSettings } from "../types";
import { useEditorEditGesture } from "../use-editor-edit-history";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";
import type { TimedLaneFragment, TimedLaneItem } from "./timed-lane-layout";
import type { TimelineItemSelection } from "./timeline-item-selection";

export const KEYBOARD_LAYER_ID = 0xfffffffe;

/**
 * Placement of the keyboard shortcut drawn over the preview: the drag on the
 * canvas and the Selection panel's fields commit through the same edits.
 */
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
  const editGesture = useEditorEditGesture();
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
  /** The fragments a panel edit acts on: whatever is selected, or the one on
   * screen when the selection has drifted away from it. */
  const editedIds = useCallback(() => {
    const selected = selectedIdsRef.current;
    if (selected.size > 0) return selected;
    return new Set(visibleFragment ? [visibleFragment.fragmentId] : []);
  }, [visibleFragment]);
  /**
   * The Selection panel's own size and position fields, committed through the
   * very edit functions the drag on the shortcut uses, so a number typed there
   * and a drag out here are the same edit.
   */
  const applyPlacement = useCallback(
    (placement: {
      positionXPercent?: number;
      positionYPercent?: number;
      sizePercent?: number;
    }) => {
      if (!geometry || !onChange) return;
      const ids = editedIds();
      if (ids.size === 0) return;
      const center = {
        x:
          placement.positionXPercent === undefined
            ? geometry.center.x
            : placement.positionXPercent / 100,
        y:
          placement.positionYPercent === undefined
            ? geometry.center.y
            : placement.positionYPercent / 100,
      };
      let next = edit;
      if (placement.sizePercent !== undefined) {
        next = resizeRecordingKeyboardShortcutFragments({
          center,
          edit: next,
          fragmentIds: ids,
          maximumSizePercent: geometry.maximumSizePercent,
          minimumSizePercent: geometry.minimumSizePercent,
          sizePercent: placement.sizePercent,
        });
      }
      if (
        placement.positionXPercent !== undefined ||
        placement.positionYPercent !== undefined
      ) {
        next = moveRecordingKeyboardShortcutFragments({
          bounds: geometry.rect,
          delta: {
            x: center.x - geometry.center.x,
            y: center.y - geometry.center.y,
          },
          edit: next,
          fragmentIds: ids,
          leaderCenter: geometry.center,
        });
      }
      if (next === edit) return;
      // Not a gesture of its own: a slider in the panel arrives one value at
      // a time, and the history groups a run of edits to the same key into
      // one step, the way it does for every other panel field. Opening a
      // gesture per value would close that group each time.
      onChange(next);
    },
    [edit, editedIds, geometry, onChange],
  );
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
        // A press that never travelled is a selection, not a placement: an
        // explicit position written for it would mark the shortcut as
        // adjusted in the timeline while it sits exactly where it was.
        const untouched =
          event.deltaX === 0 &&
          event.deltaY === 0 &&
          (event.operation !== "resize" || event.scale === 1);
        if (untouched) {
          gestureRef.current = null;
          requestAnimationFrame(editGesture.endGesture);
          return true;
        }
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
    [edit, editGesture, geometry, onChange, selectVisible, visibleFragment],
  );
  return { applyGesture, applyPlacement, applyToAll, reset, selectVisible };
}
