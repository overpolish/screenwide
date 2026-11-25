// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { ScrollArea } from "../../../../../primitives/scroll-area/scroll-area";
import { ImageToolIcon } from "../../../../annotations/annotation-tool-icons";
import { LibraryPictureGrid } from "../../library/library-picture-grid";
import { AnnotationAnimateToggle } from "../annotation-island-parts";

import type { ImageArt } from "../../../annotations/annotations";
import type { EditorIslandContent } from "../../editor-island";

// The image pen in hand with no image chosen: the library's pictures, led
// by a card that adds more. Pressing one places it as a new image in the
// middle of the layer in hand, chosen, so the island turns to the placed
// image's own; letting it go brings the pictures back. The row keeps
// whether the next image pops in, which only a recording has a clip for;
// the corners and shadow are chosen on the placed image, which the next one
// remembers.

/** How wide the grid stands, in px: four tiles to a row, whatever the
 * library holds, so the island does not change width as pictures arrive. */
const GRID_PX = 320;

export type ImagePickIslandProps = {
  canAnimate: boolean;
  isAnimated: boolean;
  onAnimatedChange: (isAnimated: boolean) => void;
  onPlace: (art: ImageArt) => void;
};

export function imagePickIsland(
  props: ImagePickIslandProps,
): EditorIslandContent {
  return {
    actions: props.canAnimate ? (
      <AnnotationAnimateToggle
        isAnimated={props.isAnimated}
        onChange={props.onAnimatedChange}
      />
    ) : null,
    anchor: "end",
    key: "image",
    section: (
      <div style={{ width: GRID_PX }}>
        <ScrollArea constrainHeight rootClassName="max-h-80">
          <LibraryPictureGrid
            hasAddCard
            hasNames={false}
            onAdd={props.onPlace}
          />
        </ScrollArea>
      </div>
    ),
    sectionKey: "pick",
    title: (
      <IslandRowTitle
        glyph={<ImageToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-image")}
      />
    ),
  };
}
