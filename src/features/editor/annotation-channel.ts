// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { AnnotationStyle } from "./annotations";
import { ToolPanelAnnotation } from "./tool-panels/tool-panel-selection";
import { EditorKind } from "./types";

/**
 * The annotation the preview has in hand, and the edits that dress it,
 * reachable from outside the preview that owns them.
 *
 * Which annotation is chosen is the preview's own business: the native tool
 * hit-tests it and reports it back, and the tool that did so keeps it. The
 * annotation panel arrives by another road entirely - the panel window, through
 * the editor's bridge - and has to reach the same commit path the drag on the
 * picture uses, so each workspace leaves what it has in hand here for the panel
 * to find. This is the twin of `keyboard-shortcut-channel.ts`.
 */
type PublishedAnnotation = {
  applyAngle: (angle: number) => void;
  applyAnimated: (animated: boolean) => void;
  applyReverse: () => void;
  applyShuffle: () => void;
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
 * Publish the annotation this workspace has in hand for as long as its preview
 * is mounted.
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
      applyAngle: (angle) => {
        applyRef.current.applyAngle(angle);
      },
      applyAnimated: (animated) => {
        applyRef.current.applyAnimated(animated);
      },
      applyReverse: () => {
        applyRef.current.applyReverse();
      },
      applyShuffle: () => {
        applyRef.current.applyShuffle();
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

/** The annotation this workspace has in hand, or nothing. */
export const useAnnotationSelection = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    () => workspaces.get(workspace)?.selection ?? null,
  );

/** Dress the chosen annotation. A no-op when the workspace has no preview
 * mounted to ask. */
export const applyAnnotationStyle = (
  workspace: EditorKind,
  style: Partial<AnnotationStyle>,
) => {
  workspaces.get(workspace)?.applyStyle(style);
};

/** Draw the chosen annotation in and out over its clip, or leave it standing. A
 * no-op when the workspace has no preview mounted to ask. */
export const applyAnnotationAnimated = (
  workspace: EditorKind,
  animated: boolean,
) => {
  workspaces.get(workspace)?.applyAnimated(animated);
};

/** Turn the chosen counter's tail to `angle`, in radians clockwise from
 * east. A no-op when the workspace has no preview mounted to ask. */
export const applyAnnotationAngle = (workspace: EditorKind, angle: number) => {
  workspaces.get(workspace)?.applyAngle(angle);
};

/** Turn the chosen annotation round. A no-op when the workspace has no preview
 * mounted to ask. */
export const applyAnnotationReverse = (workspace: EditorKind) => {
  workspaces.get(workspace)?.applyReverse();
};

/** Lay the chosen redaction's blocks out again from a fresh seed. A no-op when
 * the workspace has no preview mounted to ask. */
export const applyAnnotationShuffle = (workspace: EditorKind) => {
  workspaces.get(workspace)?.applyShuffle();
};
