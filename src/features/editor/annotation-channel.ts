// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { AnnotationStyle } from "./annotations";
import { ToolPanelAnnotation } from "./tool-panels/tool-panel-selection";
import { EditorKind } from "./types";

/**
 * The mark the preview has in hand, and the edits that dress it, reachable
 * from outside the preview that owns them.
 *
 * Which arrow is chosen is the preview's own business: the native tool
 * hit-tests it and reports it back, and the tool that did so keeps it. The
 * Arrow panel arrives by another road entirely - the panel window, through
 * the editor's bridge - and has to reach the same commit path the drag on the
 * picture uses, so each workspace leaves what it has in hand here for the
 * panel to find. This is the twin of `keyboard-shortcut-channel.ts`.
 */
type PublishedAnnotation = {
  applyAnimated: (animated: boolean) => void;
  applyReverse: () => void;
  applyStyle: (style: Partial<AnnotationStyle>) => void;
  selection: ToolPanelAnnotation | null;
};

const workspaces = new Map<EditorKind, PublishedAnnotation>();
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
 * Publish the mark this workspace has in hand for as long as its preview is
 * mounted.
 *
 * The selection is compared by value: the preview rebuilds it on every render,
 * and a panel that re-rendered on identity alone would never settle.
 */
export function usePublishAnnotationSelection(
  workspace: EditorKind,
  selection: ToolPanelAnnotation | null,
  apply: Omit<PublishedAnnotation, "selection">,
) {
  const applyRef = useRef(apply);
  applyRef.current = apply;
  const serialized = selection === null ? null : JSON.stringify(selection);
  useEffect(() => {
    const published: PublishedAnnotation = {
      applyAnimated: (animated) => {
        applyRef.current.applyAnimated(animated);
      },
      applyReverse: () => {
        applyRef.current.applyReverse();
      },
      applyStyle: (style) => {
        applyRef.current.applyStyle(style);
      },
      selection:
        serialized === null
          ? null
          : (JSON.parse(serialized) as ToolPanelAnnotation),
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

/** The mark this workspace has in hand, or nothing. */
export const useAnnotationSelection = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    () => workspaces.get(workspace)?.selection ?? null,
  );

/** Dress the chosen mark. A no-op when the workspace has no preview mounted
 * to ask. */
export const applyAnnotationStyle = (
  workspace: EditorKind,
  style: Partial<AnnotationStyle>,
) => {
  workspaces.get(workspace)?.applyStyle(style);
};

/** Draw the chosen mark in and out over its clip, or leave it standing. A
 * no-op when the workspace has no preview mounted to ask. */
export const applyAnnotationAnimated = (
  workspace: EditorKind,
  animated: boolean,
) => {
  workspaces.get(workspace)?.applyAnimated(animated);
};

/** Turn the chosen mark round. A no-op when the workspace has no preview
 * mounted to ask. */
export const applyAnnotationReverse = (workspace: EditorKind) => {
  workspaces.get(workspace)?.applyReverse();
};
