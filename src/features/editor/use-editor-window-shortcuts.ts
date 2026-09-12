// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { preserveEscapeFocus } from "./escape-focus";
import {
  ownsActivationKeys,
  ownsArrowKeys,
  ownsPopupInteractionKeys,
  ownsTextEditingKeys,
} from "./keyboard-target";

/**
 * How many mounted hooks have claimed Escape for a tool that is in the middle
 * of something - the crop tool, today.
 *
 * Module state because the several shortcut hooks on one editor window are
 * siblings on the same listener target: `stopPropagation` cannot hold one back
 * from another, and which of them registered first is an accident of render
 * order. A claim is asked about at event time instead, so a claimed Escape
 * leaves the tool and every plain deselect on the window stands down.
 */
let escapeClaims = 0;

const arrowDirections = new Map([
  ["ArrowDown", { x: 0, y: 1 }],
  ["ArrowLeft", { x: -1, y: 0 }],
  ["ArrowRight", { x: 1, y: 0 }],
  ["ArrowUp", { x: 0, y: -1 }],
]);

export function useEditorWindowShortcuts({
  onConfirm,
  onCopy,
  onCutTimeline,
  onDelete,
  onDeselect,
  onExport,
  onMoveBackward,
  onMoveForward,
  onNudge,
  onRecenter,
  onRedo,
  onResizeCanvas,
  onSelectTool,
  onStep,
  onToggleBladeTool,
  onToggleCrop,
  onToggleCursorPanel,
  onTogglePlayback,
  onToggleRangeTool,
  onUndo,
  ownsEscape = false,
}: {
  /** Enter: done with whatever this window is in the middle of. */
  onConfirm?: () => void;
  onCopy?: () => void;
  onCutTimeline?: () => void;
  onDelete?: () => void;
  onDeselect?: () => void;
  onExport?: () => void;
  onMoveBackward?: () => void;
  onMoveForward?: () => void;
  /** Moves the selected layer by one arrow press; `coarse` is the Shift jump. */
  onNudge?: (directionX: number, directionY: number, coarse: boolean) => void;
  onRecenter?: () => void;
  onRedo?: () => void;
  onResizeCanvas?: () => void;
  onSelectTool?: () => void;
  /** Moves the playhead by one arrow press; `coarse` is the Shift jump. */
  onStep?: (direction: -1 | 1, coarse: boolean) => void;
  onToggleBladeTool?: () => void;
  onToggleCrop?: () => void;
  /** M: the cursor panel, on or away. */
  onToggleCursorPanel?: () => void;
  onTogglePlayback?: () => void;
  onToggleRangeTool?: () => void;
  onUndo?: () => void;
  /** This hook's Escape outranks every plain `onDeselect` on the window:
   * leaving the tool in hand comes before clearing a selection under it. */
  ownsEscape?: boolean;
}) {
  const focusIntentRef = useRef<"keyboard" | "pointer">("keyboard");
  useEffect(() => {
    if (ownsEscape) escapeClaims += 1;
    return () => {
      if (ownsEscape) escapeClaims -= 1;
    };
  }, [ownsEscape]);
  const consumedKeysRef = useRef(new Set<string>());
  useEffect(preserveEscapeFocus, []);

  useEffect(() => {
    const consumedKeys = consumedKeysRef.current;
    const consume = (event: KeyboardEvent) => {
      event.preventDefault();
      // Stop before document-level navigation and React Aria modality tracking.
      // Other editor handlers on this window still receive the event.
      event.stopPropagation();
      consumedKeys.add(event.code);
    };
    const onKeyUp = (event: KeyboardEvent) => {
      if (consumedKeys.delete(event.code)) event.stopPropagation();
    };
    const onPointerDown = () => {
      focusIntentRef.current = "pointer";
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        event.repeat &&
        consumedKeys.has(event.code) &&
        !arrowDirections.has(event.code)
      ) {
        consume(event);
        return;
      }
      if (
        event.code === "Tab" ||
        (arrowDirections.has(event.code) && ownsArrowKeys(event.target))
      ) {
        focusIntentRef.current = "keyboard";
      }

      // Arrows run before the shared guards: holding one has to repeat, and
      // Shift only picks the bigger jump rather than naming another shortcut.
      const arrow = arrowDirections.get(event.code);
      if (arrow) {
        if (
          event.isComposing ||
          event.altKey ||
          event.ctrlKey ||
          event.metaKey ||
          ownsTextEditingKeys(event.target) ||
          ownsArrowKeys(event.target)
        )
          return;
        const handled = onNudge
          ? () => {
              onNudge(arrow.x, arrow.y, event.shiftKey);
            }
          : onStep && arrow.x !== 0
            ? () => {
                onStep(arrow.x > 0 ? 1 : -1, event.shiftKey);
              }
            : null;
        if (!handled) return;
        consume(event);
        handled();
        return;
      }

      if (event.repeat || event.isComposing || event.altKey) return;

      if (event.code === "Escape" && !ownsTextEditingKeys(event.target)) {
        // A claimed Escape is the only one that runs: the hook that owns it
        // takes the event, and the plain deselects elsewhere on the window
        // let it through rather than racing it.
        if (onDeselect && (ownsEscape || escapeClaims === 0)) {
          consume(event);
          onDeselect();
        }
        if (ownsEscape || escapeClaims > 0) return;
      }

      if (
        (event.code === "Enter" || event.code === "NumpadEnter") &&
        onConfirm &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onConfirm();
        return;
      }

      const commandKey = event.ctrlKey || event.metaKey;
      if (
        commandKey &&
        event.code === "KeyZ" &&
        !ownsTextEditingKeys(event.target)
      ) {
        const action = event.shiftKey ? onRedo : onUndo;
        if (action) {
          consume(event);
          action();
        }
        return;
      }
      if (commandKey && !event.shiftKey) {
        if (event.code === "KeyB" && onCutTimeline) {
          if (ownsTextEditingKeys(event.target)) return;
          consume(event);
          onCutTimeline();
        } else if (event.code === "KeyC" && onCopy) {
          if (ownsTextEditingKeys(event.target)) return;
          consume(event);
          onCopy();
        } else if (event.code === "KeyE" && onExport) {
          consume(event);
          onExport();
        }
        return;
      }

      if (
        event.shiftKey &&
        !commandKey &&
        event.code === "KeyR" &&
        onToggleRangeTool &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onToggleRangeTool();
        return;
      }

      if (event.ctrlKey || event.metaKey || event.shiftKey) return;

      if (
        event.code === "Space" &&
        onTogglePlayback &&
        !ownsTextEditingKeys(event.target)
      ) {
        if (
          ownsPopupInteractionKeys(event.target) ||
          (focusIntentRef.current === "keyboard" &&
            ownsActivationKeys(event.target))
        ) {
          return;
        }
        consume(event);
        onTogglePlayback();
        return;
      }

      if (
        (event.code === "Backspace" || event.code === "Delete") &&
        onDelete &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onDelete();
        return;
      }

      if (
        event.code === "BracketLeft" &&
        onMoveForward &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onMoveForward();
        return;
      }

      if (
        event.code === "BracketRight" &&
        onMoveBackward &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onMoveBackward();
        return;
      }

      if (
        event.code === "KeyB" &&
        onToggleBladeTool &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onToggleBladeTool();
      } else if (
        event.code === "KeyP" &&
        onTogglePlayback &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onTogglePlayback();
      } else if (
        event.code === "KeyR" &&
        onRecenter &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onRecenter();
      } else if (
        event.code === "KeyF" &&
        onResizeCanvas &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onResizeCanvas();
      } else if (
        event.code === "KeyC" &&
        onToggleCrop &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onToggleCrop();
      } else if (
        event.code === "KeyV" &&
        onSelectTool &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onSelectTool();
      } else if (
        event.code === "KeyM" &&
        onToggleCursorPanel &&
        !ownsTextEditingKeys(event.target)
      ) {
        consume(event);
        onToggleCursorPanel();
      }
    };

    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
    };
  }, [
    onConfirm,
    onCopy,
    onCutTimeline,
    onDelete,
    onDeselect,
    onExport,
    onMoveBackward,
    onMoveForward,
    onNudge,
    onRecenter,
    onRedo,
    onResizeCanvas,
    onSelectTool,
    onStep,
    onToggleCrop,
    onToggleCursorPanel,
    onToggleBladeTool,
    onTogglePlayback,
    onToggleRangeTool,
    onUndo,
    ownsEscape,
  ]);
}
