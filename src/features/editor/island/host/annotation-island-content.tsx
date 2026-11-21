// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NEW_LENS_ZOOM } from "../../annotations/annotation-shape-parsers";
import { browseAnnotationImage } from "../../images/image-api";
import { shownTextLook } from "../../tools/annotation/text/text-look";
import { drawingToolKey } from "../../tools/tool-registry";
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

import { type IslandStyle, patchOf } from "./annotation-island-patch";

import type { SceneRotation } from "../../../../bindings/SceneRotation";
import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorKind } from "../../types";
import type { AnnotationPinAction } from "../annotation/annotation-pin";
import type { EditorIslandContent } from "../editor-island";

// The annotation in hand on its pen's island, or the pen's next one with
// nothing chosen, or how many Select's band chose.

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
      if (isDraft) {
        const { code, modifiers } = drawingToolKey(kind);
        forwardToolPanelShortcut(workspace, code, modifiers);
      } else change({ annotationKind: kind });
    }
    const patch = patchOf(next);
    if (Object.keys(patch).length > 0) change(patch);
  };
  const shared = {
    canAnimate,
    onPinAction: (action: AnnotationPinAction) => {
      change({ annotationPin: action });
    },
    onSceneReset: () => {
      change({ annotationSceneReset: true });
    },
    onStyleChange,
    pin: annotation.pin,
    placedInScene: annotation.placedInScene,
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
          isTinted: style.tint,
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
          upscales: annotation.stretched !== true,
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
