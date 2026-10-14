// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import { EditorKind } from "./types";

const WORKSPACES: EditorKind[] = ["recording", "screenshot"];

/** The build that wrote a snapshot. Every window loads at launch, before
 * its editor has published anything, and would otherwise take up what an
 * earlier run left in storage: a snapshot whose shape an older build set,
 * which this one's panels can fail to draw. */
const BUILD = import.meta.env.SCREENWIDE_BUILD ?? "";

/** A snapshot as stored, with the build that wrote it. */
type Stored<T> = { build: string; snapshot: T };

type WorkspaceMirror<T> = {
  publish: (workspace: EditorKind, snapshot: T) => void;
  snapshots: Partial<Record<EditorKind, T>>;
};

/** Settings groups travel as objects, and a copy read back from storage is
 * never the object that was published, so values are compared by content. */
const isSameValue = (a: unknown, b: unknown) =>
  a === b ||
  (typeof a === "object" &&
    a !== null &&
    typeof b === "object" &&
    b !== null &&
    JSON.stringify(a) === JSON.stringify(b));

const isSameSnapshot = <T extends object>(a: T | undefined, b: T) =>
  a !== undefined &&
  (Object.keys(b) as (keyof T)[]).every((key) => isSameValue(a[key], b[key]));

/**
 * Settings an editor window owns, mirrored through `localStorage` to the
 * windows that show them.
 *
 * Each workspace has a key of its own because each has its own editor window
 * writing it. With one shared record, an editor publishing its own change also
 * wrote back its possibly stale copy of the other workspace's settings, and
 * the last writer won: a running save could briefly read as idle again.
 */
export function createWorkspaceMirror<T extends object>(name: string) {
  const keyOf = (workspace: EditorKind) => `${name}-${workspace}`;
  /** The snapshot `raw` holds, where this build wrote it. */
  const parse = (raw: string | null): T | undefined => {
    if (raw === null) return undefined;
    try {
      const stored = JSON.parse(raw) as Partial<Stored<T>> | null;
      return stored?.build === BUILD ? stored.snapshot : undefined;
    } catch {
      return undefined;
    }
  };
  const readAll = () => {
    const snapshots: Partial<Record<EditorKind, T>> = {};
    for (const workspace of WORKSPACES) {
      const snapshot = parse(localStorage.getItem(keyOf(workspace)));
      if (snapshot !== undefined) snapshots[workspace] = snapshot;
    }
    return snapshots;
  };

  const useMirror = create<WorkspaceMirror<T>>()((set, get) => ({
    publish: (workspace, snapshot) => {
      if (isSameSnapshot(get().snapshots[workspace], snapshot)) return;
      const stored: Stored<T> = { build: BUILD, snapshot };
      localStorage.setItem(keyOf(workspace), JSON.stringify(stored));
      set((state) => ({
        snapshots: { ...state.snapshots, [workspace]: snapshot },
      }));
    },
    snapshots: typeof localStorage === "undefined" ? {} : readAll(),
  }));

  /** Adopts another window's publish; other keys are ignored. */
  const synchronize = (event: StorageEvent) => {
    const workspace = WORKSPACES.find((kind) => keyOf(kind) === event.key);
    if (workspace === undefined) return;
    const snapshot = parse(event.newValue);
    useMirror.setState((state) => ({
      snapshots: { ...state.snapshots, [workspace]: snapshot },
    }));
  };

  return { synchronize, useMirror };
}
