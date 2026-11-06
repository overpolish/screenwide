// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useSyncExternalStore } from "react";

import { SceneRotation } from "../layer-rotation";
import { ToolPanelAnnotation } from "../tools/tool-panel-selection";
import { EditorKind } from "../types";

import { AnnotationStyle, ImageArt, ImagePlay, TextSpan } from "./annotations";

import type { TextStyle } from "../../../bindings/TextStyle";
import type { AnnotationPinAction } from "../island/annotation/annotation-pin";

/** What the panel changes about how a moving image plays. */
export type ImagePlayChange = Partial<Pick<ImagePlay, "frame" | "once">>;

/**
 * The annotation the preview has in hand, and the edits that dress it,
 * reachable from outside the preview that owns them.
 *
 * Which annotation is chosen is the preview's own business: the native tool
 * hit-tests it and reports it back, and the tool that did so keeps it. The
 * pen's island arrives by another road entirely - the island window, through
 * the editor's bridge - and has to reach the same commit path the drag on the
 * picture uses, so each workspace leaves what it has in hand here for the
 * island to find.
 */
type PublishedAnnotation = {
  applyAngle: (angle: number) => void;
  applyAnimated: (animated: boolean) => void;
  /** Delete every chosen annotation, in one edit. */
  applyDelete: () => void;
  /** Set apart the chosen text's words `emphasis`, and no others. */
  applyEmphasis: (emphasis: TextSpan[]) => void;
  /** Give the chosen image another picture. */
  applyImage: (art: ImageArt) => void;
  /** Change how the chosen moving image plays. */
  applyImagePlay: (play: ImagePlayChange) => void;
  /** Turn the chosen focus into a spotlight, or the spotlight into a focus. */
  applyKind: (kind: "focus" | "spotlight") => void;
  applyLook: (look: TextStyle) => void;
  /** Act on the chosen annotation's pin, where it can be pinned. */
  applyPin: (action: AnnotationPinAction) => void;
  applyReverse: () => void;
  /** Turn the chosen image, a field at a time, in degrees about each axis. */
  applyRotation: (rotation: Partial<SceneRotation>) => void;
  applyShuffle: () => void;
  applyStyle: (style: Partial<AnnotationStyle>) => void;
  /** Sway the chosen image over its clip, or stand it still. */
  applySway: (sway: boolean) => void;
  /** Set how many times the chosen lens enlarges. */
  applyZoom: (zoom: number) => void;
  /** How many annotations are chosen; only one on its own is `selection`. */
  count: number;
  /** Take away the chosen annotation's own place in the scene at the
   * playhead. */
  resetScene: () => void;
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
 * is mounted, and how many are chosen.
 *
 * The selection is compared by value: the preview rebuilds it on every render,
 * and a panel that re-rendered on identity alone would never settle.
 */
export function usePublishAnnotationSelection(
  workspace: EditorKind,
  { count, selection }: Pick<PublishedAnnotation, "count" | "selection">,
  apply: Omit<PublishedAnnotation, "count" | "selection">,
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
      applyKind: (kind) => {
        applyRef.current.applyKind(kind);
      },
      applyLook: (look) => {
        applyRef.current.applyLook(look);
      },
      applyPin: (action) => {
        applyRef.current.applyPin(action);
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
      applyZoom: (zoom) => {
        applyRef.current.applyZoom(zoom);
      },
      count,
      resetScene: () => {
        applyRef.current.resetScene();
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
  }, [count, serialized, workspace]);
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
