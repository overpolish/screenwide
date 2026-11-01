// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { type CropAspect, CropAspectChoice } from "../crop-aspect-choice";
import { SceneOnlyNote } from "../scene-only-note";

import type { EditorIslandContent } from "../editor-island";

// The Crop tool's island: the crop of the layer in hand, the screen or the
// camera of a recording or a screenshot's layer. How much the crop keeps is
// dragged on the picture with its handles, where a size can be seen rather
// than read as a number; here it only takes a shape, kept as the handles
// are dragged until another is chosen or the crop goes free again. Over a
// scene the crop is that scene's own, as a note under the shapes says. Last,
// giving the whole capture back, the scene's crop over a scene.

export type CropIslandProps = {
  /** The shape the crop keeps, the layer's own, and the scene's while
   * `isInScene`. */
  aspect: CropAspect;
  /** Whether a free crop has been drawn, whose shape can be locked. */
  canLock: boolean;
  /** Whether the playhead is in a recording's scene, whose own crop the
   * island then sets. A screenshot has no scenes. */
  isInScene: boolean;
  onAspectChange: (aspect: CropAspect) => void;
  /** Gives the whole capture back, free of any shape. */
  onReset: () => void;
};

export function cropIsland(props: CropIslandProps): EditorIslandContent {
  return {
    key: "crop",
    section: (
      <IslandFlow className="flex flex-col gap-separate">
        <AnnotationLine key="aspect">
          <CropAspectChoice
            canLock={props.canLock}
            onChange={props.onAspectChange}
            value={props.aspect}
          />
        </AnnotationLine>
        {props.isInScene ? (
          <SceneOnlyNote key="scene-note" layer="crop" />
        ) : null}
        <div className="flex justify-end" key="actions">
          <Button onPress={props.onReset}>
            {t("editor-island-reset-crop")}
          </Button>
        </div>
      </IslandFlow>
    ),
    title: (
      <IslandRowTitle
        glyph={<Crop aria-hidden className="size-icon" />}
        title={t("editor-tool-crop")}
      />
    ),
  };
}
