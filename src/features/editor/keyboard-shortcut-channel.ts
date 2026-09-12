// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { ToolPanelShortcutSelection } from "./tool-panels/tool-panel-store";
import { EditorKind } from "./types";

/**
 * The shortcut the preview has in hand, and the edits that move it, reachable
 * from outside the preview that owns them.
 *
 * The shortcut on screen is the preview's own business: it knows where the
 * playhead is, which fragment is drawn there and which of them are selected.
 * The Selection panel's shortcut rows arrive by another road entirely - the
 * panel window, through the editor's bridge - and have to reach the same edit
 * functions the drag on the picture uses, so each workspace leaves what it has
 * in hand here for the panel to find.
 */
export type KeyboardShortcutPlacement = {
  positionXPercent?: number;
  positionYPercent?: number;
  sizePercent?: number;
};

export type KeyboardShortcutControls = {
  /** Draw the selected shortcut at this size and centre, all in percent. */
  applyPlacement: (placement: KeyboardShortcutPlacement) => void;
  /** Give every shortcut this one's size and position. */
  applyToAll: () => void;
  /** Put the selected shortcut back where the recording drew it. */
  reset: () => void;
};

type PublishedShortcut = {
  controls: KeyboardShortcutControls;
  selection: ToolPanelShortcutSelection | null;
};

const workspaces = new Map<EditorKind, PublishedShortcut>();
const listeners = new Set<() => void>();

const notify = () => {
  for (const listener of listeners) listener();
};

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/**
 * Publish the shortcut this workspace has in hand for as long as its preview
 * is mounted.
 *
 * The selection is compared by value: the preview rebuilds it every frame, and
 * a panel that re-rendered on identity alone would never settle.
 */
export function usePublishKeyboardShortcut(
  workspace: EditorKind,
  selection: ToolPanelShortcutSelection | null,
  controls: KeyboardShortcutControls,
) {
  const controlsRef = useRef(controls);
  controlsRef.current = controls;
  const serialized = selection === null ? null : JSON.stringify(selection);
  useEffect(() => {
    const published: PublishedShortcut = {
      controls: {
        applyPlacement: (placement) => {
          controlsRef.current.applyPlacement(placement);
        },
        applyToAll: () => {
          controlsRef.current.applyToAll();
        },
        reset: () => {
          controlsRef.current.reset();
        },
      },
      selection:
        serialized === null
          ? null
          : (JSON.parse(serialized) as ToolPanelShortcutSelection),
    };
    workspaces.set(workspace, published);
    notify();
    return () => {
      if (workspaces.get(workspace) !== published) return;
      workspaces.delete(workspace);
      notify();
    };
  }, [serialized, workspace]);
}

/** The shortcut this workspace has in hand, or nothing. */
export const useKeyboardShortcutSelection = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    () => workspaces.get(workspace)?.selection ?? null,
  );

/** Draw the selected shortcut at this size and centre. A no-op when the
 * workspace has no preview mounted to ask. */
export const placeKeyboardShortcut = (
  workspace: EditorKind,
  placement: KeyboardShortcutPlacement,
) => {
  workspaces.get(workspace)?.controls.applyPlacement(placement);
};

/** Put the selected shortcut back where the recording drew it. */
export const resetKeyboardShortcut = (workspace: EditorKind) => {
  workspaces.get(workspace)?.controls.reset();
};

/** Give every shortcut the selected one's size and position. */
export const applyKeyboardShortcutToAll = (workspace: EditorKind) => {
  workspaces.get(workspace)?.controls.applyToAll();
};
