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
import { Toolbar } from "react-aria-components";

import { motionDurations } from "../../lib/motion";
import { cn } from "../../lib/styling";

import { IslandContext } from "./island-context";
import {
  ISLAND,
  type IslandAnchor,
  islandCssVariables,
  type IslandNotch,
} from "./island-geometry";
import { islandLayout } from "./island-layout";
import {
  type Curve,
  focusIn,
  glide,
  HOVER_LINGER_MS,
  morph,
} from "./island-motion";
import { useOpening } from "./island-opening";
import { useIslandOutline, useIslandShapeValues } from "./island-outline-sync";
import { useRowOffset } from "./island-row-offset";
import { IslandSection } from "./island-section";
import { useSize } from "./island-size";

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
// outside it, so a list in the section keeps its own arrow keys. Controls are
// native buttons with native tooltips (`title`): the OS draws those outside
// the island's small window, where a tooltip of the page's own is clipped.

export type IslandProps = {
  "aria-label": string;
  /** The controls along the top, usually an `IslandRow`. */
  row: ReactNode;
  /** Where under the row the section hangs. */
  anchor?: IslandAnchor;
  /** Whether it is out; hidden, it is tucked into the notch, or gone. */
  isShown?: boolean;
  /** The notch of the screen the island is on, measured; leave it out on a
   * screen without one. */
  notch?: IslandNotch | null;
  /** The pointer coming onto the island, which takes in its row and what
   * hangs from it, and leaving it, a moment after it goes, so a slip does
   * not close what hovering opened. */
  onHoverChange?: (isHovered: boolean) => void;
  /** What hangs beneath the row, if anything. */
  section?: ReactNode;
};

export function Island({
  anchor = "center",
  "aria-label": label,
  isShown = true,
  notch = null,
  onHoverChange,
  row,
  section,
}: IslandProps) {
  const [rowElement, setRowElement] = useState<HTMLDivElement | null>(null);
  const [sectionElement, setSectionElement] = useState<HTMLDivElement | null>(
    null,
  );
  const rowSize = useSize(rowElement);
  const sectionSize = useSize(sectionElement);
  const [shift, setShift] = useState(0);
  const context = useMemo(() => ({ notch, setShift }), [notch]);

  const closeRef = useRef(0);
  useEffect(
    () => () => {
      window.clearTimeout(closeRef.current);
    },
    [],
  );

  // The spring is kept from the moment the island is shown or hidden until
  // the row has settled; everything after that glides.
  const [wasShown, setWasShown] = useState(isShown);
  const [isComing, setIsComing] = useState(false);
  if (wasShown !== isShown) {
    setWasShown(isShown);
    setIsComing(true);
  }
  const curve: Curve = isComing ? morph : glide;

  const layout = islandLayout({
    anchor,
    hasSection: Boolean(section),
    isShown,
    notch,
    rowSize,
    sectionSize,
  });
  const { flare, isOpen, radius, rowTarget, top, width } = layout;
  // A section opens out of the middle, and folds back into it, only as the
  // island comes out of the notch or goes back in.
  const isFromMiddle = !isShown || isComing;
  const { finish, isOpening } = useOpening(isOpen);

  const { islandX, rowLayerRef, rowX, sectionX, shapeWidth } = useRowOffset({
    curve,
    hasNotch: notch !== null,
    rowElement,
    shiftTarget: isShown && notch ? shift : 0,
    width,
  });

  // The parts are laid out and animated as boxes that paint nothing; the
  // black is their outline, drawn from the same animated values.
  const values = useIslandShapeValues({ flare, radius, top });
  useLayoutEffect(() => {
    const corners = [
      animate(values.flare, flare, curve),
      animate(values.radius, radius, curve),
      animate(values.top, top, curve),
    ];
    return () => {
      corners.forEach((corner) => {
        corner.stop();
      });
    };
  }, [curve, flare, radius, top, values]);
  const rowPathRef = useRef<SVGPathElement>(null);
  const shapePathRef = useRef<SVGPathElement>(null);
  useIslandOutline({ rowPathRef, shapePathRef, values, width: shapeWidth });

  return (
    // The frame only centres the island; hover is read from the island's
    // own box.
    <div className="flex justify-center">
      <div
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
        style={islandCssVariables}
      >
        {/* The black is the notch's whatever the theme, so what it holds is
            always drawn dark. */}
        <motion.div
          aria-hidden={!isShown}
          className="dark relative text-fg"
          style={{ x: islandX }}
        >
          {/* The outline, under everything the island holds. It carries
              the shadow itself, so what the island holds changing, as the
              recording's time does each second, does not redraw it. Drawn
              from the row's start, it is mirrored right to left. */}
          <svg
            aria-hidden
            className={cn(
              "pointer-events-none absolute start-0 top-0 h-px w-full overflow-visible fill-island rtl:-scale-x-100",
              isShown && "drop-shadow-xl",
            )}
          >
            <path ref={rowPathRef} />
            <path ref={shapePathRef} />
          </svg>
          <motion.div
            animate={{
              height: rowTarget.height,
              opacity: isShown || notch ? 1 : 0,
            }}
            className="relative overflow-hidden"
            initial={false}
            onAnimationComplete={() => {
              setIsComing(false);
            }}
            style={{
              height: values.row.height,
              opacity: values.row.opacity,
              width: shapeWidth,
            }}
            transition={{
              ...curve,
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
                    <IslandContext value={context}>
                      <Toolbar aria-label={label} className="flex">
                        {row}
                      </Toolbar>
                    </IslandContext>
                  </motion.div>
                ) : null}
              </AnimatePresence>
            </motion.div>
          </motion.div>

          <IslandSection
            curve={curve}
            isFromMiddle={isFromMiddle}
            isOpening={isOpening}
            isShown={isShown}
            layout={layout}
            onElement={setSectionElement}
            onOpened={finish}
            sectionX={sectionX}
            values={values.section}
          >
            {section}
          </IslandSection>
        </motion.div>
      </div>
    </div>
  );
}
