// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { LayoutGroup, motion } from "motion/react";
import { type ReactNode, useMemo, useRef, useState } from "react";
import { useLocale } from "react-aria";

import { SkinContext } from "../skin/skin";

import { useAfterLeave } from "./island-after-leave";
import { useControlAnchor } from "./island-anchor";
import { IslandBelow } from "./island-below";
import { IslandAsideContext, IslandContext } from "./island-context";
import { IslandDrawing } from "./island-drawing";
import { IslandFollowContext, useIslandFollow } from "./island-follow";
import {
  type IslandAnchor,
  islandCssVariables,
  type IslandNotch,
} from "./island-geometry";
import { useIslandHover } from "./island-hover";
import { useIslandMeasures } from "./island-measures";
import { fade } from "./island-motion";
import { useOpening } from "./island-opening";
import {
  type IslandPart,
  useIslandOutline,
  useIslandShapeValues,
} from "./island-outline-sync";
import { useIslandPresence } from "./island-presence";
import { IslandRowBox } from "./island-row-box";
import { useRowOffset } from "./island-row-offset";
import { IslandSection } from "./island-section";
import { layOutIsland } from "./island-stack-layout";

// A shape at the top of the screen in the notch's black: a row of controls,
// and beneath it a section only as wide as what it holds, hung under the
// part of the row it belongs to. Where the two meet the black flows round a
// curved inner corner, so it reads as one shape, and it is drawn as one: an
// outline worked out from where its parts are (`island-outline.ts`), so no
// seam can open between them. A section that would leave only a sliver of
// row beside it takes the row's width instead, and one wider than the row
// widens the row with it. Each part grows by changing its real size, so
// what it holds is laid out once, in its place, and only revealed as the
// shape opens around it. Around a notch each side of the row is only as
// wide as it needs, and the whole shape moves sideways by half the
// difference so the notch stays over its gap. Hidden, it tucks into the
// notch, or shrinks to a speck, or nothing, and fades where there is none.
//
// The row is a toolbar: arrow keys move between its controls. The section is
// outside it, so a list in the section keeps its own arrow keys. Nothing on
// it has a tooltip (`island-controls.tsx`).

export type IslandProps = {
  "aria-label": string;
  /** The controls along the top, usually an `IslandRow`. */
  row: ReactNode;
  /** Where under the row the section hangs. */
  anchor?: IslandAnchor;
  /** A row control to centre the section under, in place of `anchor`. */
  anchorControl?: HTMLElement | null;
  // Centred under the notch whatever the row does beside it, so a row growing
  // on one side widens the island on that side alone and nothing under it
  // moves (`island-stack-layout.ts`).
  /** A second line under the row, in its black; the section hangs below. */
  below?: ReactNode;
  /** Whether it is out; hidden, it is tucked into the notch, or gone. */
  isShown?: boolean;
  /** The screen's measured notch; left out on a screen without one. */
  notch?: IslandNotch | null;
  /** The island has tucked itself away after `isShown` turned false. */
  onHidden?: () => void;
  // Leaving is reported a moment after the pointer goes, so a slip does not
  // close what hovering opened.
  /** The pointer coming onto the row or section, or leaving them. */
  onHoverChange?: (isHovered: boolean) => void;
  /** The row's and section's places in the page, for click-through, and
   * from the island's own middle and top, for placing it. */
  onShapeChange?: (parts: IslandPart[], fromMiddle: IslandPart[]) => void;
  // So its parts keep to the island's ends rather than centring over the
  // black. Not under a notch, where the row keeps its gap under the notch.
  /** Stretches the row to a wider section's width. */
  rowFillsWidth?: boolean;
  /** What hangs beneath the row, if anything. */
  section?: ReactNode;
  /** Makes each section at least the row's width, as one block. */
  sectionSpansRow?: boolean;
  // For an island on a picture, where a speck would read as something left
  // behind.
  /** Without a notch, shrinks past a speck to nothing. */
  shrinksToNothing?: boolean;
};

