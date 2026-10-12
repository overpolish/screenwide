// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { EditorKind } from "../../types";

/**
 * What the Transcript panel's asks and the playhead reach in the preview,
 * which is the only place that owns them. Where the playhead stands is the
 * preview's own business, while the panel's asks arrive through the editor's
 * bridge; the preview leaves these here for the editor to find, the way the
 * scene under the playhead is published.
 */
export type RecordingTranscriptPreviewControls = {
  /** Show this stretch of the source, in milliseconds, on the timeline as
   * the words chosen in the panel, or show none with null. */
  highlight: (range: { endMs: number; startMs: number } | null) => void;
  /** Seek the preview to this source position, in milliseconds. */
  seek: (sourceMs: number) => void;
  /** Follow the playhead, in source milliseconds, until unsubscribed. */
  subscribe: (listener: (sourceMs: number) => void) => () => void;
};

const workspaces = new Map<EditorKind, RecordingTranscriptPreviewControls>();
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

/** Publish this workspace's preview controls for as long as its preview is
 * mounted. The controls are read through the ref when called, so whoever
 * holds them still reaches the preview's current edit and playhead. */
export function usePublishRecordingTranscriptPreview(
  workspace: EditorKind,
  controls: RecordingTranscriptPreviewControls,
) {
  const controlsRef = useRef(controls);
  controlsRef.current = controls;
  useEffect(() => {
    const published: RecordingTranscriptPreviewControls = {
      highlight: (range) => {
        controlsRef.current.highlight(range);
      },
      seek: (sourceMs) => {
        controlsRef.current.seek(sourceMs);
      },
      subscribe: (listener) => controlsRef.current.subscribe(listener),
    };
    workspaces.set(workspace, published);
    notify();
    return () => {
      if (workspaces.get(workspace) !== published) return;
      workspaces.delete(workspace);
      notify();
    };
  }, [workspace]);
}

/** What the Transcript panel's asks reach in `workspace`'s preview, while it
 * is mounted. Re-renders as the preview publishes and retires. */
export const useRecordingTranscriptPreviewControls = (workspace: EditorKind) =>
  useSyncExternalStore(subscribe, () => workspaces.get(workspace) ?? null);
