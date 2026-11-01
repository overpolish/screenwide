// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { SourceRect } from "./screenshot/screenshot-geometry";
import { EditorKind } from "./types";
import {
  EditorEditGesture,
  endGestureAfter,
  useEditorEditGesture,
} from "./use-editor-edit-history";

/**
 * The workspace's own padding calls, reachable from outside the preview that
 * owns them.
 *
 * The analysis lives with the preview: it knows which layer is in hand, where
 * the playhead is, and which request is still the current one. The Screen
 * island's inset presets and Recenter arrive by another road entirely - the
 * island window, through the editor's bridge - and have to reach the same
 * hook, so each workspace leaves its calls here for the island to find.
 */
export type RecenterInsetControls = {
  /** Detect the content and pull it to the middle of its padded frame. */
  begin: () => void;
  /** Read the colour behind the layer, so a pad drawn now has one to use;
   * settles once the colour is in, or at once when there is none to read. */
  prepare: () => Promise<void>;
  /** Re-read the inset colour behind a crop that was just committed; settles
   * once the colour is in, so the crop's undo step can wait for it. */
  refresh: (crop: SourceRect) => Promise<void>;
};

type PublishedControls = RecenterInsetControls & {
  gesture: EditorEditGesture;
};

const workspaces = new Map<EditorKind, PublishedControls>();

/** Publish this workspace's padding calls for as long as its preview is
 * mounted. */
export function useRecenterInsetControls(
  workspace: EditorKind,
  controls: RecenterInsetControls,
) {
  const gesture = useEditorEditGesture();
  const controlsRef = useRef({ controls, gesture });
  controlsRef.current = { controls, gesture };
  useEffect(() => {
    const published: PublishedControls = {
      begin: () => {
        controlsRef.current.controls.begin();
      },
      gesture: {
        beginGesture: () => {
          controlsRef.current.gesture.beginGesture();
        },
        endGesture: () => {
          controlsRef.current.gesture.endGesture();
        },
      },
      prepare: () => controlsRef.current.controls.prepare(),
      refresh: (crop) => controlsRef.current.controls.refresh(crop),
    };
    workspaces.set(workspace, published);
    return () => {
      if (workspaces.get(workspace) === published) workspaces.delete(workspace);
    };
  }, [workspace]);
}

/**
 * Makes `edit`, then reads the colour its padding is filled with, as one undo
 * step. The colour is read off the picture and lands a moment later in a
 * change of its own, which would otherwise be a step of its own: undoing the
 * edit would seem to do nothing, the colour going back first.
 */
const editThenColor = (
  workspace: EditorKind,
  edit: () => void,
  color: (controls: RecenterInsetControls) => Promise<void>,
) => {
  const published = workspaces.get(workspace);
  if (!published) {
    edit();
    return;
  }
  published.gesture.beginGesture();
  edit();
  endGestureAfter(published.gesture, color(published));
};

/** Make `edit`, which pads the layer, with the colour the pad is filled
 * with read if none was yet. */
export const padWithInsetColor = (workspace: EditorKind, edit: () => void) => {
  editThenColor(workspace, edit, (controls) => controls.prepare());
};

/** Make `edit`, which crops the layer to `crop`, with the inset colour
 * behind the new crop read again. */
export const cropWithInsetColor = (
  workspace: EditorKind,
  crop: SourceRect,
  edit: () => void,
) => {
  editThenColor(workspace, edit, (controls) => controls.refresh(crop));
};

/** Put the layer's content in the middle of its padded frame. */
export const recenterWorkspaceContent = (workspace: EditorKind) => {
  workspaces.get(workspace)?.begin();
};
