// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The pen island's edits, applied to the annotation a workspace has chosen,
 * or with none chosen, remembered as the dress the next one is drawn in.
 */

import { requestImagePlacement } from "../images/image-requests";
import { SceneRotation } from "../layer-rotation";
import { EditorKind } from "../types";

import {
  annotationDraft,
  chosenAnnotation,
  publishedAnnotations,
  ImagePlayChange,
} from "./annotation-channel";
import {
  rememberAnnotationAngle,
  rememberAnnotationAnimated,
  rememberAnnotationImage,
  rememberAnnotationStyle,
  rememberAnnotationTextLook,
  rememberAnnotationZoom,
} from "./annotation-defaults";
import { AnnotationStyle, ImageArt, TextSpan } from "./annotations";

import type { TextStyle } from "../../../bindings/TextStyle";
import type { AnnotationPinAction } from "../island/annotation/annotation-pin";

/** Dress the chosen annotation, or with none chosen, the next one the tool in
 * hand draws. A no-op when the workspace has neither to dress. */
export const applyAnnotationStyle = (
  workspace: EditorKind,
  style: Partial<AnnotationStyle>,
) => {
  const published = chosenAnnotation(workspace);
  if (published) {
    published.applyStyle(style);
    return;
  }
  const draft = annotationDraft(workspace);
  if (draft) rememberAnnotationStyle({ ...draft.style, ...style }, draft.kind);
};

/** Draw the chosen annotation in and out over its clip, or leave it standing;
 * with none chosen, the next one. */
export const applyAnnotationAnimated = (
  workspace: EditorKind,
  animated: boolean,
) => {
  const published = chosenAnnotation(workspace);
  if (published) published.applyAnimated(animated);
  else if (annotationDraft(workspace) !== undefined)
    rememberAnnotationAnimated(animated);
};

/** Turn the chosen counter's tail to `angle`, in radians clockwise from
 * east; with none chosen, the next counter's. */
export const applyAnnotationAngle = (workspace: EditorKind, angle: number) => {
  const published = chosenAnnotation(workspace);
  if (published) published.applyAngle(angle);
  else if (annotationDraft(workspace) !== undefined)
    rememberAnnotationAngle(angle);
};

/** Set how many times the chosen lens enlarges; with none chosen, the next
 * lens's. */
export const applyAnnotationZoom = (workspace: EditorKind, zoom: number) => {
  const published = chosenAnnotation(workspace);
  if (published) published.applyZoom(zoom);
  else if (annotationDraft(workspace) !== undefined)
    rememberAnnotationZoom(zoom);
};

/** Turn the chosen annotation round. A no-op with none chosen: there is
 * nothing drawn yet to turn. */
export const applyAnnotationReverse = (workspace: EditorKind) => {
  chosenAnnotation(workspace)?.applyReverse();
};

/** Pin the chosen annotation to its content, let it go, say its content is
 * out of view or back at the playhead, or take its corrections away. A
 * no-op with none chosen, or one that cannot be pinned. */
export const applyAnnotationPin = (
  workspace: EditorKind,
  action: AnnotationPinAction,
) => {
  chosenAnnotation(workspace)?.applyPin(action);
};

/** Take away the chosen annotation's own place in the scene at the
 * playhead, so it stands there where it does outside every scene. A no-op
 * with none chosen, or outside every scene. */
export const applyAnnotationSceneReset = (workspace: EditorKind) => {
  chosenAnnotation(workspace)?.resetScene();
};

/** Turn the chosen image, a field at a time. A no-op with none chosen: a
 * fresh image always lands upright. */
export const applyAnnotationRotation = (
  workspace: EditorKind,
  rotation: Partial<SceneRotation>,
) => {
  chosenAnnotation(workspace)?.applyRotation(rotation);
};

/** Give the chosen image the picture `art`. With none chosen, an image
 * showing it is placed in the middle of the layer in hand, and the next
 * image shows it too. */
export const applyAnnotationImage = (workspace: EditorKind, art: ImageArt) => {
  const published = chosenAnnotation(workspace);
  if (published) {
    published.applyImage(art);
    return;
  }
  if (annotationDraft(workspace) === undefined) return;
  rememberAnnotationImage(art);
  requestImagePlacement(workspace, art);
};

/** Change how the chosen moving image plays. A no-op with none chosen: a
 * fresh image always starts on its first frame and loops. */
export const applyAnnotationImagePlay = (
  workspace: EditorKind,
  play: ImagePlayChange,
) => {
  chosenAnnotation(workspace)?.applyImagePlay(play);
};

/** Lay the chosen redaction's blocks out again, draw the chosen hand-drawn
 * highlight's stroke again, or sway the chosen image another way, from a
 * fresh seed. A no-op with none chosen: every fresh one is drawn from a seed
 * of its own. */
export const applyAnnotationShuffle = (workspace: EditorKind) => {
  chosenAnnotation(workspace)?.applyShuffle();
};

/** Turn the chosen focus into a spotlight, or the spotlight into a focus. A
 * no-op with none chosen: the pen in hand changes look by changing tool. */
export const applyAnnotationKind = (
  workspace: EditorKind,
  kind: "focus" | "spotlight",
) => {
  chosenAnnotation(workspace)?.applyKind(kind);
};

/** Sway the chosen image over its clip, or stand it still. A no-op with
 * none chosen: a fresh image always stands still. */
export const applyAnnotationSway = (workspace: EditorKind, sway: boolean) => {
  chosenAnnotation(workspace)?.applySway(sway);
};

/** Delete every annotation this workspace has chosen. */
export const applyAnnotationsDelete = (workspace: EditorKind) => {
  publishedAnnotations(workspace)?.applyDelete();
};

/** Dress the chosen text in `look`; with none chosen, the next text. */
export const applyAnnotationLook = (workspace: EditorKind, look: TextStyle) => {
  const published = chosenAnnotation(workspace);
  if (published) published.applyLook(look);
  else if (annotationDraft(workspace) !== undefined)
    rememberAnnotationTextLook(look);
};

/** Set apart the chosen text's words `emphasis`, and no others. A no-op with
 * none chosen: a text has words to choose only once it is typed. */
export const applyAnnotationEmphasis = (
  workspace: EditorKind,
  emphasis: TextSpan[],
) => {
  chosenAnnotation(workspace)?.applyEmphasis(emphasis);
};
