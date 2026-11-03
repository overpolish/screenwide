// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SceneRotation } from "../layer-rotation";

import { textSelection } from "./annotation-text-selection";
import { Annotation, ImageArt } from "./annotations";

import type {
  AnnotationPinAction,
  AnnotationPinView,
} from "../island/annotation/annotation-pin";
import type { ToolPanelAnnotation } from "../tools/tool-panel-selection";

/** What a workspace whose annotations can be pinned tells of them: how the
 * pin of the one named stands at the playhead (null while it is not pinned,
 * absent where it cannot be), and acting on it. */
export type AnnotationPins = {
  apply: (id: string, action: AnnotationPinAction) => void;
  viewOf: (id: string) => AnnotationPinView | null | undefined;
};

/** The chosen annotation as its island shows it: its dress, and what only
 * some kinds have, each left out where it has none. */
export const publishedAnnotation = (
  selected: Annotation,
  {
    angle,
    art,
    pin,
    rotation,
    zoom,
  }: {
    angle: number | null;
    art: ImageArt | null;
    pin: AnnotationPinView | null | undefined;
    rotation: SceneRotation | undefined;
    zoom: number | null;
  },
): ToolPanelAnnotation => ({
  angle: angle ?? undefined,
  animated: selected.animated,
  id: selected.id,
  image: art ?? undefined,
  kind: selected.shape.kind,
  ...textSelection(selected.shape),
  style: selected.style,
  ...(selected.shape.kind === "image"
    ? { rotation, sway: selected.shape.sway !== undefined }
    : {}),
  ...(zoom === null ? {} : { zoom }),
  ...(pin === undefined ? {} : { pin }),
});
