// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { SceneRotation } from "../layer-rotation";
import { ToolPanelAnnotation } from "../tool-panels/tool-panel-selection";
import { EditorKind } from "../types";

import { AnnotationStyle, ImageArt, ImagePlay, TextSpan } from "./annotations";

import type { TextStyle } from "../../../bindings/TextStyle";

/** What the panel changes about how a moving image plays. */
export type ImagePlayChange = Partial<Pick<ImagePlay, "frame" | "once">>;

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
  applyClearDrawings: () => void;
  /** Delete every chosen annotation, in one edit. */
  applyDelete: () => void;
  /** Set apart the chosen text's words `emphasis`, and no others. */
  applyEmphasis: (emphasis: TextSpan[]) => void;
  /** Give the chosen image another picture. */
  applyImage: (art: ImageArt) => void;
  /** Change how the chosen moving image plays. */
  applyImagePlay: (play: ImagePlayChange) => void;
  applyLook: (look: TextStyle) => void;
  applyReverse: () => void;
  /** Turn the chosen image, a field at a time, in degrees about each axis. */
  applyRotation: (rotation: Partial<SceneRotation>) => void;
  applyShuffle: () => void;
  applyStyle: (style: Partial<AnnotationStyle>) => void;
  /** Sway the chosen image over its clip, or stand it still. */
  applySway: (sway: boolean) => void;
  /** Whether the picture the workspace edits has any stroke to clear. */
  canClearDrawings: boolean;
  /** How many annotations are chosen; only one on its own is `selection`. */
  count: number;
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
 * is mounted, and whether it has strokes to clear.
 *
 * The selection is compared by value: the preview rebuilds it on every render,
 * and a panel that re-rendered on identity alone would never settle.
 */
export function usePublishAnnotationSelection(
  workspace: EditorKind,
  {
    canClearDrawings,
    count,
    selection,
  }: Pick<PublishedAnnotation, "canClearDrawings" | "count" | "selection">,
  apply: Omit<PublishedAnnotation, "canClearDrawings" | "count" | "selection">,
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
      applyClearDrawings: () => {
        applyRef.current.applyClearDrawings();
      },
      applyDelete: () => {
        applyRef.current.applyDelete();
      },
      applyEmphasis: (emphasis) => {
        applyRef.current.applyEmphasis(emphasis);
      },
      applyImage: (art) => {
        applyRef.current.applyImage(art);
      },
      applyImagePlay: (play) => {
        applyRef.current.applyImagePlay(play);
      },
      applyLook: (look) => {
        applyRef.current.applyLook(look);
      },
      applyReverse: () => {
        applyRef.current.applyReverse();
      },
      applyRotation: (rotation) => {
        applyRef.current.applyRotation(rotation);
      },
      applyShuffle: () => {
        applyRef.current.applyShuffle();
      },
      applyStyle: (style) => {
        applyRef.current.applyStyle(style);
      },
      applySway: (sway) => {
        applyRef.current.applySway(sway);
      },
      canClearDrawings,
      count,
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
  }, [canClearDrawings, count, serialized, workspace]);
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
 * draws in, or nothing. With several chosen there is no one annotation to
 * dress, so the dress of the next one is not shown either. */
export const useAnnotationSelection = (workspace: EditorKind) =>
  useSyncExternalStore(subscribe, () => {
    const published = workspaces.get(workspace);
    if ((published?.count ?? 0) > 1) return null;
    return published?.selection ?? drafts.get(workspace) ?? null;
  });

/** How many annotations this workspace has chosen. */
export const useAnnotationSelectionCount = (workspace: EditorKind) =>
  useSyncExternalStore(subscribe, () => workspaces.get(workspace)?.count ?? 0);

/** Whether the picture this workspace edits has any stroke to clear. */
export const useCanClearDrawings = (workspace: EditorKind) =>
  useSyncExternalStore(
    subscribe,
    () => workspaces.get(workspace)?.canClearDrawings ?? false,
  );

/** The workspace's published edits when it has an annotation chosen. */
export const chosenAnnotation = (workspace: EditorKind) => {
  const published = workspaces.get(workspace);
  return published?.selection ? published : null;
};

/** The dress the tool in hand draws its next annotation in, if it draws one. */
export const annotationDraft = (workspace: EditorKind) => drafts.get(workspace);

/** The workspace's published edits, whatever it has chosen. */
export const publishedAnnotations = (workspace: EditorKind) =>
  workspaces.get(workspace);
