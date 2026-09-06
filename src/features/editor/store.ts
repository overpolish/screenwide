// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import {
  EditorKind,
  EditorSnapshot,
  EditorSnapshots,
  initialEditorSnapshot,
} from "./types";

type EditorStore = {
  hydrated: boolean;
  setSnapshot: (snapshot: EditorSnapshot) => void;
  setSnapshots: (snapshots: EditorSnapshots) => void;
  snapshots: EditorSnapshots;
};

/**
 * Owned by Rust and broadcast, like the recording snapshot. Not persisted.
 *
 * Every workspace is held here rather than only this window's: the recording
 * bar reads both, and an editor window would otherwise have to filter the
 * broadcast before it could trust what it stored.
 */
export const useEditorStore = create<EditorStore>()((set) => ({
  hydrated: false,
  setSnapshot: (snapshot) => {
    set((state) => ({
      hydrated: true,
      snapshots: { ...state.snapshots, [snapshot.workspace]: snapshot },
    }));
  },
  setSnapshots: (snapshots) => {
    set({ hydrated: true, snapshots });
  },
  snapshots: {
    recording: initialEditorSnapshot("recording"),
    screenshot: initialEditorSnapshot("screenshot"),
  },
}));

export const selectSnapshot = (kind: EditorKind) => (state: EditorStore) =>
  state.snapshots[kind];

export const selectArtifact = (kind: EditorKind) => (state: EditorStore) =>
  state.snapshots[kind].artifact;

export const selectDirectory = (kind: EditorKind) => (state: EditorStore) =>
  state.snapshots[kind].directory;

/**
 * What the recording bar needs. Two booleans rather than one object: a
 * selector that built an object would hand back a fresh identity on every
 * store read and re-render the bar continuously.
 */
export const selectHasPendingRecording = (state: EditorStore) =>
  state.snapshots.recording.artifact !== null;

export const selectHasPendingScreenshot = (state: EditorStore) =>
  state.snapshots.screenshot.artifact !== null;
