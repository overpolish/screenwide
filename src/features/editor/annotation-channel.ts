// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import {
  rememberAnnotationAngle,
  rememberAnnotationAnimated,
  rememberAnnotationStyle,
} from "./annotation-defaults";
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
/** The dress the tool in hand draws its next annotation in, for the panel to
 * show and edit while nothing is chosen. */
const drafts = new Map<EditorKind, ToolPanelAnnotation>();
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

/**
 * Publish the dress the tool in hand draws its next annotation in, or `null`
 * while no drawing tool is in hand. The panel shows it whenever nothing is
 * chosen, so a dress is picked before drawing rather than fixed after.
 */
export function usePublishAnnotationDraft(
  workspace: EditorKind,
  draft: ToolPanelAnnotation | null,
) {
  const serialized = draft === null ? null : JSON.stringify(draft);
  useEffect(() => {
    if (serialized === null) {
      if (drafts.delete(workspace)) notify();
      return;
    }
    const published = JSON.parse(serialized) as ToolPanelAnnotation;
    drafts.set(workspace, published);
    notify();
    return () => {
      if (drafts.get(workspace) !== published) return;
      drafts.delete(workspace);
      notify();
    };
  }, [serialized, workspace]);
}

/** The annotation this workspace has in hand, or else the dress its tool
 * draws in, or nothing. */
export const useAnnotationSelection = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    () => workspaces.get(workspace)?.selection ?? drafts.get(workspace) ?? null,
  );

/** The workspace's published edits when it has an annotation chosen. */
const chosen = (workspace: EditorKind) => {
  const published = workspaces.get(workspace);
  return published?.selection ? published : null;
};

/** Dress the chosen annotation, or with none chosen, the next one the tool in
 * hand draws. A no-op when the workspace has neither to dress. */
export const applyAnnotationStyle = (
  workspace: EditorKind,
  style: Partial<AnnotationStyle>,
) => {
  const published = chosen(workspace);
  if (published) {
    published.applyStyle(style);
    return;
  }
  const draft = drafts.get(workspace);
  if (draft) rememberAnnotationStyle({ ...draft.style, ...style }, draft.kind);
};

/** Draw the chosen annotation in and out over its clip, or leave it standing;
 * with none chosen, the next one. */
export const applyAnnotationAnimated = (
  workspace: EditorKind,
  animated: boolean,
) => {
  const published = chosen(workspace);
  if (published) published.applyAnimated(animated);
  else if (drafts.has(workspace)) rememberAnnotationAnimated(animated);
};

/** Turn the chosen counter's tail to `angle`, in radians clockwise from
 * east; with none chosen, the next counter's. */
export const applyAnnotationAngle = (workspace: EditorKind, angle: number) => {
  const published = chosen(workspace);
  if (published) published.applyAngle(angle);
  else if (drafts.has(workspace)) rememberAnnotationAngle(angle);
};

/** Turn the chosen annotation round. A no-op with none chosen: there is
 * nothing drawn yet to turn. */
export const applyAnnotationReverse = (workspace: EditorKind) => {
  chosen(workspace)?.applyReverse();
};

/** Lay the chosen redaction's blocks out again, or draw the chosen hand-drawn
 * highlight's stroke again, from a fresh seed. A no-op with none chosen: every
 * fresh one is drawn from a seed of its own. */
export const applyAnnotationShuffle = (workspace: EditorKind) => {
  chosen(workspace)?.applyShuffle();
};
