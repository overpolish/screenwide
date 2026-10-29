// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { AnnotationLine } from "../annotation/annotation-island-parts";

import type { EditorIslandContent } from "../editor-island";

// The shape the crop keeps. How much it keeps is dragged on the picture with
// its handles, where a size can be seen rather than read as a number; here
// it only takes a shape, kept as the handles are dragged until another is
// chosen or the crop goes free again. Then giving the whole capture back.

/** The shape a crop keeps: none, the whole capture's, or a ratio of width to
 * height. */
type CropAspect = "16:9" | "1:1" | "4:3" | "9:16" | "free" | "original";

/** The ratios offered, widest first after the square, as width to height. */
const RATIOS = [
  { height: 1, id: "1:1", width: 1 },
  { height: 3, id: "4:3", width: 4 },
  { height: 9, id: "16:9", width: 16 },
  { height: 16, id: "9:16", width: 9 },
] as const;

export type CropIslandProps = {
  aspect: CropAspect;
  onAspectChange: (aspect: CropAspect) => void;
  /** Gives the whole capture back, free of any shape. */
  onReset: () => void;
};

export function cropIsland(props: CropIslandProps): EditorIslandContent {
  return {
    key: "crop",
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationLine>
          <PillGroup
            aria-label={t("editor-island-crop-aspect")}
            items={[
              { id: "free", label: t("editor-island-crop-free") },
              { id: "original", label: t("editor-island-crop-original") },
              ...RATIOS.map(({ height, id, width }) => ({
                ariaLabel: t("editor-island-crop-ratio-name", {
                  height,
                  width,
                }),
                id,
                label: t("editor-island-crop-ratio", { height, width }),
              })),
            ]}
            onSelectionChange={(aspect) => {
              props.onAspectChange(aspect as CropAspect);
            }}
            selected={props.aspect}
          />
        </AnnotationLine>
        <div className="flex justify-end">
          <Button onPress={props.onReset}>{t("editor-panels-reset")}</Button>
        </div>
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<Crop aria-hidden className="size-icon" />}
        title={t("editor-tool-crop")}
      />
    ),
  };
}
