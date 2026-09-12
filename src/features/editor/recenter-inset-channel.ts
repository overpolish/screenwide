// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { SourceRect } from "./screenshot-geometry";
import { EditorKind } from "./types";

/**
 * The workspace's own "the crop moved, look at the inset again" call, reachable
 * from outside the preview that owns it.
 *
 * A crop drag ends inside the preview and can call the recenter hook directly.
 * A crop committed from the tool panel arrives by another road entirely - the
 * panel window, through the editor's bridge - and has to reach the same hook,
 * so each workspace leaves its refresh here for the panel's commit to find.
 */
type RecenterInsetRefresh = (crop: SourceRect) => void;

const refreshers = new Map<EditorKind, RecenterInsetRefresh>();

/** Publish this workspace's refresh for as long as its preview is mounted. */
export function useRecenterInsetRefresh(
  workspace: EditorKind,
  refresh: RecenterInsetRefresh,
) {
  const refreshRef = useRef(refresh);
  refreshRef.current = refresh;
  useEffect(() => {
    const call: RecenterInsetRefresh = (crop) => {
      refreshRef.current(crop);
    };
    refreshers.set(workspace, call);
    return () => {
      if (refreshers.get(workspace) === call) refreshers.delete(workspace);
    };
  }, [workspace]);
}

/** Re-read the inset colour behind a crop that was just committed. A no-op
 * when the workspace has no preview mounted to ask. */
export const refreshRecenterInset = (
  workspace: EditorKind,
  crop: SourceRect,
) => {
  refreshers.get(workspace)?.(crop);
};
