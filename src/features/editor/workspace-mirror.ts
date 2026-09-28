// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import { EditorKind } from "./types";

const WORKSPACES: EditorKind[] = ["recording", "screenshot"];

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
  const parse = (raw: string | null): T | undefined => {
    if (raw === null) return undefined;
    try {
      return JSON.parse(raw) as T;
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
      localStorage.setItem(keyOf(workspace), JSON.stringify(snapshot));
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
