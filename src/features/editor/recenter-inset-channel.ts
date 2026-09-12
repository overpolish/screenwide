// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { SourceRect } from "./screenshot-geometry";
import { EditorKind } from "./types";

/**
 * The workspace's own padding calls, reachable from outside the preview that
 * owns them.
 *
 * The analysis lives with the preview: it knows which layer is in hand, where
 * the playhead is, and which request is still the current one. The Select
 * panel's padding controls arrive by another road entirely - the panel window,
 * through the editor's bridge - and have to reach the same hook, so each
 * workspace leaves its calls here for the panel to find.
 */
export type RecenterInsetControls = {
  /** Detect the content and pull it to the middle of its padded frame. */
  begin: () => void;
  /** Read the colour behind the layer, so a pad drawn now has one to use. */
  prepare: () => void;
  /** Re-read the inset colour behind a crop that was just committed. */
  refresh: (crop: SourceRect) => void;
};

const workspaces = new Map<EditorKind, RecenterInsetControls>();

/** Publish this workspace's padding calls for as long as its preview is
 * mounted. */
export function useRecenterInsetControls(
  workspace: EditorKind,
  controls: RecenterInsetControls,
) {
  const controlsRef = useRef(controls);
  controlsRef.current = controls;
  useEffect(() => {
    const published: RecenterInsetControls = {
      begin: () => {
        controlsRef.current.begin();
      },
      prepare: () => {
        controlsRef.current.prepare();
      },
      refresh: (crop) => {
        controlsRef.current.refresh(crop);
      },
    };
    workspaces.set(workspace, published);
    return () => {
      if (workspaces.get(workspace) === published) workspaces.delete(workspace);
    };
  }, [workspace]);
}

/** Re-read the inset colour behind a crop that was just committed. A no-op
 * when the workspace has no preview mounted to ask. */
export const refreshRecenterInset = (
  workspace: EditorKind,
  crop: SourceRect,
) => {
  workspaces.get(workspace)?.refresh(crop);
};

/** Detect the colour a pad about to be drawn should be filled with. */
export const prepareRecenterInset = (workspace: EditorKind) => {
  workspaces.get(workspace)?.prepare();
};

/** Put the layer's content in the middle of its padded frame. */
export const recenterWorkspaceContent = (workspace: EditorKind) => {
  workspaces.get(workspace)?.begin();
};
