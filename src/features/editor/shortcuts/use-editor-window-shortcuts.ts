// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { isWindowsHost } from "../../../lib/platform";

import {
  arrowDirections,
  editorCommandKeyAction,
  EditorCommandKeys,
  editorToolKeyAction,
  EditorToolKeys,
} from "./editor-tool-keys";
import { preserveEscapeFocus } from "./escape-focus";
import { exportIslandOwnsKey } from "./export-island-keys";
import { keepPointerShown } from "./keep-pointer-shown";
import {
  editsText,
  ownsActivationKeys,
  arrowsOnlyMoveFocus,
  ownsArrowKeys,
  ownsPopupInteractionKeys,
  ownsTextEditingKeys,
} from "./keyboard-target";
import {
  useWindowKeyListeners,
  WindowKeyHandlers,
} from "./use-window-key-listeners";

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

export function useEditorWindowShortcuts({
  onConfirm,
  onCopy,
  onCutTimeline,
  onDelete,
  onDeselect,
  onDeselectAll,
  onExport,
  onNudge,
  onPaste,
  onRedo,
  onSeekEdge,
  onStep,
  onUndo,
  ownsEscape = false,
  ...toolKeys
}: EditorToolKeys &
  EditorCommandKeys & {
    /** Enter: done with whatever this window is in the middle of. */
    onConfirm?: () => void;
    onDelete?: () => void;
    onDeselect?: () => void;
    /** Moves the selected layer by one arrow press; `coarse` is the Shift jump. */
    onNudge?: (directionX: number, directionY: number, coarse: boolean) => void;
    onRedo?: () => void;
    /** Home and End: the playhead to the recording's start or end. */
    onSeekEdge?: (edge: "end" | "start") => void;
    /** Moves the playhead by one arrow press; `coarse` is the Shift jump. */
    onStep?: (direction: -1 | 1, coarse: boolean) => void;
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
  const handlersRef = useRef<WindowKeyHandlers | null>(null);
  useEffect(preserveEscapeFocus, []);

  useEffect(() => {
    const consumedKeys = consumedKeysRef.current;
    const consume = (event: KeyboardEvent) => {
      event.preventDefault();
      // Stop before document-level navigation and React Aria modality tracking.
      // Other editor handlers on this window still receive the event.
      event.stopPropagation();
      consumedKeys.add(event.code);
      keepPointerShown();
    };
    const onKeyUp = (event: KeyboardEvent) => {
      if (consumedKeys.delete(event.code)) event.stopPropagation();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (exportIslandOwnsKey(event, consume)) return;
      if (
        event.repeat &&
        consumedKeys.has(event.code) &&
        !arrowDirections.has(event.code)
      ) {
        consume(event);
        return;
      }
      // Only Tab says the keyboard is being used to move around the window;
      // a pointer press says it is not, until the next Tab.
      if (event.code === "Tab") focusIntentRef.current = "keyboard";

      // Arrows run before the shared guards: holding one has to repeat, and
      // Shift only picks the bigger jump rather than naming another shortcut.
      const arrow = arrowDirections.get(event.code);
      if (arrow) {
        const pickedWithPointer =
          focusIntentRef.current === "pointer" &&
          arrowsOnlyMoveFocus(event.target);
        if (
          event.isComposing ||
          event.altKey ||
          event.ctrlKey ||
          event.metaKey ||
          ownsTextEditingKeys(event.target) ||
          (ownsArrowKeys(event.target) && !pickedWithPointer)
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

      if (event.code === "Escape" && !editsText(event.target)) {
        // Only text being typed into keeps Escape (chosen words leave it to
        // put the mode away). A claimed Escape is the only one that runs: its
        // owner takes it, and the plain deselects on the window let it through.
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
      // Windows apps answer Ctrl+Y for redo as well as Ctrl+Shift+Z; on a Mac
      // ⌘Y is not redo.
      const isRedoY =
        isWindowsHost &&
        event.ctrlKey &&
        !event.shiftKey &&
        event.code === "KeyY";
      if (
        commandKey &&
        (event.code === "KeyZ" || isRedoY) &&
        !ownsTextEditingKeys(event.target)
      ) {
        const action = event.shiftKey || isRedoY ? onRedo : onUndo;
        if (action) {
          consume(event);
          action();
        }
        return;
      }
      if (commandKey && !event.shiftKey) {
        const action = editorCommandKeyAction(
          event.code,
          { onCopy, onCutTimeline, onDeselectAll, onExport, onPaste },
          ownsTextEditingKeys(event.target),
        );
        if (action) {
          consume(event);
          action();
        }
        return;
      }

      if (event.shiftKey && !commandKey) {
        const shiftedKey = editorToolKeyAction(event.code, toolKeys, true);
        if (shiftedKey && !ownsTextEditingKeys(event.target)) {
          consume(event);
          shiftedKey();
        }
        return;
      }

      if (event.ctrlKey || event.metaKey || event.shiftKey) return;

      const onTogglePlayback = toolKeys.onTogglePlayback;
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
        (event.code === "Home" || event.code === "End") &&
        onSeekEdge &&
        !ownsTextEditingKeys(event.target) &&
        !ownsArrowKeys(event.target)
      ) {
        consume(event);
        onSeekEdge(event.code === "Home" ? "start" : "end");
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

      const toolKey = editorToolKeyAction(event.code, toolKeys);
      if (toolKey && !ownsTextEditingKeys(event.target)) {
        consume(event);
        toolKey();
      }
    };

    handlersRef.current = { onKeyDown, onKeyUp };
    return () => {
      handlersRef.current = null;
    };
  }, [
    onConfirm,
    onCopy,
    onCutTimeline,
    onDelete,
    onDeselect,
    onDeselectAll,
    onExport,
    onNudge,
    onPaste,
    onRedo,
    onSeekEdge,
    onStep,
    onUndo,
    ownsEscape,
    // A new object each render: the handlers are remade with it, which only
    // swaps the ref the window's one listener reads.
    toolKeys,
  ]);
  useWindowKeyListeners(handlersRef, focusIntentRef);
}
