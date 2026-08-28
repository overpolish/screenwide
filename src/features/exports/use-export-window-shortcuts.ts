// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import {
  ownsActivationKeys,
  ownsArrowKeys,
  ownsPopupInteractionKeys,
  ownsTextEditingKeys,
} from "./keyboard-target";

const arrowDirections = new Map([
  ["ArrowDown", { x: 0, y: 1 }],
  ["ArrowLeft", { x: -1, y: 0 }],
  ["ArrowRight", { x: 1, y: 0 }],
  ["ArrowUp", { x: 0, y: -1 }],
]);

export function useExportWindowShortcuts({
  onCopy,
  onCutTimeline,
  onDelete,
  onDeselect,
  onExport,
  onMoveBackward,
  onMoveForward,
  onNudge,
  onRecenter,
  onResizeCanvas,
  onSelectTool,
  onStep,
  onToggleBladeTool,
  onToggleCrop,
  onTogglePlayback,
  onToggleRangeTool,
}: {
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
  onResizeCanvas?: () => void;
  onSelectTool?: () => void;
  /** Moves the playhead by one arrow press; `coarse` is the Shift jump. */
  onStep?: (direction: -1 | 1, coarse: boolean) => void;
  onToggleBladeTool?: () => void;
  onToggleCrop?: () => void;
  onTogglePlayback?: () => void;
  onToggleRangeTool?: () => void;
}) {
  const focusIntentRef = useRef<"keyboard" | "pointer">("keyboard");

  useEffect(() => {
    const onPointerDown = () => {
      focusIntentRef.current = "pointer";
    };
    const onKeyDown = (event: KeyboardEvent) => {
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
        event.preventDefault();
        // The timeline ruler keeps its own arrow handler for when it has focus;
        // stop the event here so a bubbling copy cannot seek a second time.
        event.stopPropagation();
        handled();
        return;
      }

      if (event.repeat || event.isComposing || event.altKey) return;

      if (
        event.code === "Escape" &&
        onDeselect &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onDeselect();
        return;
      }

      const commandKey = event.ctrlKey || event.metaKey;
      if (commandKey && !event.shiftKey) {
        if (event.code === "KeyB" && onCutTimeline) {
          if (ownsTextEditingKeys(event.target)) return;
          event.preventDefault();
          onCutTimeline();
        } else if (event.code === "KeyC" && onCopy) {
          if (ownsTextEditingKeys(event.target)) return;
          event.preventDefault();
          onCopy();
        } else if (event.code === "KeyE" && onExport) {
          event.preventDefault();
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
        event.preventDefault();
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
        event.preventDefault();
        event.stopPropagation();
        onTogglePlayback();
        return;
      }

      if (
        (event.code === "Backspace" || event.code === "Delete") &&
        onDelete &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onDelete();
        return;
      }

      if (
        event.code === "BracketLeft" &&
        onMoveForward &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onMoveForward();
        return;
      }

      if (
        event.code === "BracketRight" &&
        onMoveBackward &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onMoveBackward();
        return;
      }

      if (
        event.code === "KeyB" &&
        onToggleBladeTool &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onToggleBladeTool();
      } else if (
        event.code === "KeyP" &&
        onTogglePlayback &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onTogglePlayback();
      } else if (
        event.code === "KeyR" &&
        onRecenter &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onRecenter();
      } else if (
        event.code === "KeyF" &&
        onResizeCanvas &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onResizeCanvas();
      } else if (
        event.code === "KeyC" &&
        onToggleCrop &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onToggleCrop();
      } else if (
        event.code === "KeyV" &&
        onSelectTool &&
        !ownsTextEditingKeys(event.target)
      ) {
        event.preventDefault();
        onSelectTool();
      }
    };

    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [
    onCopy,
    onCutTimeline,
    onDelete,
    onDeselect,
    onExport,
    onMoveBackward,
    onMoveForward,
    onNudge,
    onRecenter,
    onResizeCanvas,
    onSelectTool,
    onStep,
    onToggleCrop,
    onToggleBladeTool,
    onTogglePlayback,
    onToggleRangeTool,
  ]);
}