export function Island({
  anchor = "center",
  anchorControl,
  "aria-label": label,
  below,
  isShown: isAsked = true,
  notch = null,
  onHidden,
  onHoverChange,
  onShapeChange,
  row,
  rowFillsWidth = false,
  section,
  sectionSpansRow = false,
  shrinksToNothing = false,
}: IslandProps) {
  const fillsWidth = rowFillsWidth && notch === null;
  const measured = useIslandMeasures(fillsWidth);
  const { rowElement } = measured;
  const [shift, setShift] = useState(0);

  const hover = useIslandHover(onHoverChange);

  const { contentOpacity, isComing, isShown } = useIslandPresence(isAsked);
  const context = useMemo(
    () => ({ isShown, notch, setShift }),
    [isShown, notch],
  );

  const isStacked = isShown && Boolean(below);
  const rowShift = isShown && notch ? shift : 0;
  const { direction } = useLocale();
  const controlAnchor = useControlAnchor(anchorControl, rowElement, direction);
  const settledAnchor = useAfterLeave(anchor, { isShown });
  const layout = layOutIsland({
    anchor: controlAnchor ?? settledAnchor,
    below: isStacked ? (measured.belowSize ?? { height: 0, width: 0 }) : null,
    direction,
    hasSection: Boolean(section),
    isShown,
    notch,
    reach: measured.reach,
    rowShift,
    rowSize: measured.rowSize,
    sectionSize: measured.sectionSize,
    shrinksToNothing,
    spansRow: sectionSpansRow,
  });
  const { flare, isOpen, radius, rowTarget, top, width } = layout;
  // A section opens out of the middle, and folds back into it, only as the
  // island comes out of the notch or goes back in.
  const isFromMiddle = !isShown || isComing;
  const { finish, isOpening } = useOpening(isOpen);
  const { follow, travel } = useIslandFollow();

  const { islandX, rowLayerRef, rowX, sectionX, shapeWidth } = useRowOffset({
    fillsWidth,
    hasNotch: notch !== null,
    holdsContent: !isShown,
    rowElement,
    rowTarget: rowShift,
    shiftTarget: layout.islandShift,
    travel,
    width,
  });

  // The parts are laid out and animated as boxes that paint nothing; the
  // black is their outline, drawn from the same animated values.
  const values = useIslandShapeValues({ flare, radius, top }, travel);
  const rowPathRef = useRef<SVGPathElement>(null);
  const shapePathRef = useRef<SVGPathElement>(null);
  const clipRef = useRef<HTMLDivElement>(null);
  useIslandOutline({
    clipRef,
    onShapeChange,
    rowPathRef,
    shapePathRef,
    values,
    width: shapeWidth,
    x: islandX,
  });

  return (
    // The frame only centres the island. Hover is read from the island's own
    // box, which carries its sideways shift round a notch, so the pointer is
    // on the island exactly where the island is seen.
    <div className="flex justify-center">
      <motion.div
        {...hover}
        className="w-fit"
        style={{ ...islandCssVariables, x: islandX }}
      >
        {/* The black is the notch's whatever the theme, so what it holds is
            always drawn dark, and in the island's dress. */}
        <div aria-hidden={!isAsked} className="dark relative text-fg">
          <SkinContext value="island">
            <IslandDrawing
              isShown={isShown}
              rowPathRef={rowPathRef}
              shapePathRef={shapePathRef}
            />
            {/* What it holds is clipped to the black (`useIslandOutline`),
                which is drawn from the row's start: in a right-to-left page
                the clip is mirrored as the black is, and what it holds is
                mirrored back. */}
            <motion.div
              className="relative rtl:-scale-x-100"
              ref={clipRef}
              style={{ opacity: contentOpacity }}
            >
              {/* One layout group, so a part that stays glides with the
                  black when a change elsewhere in the island moves it,
                  even where its own part did not change (`IslandParts`). */}
              <LayoutGroup>
                <IslandFollowContext value={follow}>
                  <div className="relative rtl:-scale-x-100">
                    <motion.div
                      animate={{
                        height: rowTarget.height,
                        opacity: isShown || notch ? 1 : 0,
                      }}
                      // Clipped rather than hidden, so it is no scroll box: a
                      // control focused while the row is still short, such as
                      // one pressed as the island comes out, cannot scroll the
                      // row up out of place.
                      className="relative overflow-clip"
                      initial={false}
                      onAnimationComplete={() => {
                        if (!isShown) onHidden?.();
                      }}
                      style={{
                        height: values.row.height,
                        opacity: values.row.opacity,
                        width: shapeWidth,
                      }}
                      transition={{
                        ...travel,
                        opacity: fade(isShown, notch !== null),
                      }}
                    >
                      <motion.div
                        className="absolute inset-x-0 top-0 flex justify-center"
                        ref={rowLayerRef}
                        style={{
                          x: rowX,
                          // Only once out: coming out, the row is laid out in its
                          // place and revealed as the black opens round it.
                          ...(fillsWidth && isShown && !isComing
                            ? { "--island-row-hold": "100%" }
                            : {}),
                        }}
                      >
                        {/* Round the presence, not in it: a row on its way out still
                  hears the island, and folds into the notch as it goes. */}
                        <IslandContext value={context}>
                          <IslandRowBox
                            fillsWidth={fillsWidth}
                            isShown={isShown}
                            label={label}
                            onElement={measured.setRowElement}
                          >
                            {row}
                          </IslandRowBox>
                        </IslandContext>
                      </motion.div>
                      {/* Kept while there is no line, so one going away folds
                      into the row with what it held rather than vanishing
                      from an island still its height. */}
                      <IslandBelow
                        islandX={islandX}
                        isShown={isStacked}
                        onElement={measured.setBelowElement}
                        top={measured.rowSize?.height ?? 0}
                        travel={travel}
                        width={measured.belowSize?.width}
                      >
                        {below}
                      </IslandBelow>
                    </motion.div>
                    <IslandAsideContext value={measured.setAside}>
                      <IslandSection
                        isFromMiddle={isFromMiddle}
                        isOpening={isOpening}
                        isShown={isShown}
                        layout={layout}
                        movesSideways={notch !== null && !isStacked}
                        onElement={measured.setSectionElement}
                        onOpened={finish}
                        rowWidth={shapeWidth}
                        sectionX={sectionX}
                        travel={travel}
                        values={values.section}
                      >
                        {section}
                      </IslandSection>
                    </IslandAsideContext>
                  </div>
                </IslandFollowContext>
              </LayoutGroup>
            </motion.div>
          </SkinContext>
        </div>
      </motion.div>
    </div>
  );
}
