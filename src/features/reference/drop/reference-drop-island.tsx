// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { PictureInPicture2 } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { Island } from "../../../primitives/island/island";
import { IslandParts } from "../../../primitives/island/island-parts";
import { type IslandProps } from "../../../primitives/island/island-props";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";

// The island while a picture is dragged with Option held: letting go keeps
// it as a reference, said at the row's start with the glyph Quick
// Screenshot's island gives a reference. Nothing on it is pressed: the drag
// holds the pointer, and letting Option go puts it away.

export type ReferenceDropIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
>;

export function ReferenceDropIsland({
  isShown = true,
  notch,
  onHidden,
  onShapeChange,
}: ReferenceDropIslandProps) {
  return (
    <Island
      aria-label={t("reference-drop-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          start={
            <IslandParts direction="row">
              <IslandRowTitle
                glyph={<PictureInPicture2 aria-hidden />}
                key="drop"
                title={<span role="status">{t("reference-drop-hint")}</span>}
              />
            </IslandParts>
          }
        />
      }
    />
  );
}
