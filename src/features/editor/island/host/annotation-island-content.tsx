// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NEW_LENS_ZOOM } from "../../annotations/annotation-shape-parsers";
import { browseAnnotationImage } from "../../images/image-api";
import { shownTextLook } from "../../tools/annotation/text/text-look";
import { drawingToolShortcut } from "../../tools/tool-registry";
import { forwardToolPanelShortcut } from "../../tools/use-panel-shortcuts";
import { arrowIsland } from "../annotation/arrow/arrow-island";
import { counterIsland } from "../annotation/counter/counter-island";
import { drawingIsland } from "../annotation/drawing/drawing-island";
import { focusIsland } from "../annotation/focus/focus-island";
import { highlightIsland } from "../annotation/highlight/highlight-island";
import { imageIsland } from "../annotation/image/image-island";
import { magnifierIsland } from "../annotation/magnifier/magnifier-island";
import { redactionIsland } from "../annotation/redaction/redaction-island";
import { shapeIsland } from "../annotation/shape/shape-island";
import { textIsland } from "../annotation/text/text-island";
import { marqueeIsland } from "../marquee/marquee-island";

import type { AnnotationAlign } from "../../../../bindings/AnnotationAlign";
import type { AnnotationHead } from "../../../../bindings/AnnotationHead";
import type { AnnotationRedaction } from "../../../../bindings/AnnotationRedaction";
import type { SceneRotation } from "../../../../bindings/SceneRotation";
import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorKind } from "../../types";
import type { AnnotationPinAction } from "../annotation/annotation-pin";
import type { EditorIslandContent } from "../editor-island";

// The annotation in hand on its pen's island, or the pen's next one with
// nothing chosen, or how many the marquee chose. The islands speak of an
// annotation as it looks (`isHandDrawn`, `size`); the editor dresses one
// through the fields of a `ToolPanelPatch`, so each change is put in those
// words here.

/** Every field any annotation island changes, under the island's names. */
type IslandStyle = Partial<{
  align: AnnotationAlign;
  angle: number;
  color: string;
  dims: boolean;
  hasShadow: boolean;
  head: AnnotationHead;
  isAnimated: boolean;
  isHandDrawn: boolean;
  isManual: boolean;
  loops: boolean;
  radius: number;
  redaction: AnnotationRedaction;
  rotation: SceneRotation;
  size: number;
  strength: number;
  sways: boolean;
  width: number;
  zoom: number;
}>;

/** An island's change, in the words the editor dresses an annotation by. */
const patchOf = (style: IslandStyle): ToolPanelPatch => {
  const {
    angle,
    dims: _dims,
    hasShadow,
    isAnimated,
    isHandDrawn,
    isManual,
    loops,
    rotation,
    size,
    sways,
    zoom,
    ...dress
  } = style;
  const annotationStyle = {
    ...dress,
    ...(hasShadow === undefined ? {} : { shadow: hasShadow }),
    ...(isHandDrawn === undefined ? {} : { handDrawn: isHandDrawn }),
    ...(isManual === undefined ? {} : { manual: isManual }),
    ...(size === undefined ? {} : { width: size }),
  };
  return {
    ...(Object.keys(annotationStyle).length > 0 ? { annotationStyle } : {}),
    ...(angle === undefined ? {} : { annotationAngle: angle }),
    ...(isAnimated === undefined ? {} : { annotationAnimated: isAnimated }),
    ...(loops === undefined ? {} : { annotationImagePlay: { once: !loops } }),
    ...(rotation === undefined ? {} : { annotationRotation: rotation }),
    ...(sways === undefined ? {} : { annotationImageSway: sways }),
    ...(zoom === undefined ? {} : { annotationZoom: zoom }),
  };
};

const NO_TURN: SceneRotation = { x: 0, y: 0, z: 0 };

