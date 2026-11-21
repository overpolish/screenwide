// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationAlign } from "../../../../bindings/AnnotationAlign";
import type { AnnotationHead } from "../../../../bindings/AnnotationHead";
import type { AnnotationRedaction } from "../../../../bindings/AnnotationRedaction";
import type { SceneRotation } from "../../../../bindings/SceneRotation";
import type { ToolPanelPatch } from "../../tools/tool-panel-patch";

// The islands speak of an annotation as it looks (`isHandDrawn`, `size`);
// the editor dresses one through the fields of a `ToolPanelPatch`, so each
// change is put in those words here.

/** Every field any annotation island changes, under the island's names. */
export type IslandStyle = Partial<{
  align: AnnotationAlign;
  angle: number;
  color: string;
  dims: boolean;
  hasShadow: boolean;
  head: AnnotationHead;
  isAnimated: boolean;
  isHandDrawn: boolean;
  isManual: boolean;
  isTinted: boolean;
  loops: boolean;
  radius: number;
  redaction: AnnotationRedaction;
  rotation: SceneRotation;
  size: number;
  strength: number;
  sways: boolean;
  /** An image drawn larger than its picture is upscaled by the model
   * rather than stretched. */
  upscales: boolean;
  width: number;
  zoom: number;
}>;

/** An island's change, in the words the editor dresses an annotation by. */
export const patchOf = (style: IslandStyle): ToolPanelPatch => {
  const {
    angle,
    dims: _dims,
    hasShadow,
    isAnimated,
    isHandDrawn,
    isManual,
    isTinted,
    loops,
    rotation,
    size,
    sways,
    upscales,
    zoom,
    ...dress
  } = style;
  const annotationStyle = {
    ...dress,
    ...(hasShadow === undefined ? {} : { shadow: hasShadow }),
    ...(isHandDrawn === undefined ? {} : { handDrawn: isHandDrawn }),
    ...(isManual === undefined ? {} : { manual: isManual }),
    ...(isTinted === undefined ? {} : { tint: isTinted }),
    ...(size === undefined ? {} : { width: size }),
  };
  return {
    ...(Object.keys(annotationStyle).length > 0 ? { annotationStyle } : {}),
    ...(angle === undefined ? {} : { annotationAngle: angle }),
    ...(isAnimated === undefined ? {} : { annotationAnimated: isAnimated }),
    ...(loops === undefined ? {} : { annotationImagePlay: { once: !loops } }),
    ...(rotation === undefined ? {} : { annotationRotation: rotation }),
    ...(sways === undefined ? {} : { annotationImageSway: sways }),
    ...(upscales === undefined ? {} : { annotationImageStretch: !upscales }),
    ...(zoom === undefined ? {} : { annotationZoom: zoom }),
  };
};
