// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { type ToolPanelStripItem } from "../../tools/tool-panel-strip";
import { type EditorKind } from "../../types";

/**
 * What is chosen on the recording's strip, and what its island can do to
 * it, reachable from outside the preview that owns the strip. The island
 * arrives by another road, the panel window through the editor's bridge,
 * so the preview leaves its choice here for the bridge to find, as it does
 * the scene under the playhead (`recording-scene-channel.ts`). Shortcuts
 * chosen in their lane have no island of their own: the Keyboard island
 * says how many and deletes them.
 */
export type StripIslandControls = {
  /** Cuts what is chosen, and its time with it. */
  cut: () => void;
  /** Deletes the shortcuts chosen in their lane. */
  deleteShortcuts: () => void;
  /** Squeezes the chosen lull with the rest, or keeps it as recorded. */
  playLull: (play: "keep" | "squeeze") => void;
  /** Trims the chosen lull to a breath. */
  trimLullToBreath: () => void;
};

/** What is chosen on the strip, as its islands show it. */
export type StripIslandChoice = {
  /** The one item, or the gathered beats and lulls, with an island. */
  item: ToolPanelStripItem | null;
  /** How many shortcuts are chosen in their lane. */
  shortcuts: number;
};

type PublishedStripItem = StripIslandChoice & {
  controls: StripIslandControls;
};

const NOTHING_CHOSEN: StripIslandChoice = { item: null, shortcuts: 0 };

const workspaces = new Map<EditorKind, PublishedStripItem>();
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

/** Publish what is chosen on this workspace's strip for as long as its
 * preview is mounted. `chosen` is compared by value: the preview rebuilds
 * it every render, and an island re-rendered on identity alone would never
 * settle. */
export function usePublishStripIsland(
  workspace: EditorKind,
  chosen: StripIslandChoice,
  controls: StripIslandControls,
) {
  const controlsRef = useRef(controls);
  controlsRef.current = controls;
  const serialized = JSON.stringify(chosen);
  useEffect(() => {
    const published: PublishedStripItem = {
      ...(JSON.parse(serialized) as StripIslandChoice),
      // Read through the ref when called, so the bridge reaches the
      // preview's current edit.
      controls: {
        cut: () => {
          controlsRef.current.cut();
        },
        deleteShortcuts: () => {
          controlsRef.current.deleteShortcuts();
        },
        playLull: (play) => {
          controlsRef.current.playLull(play);
        },
        trimLullToBreath: () => {
          controlsRef.current.trimLullToBreath();
        },
      },
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

/** What is chosen on `workspace`'s strip, kept as one object until it
 * changes, so a store reading it settles. */
export const useStripIslandChoice = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    (): StripIslandChoice => workspaces.get(workspace) ?? NOTHING_CHOSEN,
  );

/** What the strip islands' asks reach in `workspace`'s preview, while it
 * is mounted. */
export const stripIslandControls = (workspace: EditorKind) =>
  workspaces.get(workspace)?.controls;