export function annotationIslandContent({
  change,
  snapshot,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
  workspace: EditorKind;
}): EditorIslandContent | null {
  const { annotation, annotationColors, annotationCount, textTemplates } =
    snapshot;
  if (annotationCount > 1)
    return marqueeIsland({
      count: annotationCount,
      onDelete: () => {
        change({ deleteAnnotations: true });
      },
    });
  if (!annotation) return null;
  const isDraft = annotation.isDraft === true;
  const canAnimate = workspace === "recording";
  const { style } = annotation;
  const onStyleChange = (next: IslandStyle) => {
    // A focus dimmed is a spotlight, and a spotlight blurred a focus: the
    // two are one pen with two looks.
    if (next.dims !== undefined) {
      const kind = next.dims ? "spotlight" : "focus";
      if (isDraft)
        forwardToolPanelShortcut(workspace, `Key${drawingToolShortcut(kind)}`);
      else change({ annotationKind: kind });
    }
    const patch = patchOf(next);
    if (Object.keys(patch).length > 0) change(patch);
  };
  const shared = {
    canAnimate,
    id: annotation.id,
    onPinAction: (action: AnnotationPinAction) => {
      change({ annotationPin: action });
    },
    onStyleChange,
    pin: annotation.pin,
    ...(isDraft
      ? {}
      : {
          onDelete: () => {
            change({ deleteAnnotations: true });
          },
        }),
  };
  const colors = {
    onRemoveColor: (color: string) => {
      change({ removeAnnotationColor: color });
    },
    onSaveColor: (color: string) => {
      change({ saveAnnotationColor: color });
    },
    savedColors: annotationColors,
  };
  const isAnimated = annotation.animated;
  const reverse = () => {
    change({ reverseAnnotation: true });
  };
  switch (annotation.kind) {
    case "arrow":
      return arrowIsland({
        ...shared,
        ...colors,
        onReverse: reverse,
        style: {
          color: style.color,
          head: style.head,
          isAnimated,
          width: style.width,
        },
      });
    case "counter":
      return counterIsland({
        ...shared,
        ...colors,
        style: {
          angle: annotation.angle ?? 0,
          color: style.color,
          isAnimated,
          size: style.width,
        },
      });
    case "draw":
      return drawingIsland({
        ...shared,
        ...colors,
        style: { color: style.color, isAnimated, width: style.width },
      });
    case "focus":
    case "spotlight":
      return focusIsland({
        ...shared,
        style: {
          dims: annotation.kind === "spotlight",
          isAnimated,
          radius: style.radius,
        },
      });
    case "highlight":
      return highlightIsland({
        ...shared,
        ...colors,
        style: {
          color: style.color,
          isAnimated,
          isHandDrawn: style.handDrawn,
          isManual: style.manual,
        },
      });
    case "image":
      return imageIsland({
        ...shared,
        isMoving: Boolean(annotation.image?.play),
        onFlip: reverse,
        onReplace: () => {
          browseAnnotationImage()
            .then((art) => {
              if (art) change({ annotationImage: art });
            })
            .catch((cause: unknown) => {
              console.error("Could not use that picture", cause);
            });
        },
        style: {
          hasShadow: style.shadow,
          isAnimated,
          loops: annotation.image?.play?.once !== true,
          radius: style.radius,
          rotation: annotation.rotation ?? NO_TURN,
          sways: annotation.sway === true,
        },
      });
    case "magnify":
      return magnifierIsland({
        ...shared,
        style: {
          isAnimated,
          radius: style.radius,
          zoom: annotation.zoom ?? NEW_LENS_ZOOM,
        },
      });
    case "redact":
      return redactionIsland({
        ...shared,
        ...colors,
        onShuffle: () => {
          change({ shuffleAnnotation: true });
        },
        style: {
          color: style.color,
          isAnimated,
          radius: style.radius,
          redaction: style.redaction,
          strength: style.strength,
          width: style.width,
        },
      });
    case "shape":
      return shapeIsland({
        ...shared,
        ...colors,
        style: {
          color: style.color,
          isAnimated,
          isHandDrawn: style.handDrawn,
          radius: style.radius,
          width: style.width,
        },
      });
    case "text": {
      if (!textTemplates) return null;
      const look = shownTextLook(annotation.look, style.color, textTemplates);
      if (!look) return null;
      return textIsland({
        ...shared,
        ...colors,
        emphasis: annotation.emphasis ?? [],
        look,
        onEmphasisChange: (emphasis) => {
          change({ annotationEmphasis: emphasis });
        },
        onLookChange: (patch) => {
          change({ annotationLook: { ...look, ...patch } });
        },
        style: { align: style.align, isAnimated, size: style.width },
        templates: textTemplates,
        text: annotation.text ?? "",
      });
    }
  }
}
