// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, animate, motion } from "motion/react";
import {
  type ReactNode,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useLocale } from "react-aria";
import { Toolbar } from "react-aria-components";

import { motionDurations } from "../../lib/motion";

import { IslandBelow } from "./island-below";
import { IslandContext } from "./island-context";
import { IslandDrawing } from "./island-drawing";
import {
  ISLAND,
  type IslandAnchor,
  islandCssVariables,
  type IslandNotch,
} from "./island-geometry";
import { focusIn, glide, HOVER_LINGER_MS } from "./island-motion";
import { useOpening } from "./island-opening";
import {
  type IslandPart,
  useIslandOutline,
  useIslandShapeValues,
} from "./island-outline-sync";
import { useRowOffset } from "./island-row-offset";
import { IslandSection } from "./island-section";
import { useSize } from "./island-size";
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
// notch, or shrinks to a speck and fades where there is none.
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
  /** A second line under the row, in the row's black, centred under the
   * notch whatever the row does beside it; the section hangs under it
   * rather than under the row (`island-stack-layout.ts`). A row growing on
   * one side widens the island on that side alone, and nothing under it
   * moves. */
  below?: ReactNode;
  /** Whether it is out; hidden, it is tucked into the notch, or gone. */
  isShown?: boolean;
  /** The notch of the screen the island is on, measured; leave it out on a
   * screen without one. */
  notch?: IslandNotch | null;
  /** The island has tucked itself away after `isShown` turned false. */
  onHidden?: () => void;
  /** The pointer coming onto the island, which takes in its row and what
   * hangs from it, and leaving it, a moment after it goes, so a slip does
   * not close what hovering opened. */
  onHoverChange?: (isHovered: boolean) => void;
  /** Where the row and what hangs from it are, in the page, as they move:
   * for a window that lets clicks through everywhere but the island. */
  onShapeChange?: (parts: IslandPart[]) => void;
  /** What hangs beneath the row, if anything. */
  section?: ReactNode;
};

export function Island({
  anchor = "center",
  "aria-label": label,
  below,
  isShown = true,
  notch = null,
  onHidden,
  onHoverChange,
  onShapeChange,
  row,
  section,
}: IslandProps) {
  const [rowElement, setRowElement] = useState<HTMLDivElement | null>(null);
  const [sectionElement, setSectionElement] = useState<HTMLDivElement | null>(
    null,
  );
  const [belowElement, setBelowElement] = useState<HTMLDivElement | null>(null);
  const rowSize = useSize(rowElement);
  const sectionSize = useSize(sectionElement);
  const belowSize = useSize(belowElement);
  const [shift, setShift] = useState(0);

  const closeRef = useRef(0);
  useEffect(
    () => () => {
      window.clearTimeout(closeRef.current);
    },
    [],
  );

  // From the moment the island is shown or hidden until its row has settled.
  const [wasShown, setWasShown] = useState(isShown);
  const [isComing, setIsComing] = useState(false);
  if (wasShown !== isShown) {
    setWasShown(isShown);
    setIsComing(true);
  }
  const context = useMemo(
    () => ({ isShown, notch, setShift }),
    [isShown, notch],
  );

  const isStacked = isShown && Boolean(below);
  const rowShift = isShown && notch ? shift : 0;
  const { direction } = useLocale();
  const layout = layOutIsland({
    anchor,
    below: isStacked ? (belowSize ?? { height: 0, width: 0 }) : null,
    direction,
    hasSection: Boolean(section),
    isShown,
    notch,
    rowShift,
    rowSize,
    sectionSize,
  });
  const { flare, isOpen, radius, rowTarget, top, width } = layout;
  // A section opens out of the middle, and folds back into it, only as the
  // island comes out of the notch or goes back in.
  const isFromMiddle = !isShown || isComing;
  const { finish, isOpening } = useOpening(isOpen);

  const { islandX, rowLayerRef, rowX, sectionX, shapeWidth } = useRowOffset({
    hasNotch: notch !== null,
    holdsContent: !isShown,
    rowElement,
    rowTarget: rowShift,
    shiftTarget: layout.islandShift,
    width,
  });

  // The parts are laid out and animated as boxes that paint nothing; the
  // black is their outline, drawn from the same animated values.
  const values = useIslandShapeValues({ flare, radius, top });
  useLayoutEffect(() => {
    const corners = [
      animate(values.flare, flare, glide),
      animate(values.radius, radius, glide),
      animate(values.top, top, glide),
    ];
    return () => {
      corners.forEach((corner) => {
        corner.stop();
      });
    };
  }, [flare, radius, top, values]);
  const rowPathRef = useRef<SVGPathElement>(null);
  const shapePathRef = useRef<SVGPathElement>(null);
  useIslandOutline({
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
        className="w-fit"
        onPointerEnter={() => {
          window.clearTimeout(closeRef.current);
          onHoverChange?.(true);
        }}
        onPointerLeave={() => {
          closeRef.current = window.setTimeout(() => {
            onHoverChange?.(false);
          }, HOVER_LINGER_MS);
        }}
        style={{ ...islandCssVariables, x: islandX }}
      >
        {/* The black is the notch's whatever the theme, so what it holds is
            always drawn dark. */}
        <div aria-hidden={!isShown} className="dark relative text-fg">
          <IslandDrawing
            isShown={isShown}
            rowPathRef={rowPathRef}
            shapePathRef={shapePathRef}
          />
          <motion.div
            animate={{
              height: rowTarget.height,
              opacity: isShown || notch ? 1 : 0,
            }}
            className="relative overflow-hidden"
            initial={false}
            onAnimationComplete={() => {
              setIsComing(false);
              if (!isShown) onHidden?.();
            }}
            style={{
              height: values.row.height,
              opacity: values.row.opacity,
              width: shapeWidth,
            }}
            transition={{
              ...glide,
              // Hidden without a notch, it fades once it has shrunk.
              opacity: isShown
                ? { duration: motionDurations.state }
                : {
                    delay: motionDurations.travel,
                    duration: motionDurations.travel,
                  },
            }}
          >
            <motion.div
              className="absolute inset-x-0 top-0 flex justify-center"
              ref={rowLayerRef}
              style={{ x: rowX }}
            >
              {/* Round the presence, not in it: a row on its way out still
                  hears the island, and folds into the notch as it goes. */}
              <IslandContext value={context}>
                <AnimatePresence initial={false}>
                  {isShown ? (
                    <motion.div
                      key="row"
                      {...focusIn}
                      className="flex w-max"
                      initial={false}
                      ref={setRowElement}
                      style={{ padding: ISLAND.rowInset }}
                    >
                      <Toolbar aria-label={label} className="flex">
                        {row}
                      </Toolbar>
                    </motion.div>
                  ) : null}
                </AnimatePresence>
              </IslandContext>
            </motion.div>
            {below ? (
              <IslandBelow
                islandX={islandX}
                isShown={isStacked}
                onElement={setBelowElement}
                top={rowSize?.height ?? 0}
                width={belowSize?.width}
              >
                {below}
              </IslandBelow>
            ) : null}
          </motion.div>

          <IslandSection
            isFromMiddle={isFromMiddle}
            isOpening={isOpening}
            isShown={isShown}
            layout={layout}
            movesSideways={notch !== null && !isStacked}
            onElement={setSectionElement}
            onOpened={finish}
            sectionX={sectionX}
            values={values.section}
          >
            {section}
          </IslandSection>
        </div>
      </motion.div>
    </div>
  );
}
