// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, type LucideIcon, Monitor } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";

import { type SceneDrawing } from "./scene-layout-drawing";

import type { SceneRotation } from "../../../bindings/SceneRotation";

/** The box a small drawing fits inside, in pixels. */
const SMALL = { height: 14, width: 22 };

type Box = { height: number; width: number; x: number; y: number };

/** The glyph's size and stroke, from its pane, which is a size container:
 * the small icon size until the pane is too small to hold it with room
 * round it, then shrinking with the pane, in whole pixels so it centres
 * evenly. Strokes are drawn in screen pixels at every size, so it thins in
 * step: the usual 1.5px at the small icon size, held to at least half that.
 * Set on the element, so the icon size a row title gives the glyphs in it
 * does not reach it. */
const glyphStyle = {
  height: "round(down, min(var(--spacing-icon-sm), 60cqmin), 1px)",
  strokeWidth: "clamp(0.75px, calc(60cqmin * 3 / 28), 1.5px)",
  width: "round(down, min(var(--spacing-icon-sm), 60cqmin), 1px)",
};

/** Below 6px a glyph no longer reads as one, so a pane too small to hold
 * that, at 60% of its shorter side, is drawn plain. */
const GLYPH_ROOM =
  "[@container(max-width:11.5px)]:hidden [@container(max-height:11.5px)]:hidden";

const percent = (share: number) => `${String(share * 100)}%`;

/** The turn the picture gives the screen, as the drawing's CSS: about the
 * pane's middle, seen from in front of it at twice the video's longer side,
 * as `turned_homography` in `rotation.rs` draws it. That turn's Y runs up,
 * so its X and Z turns run the other way in CSS, whose Y runs down. The
 * video's longer side is `100cqmax` of the drawing, its size container. */
const turnStyle = ({ x, y, z }: SceneRotation) => ({
  transform: `perspective(200cqmax) rotateZ(${String(-z)}deg) rotateY(${String(y)}deg) rotateX(${String(-x)}deg)`,
});

/** A scene's layout at the video's own shape: each pane a translucent fill,
 * marked with the glyph the recording controls give its source where it has
 * room for one, so the sides read at a glance, and laid over one another so
 * an overlap shows without an outline, the screen turned in depth as the
 * scene turns it. Small enough for a row's glyph or a scene on the
 * timeline, in a faint frame like the island's angle dial's ring, its panes
 * stronger to read at that size; or filling what holds it, as a tile's
 * picture, whose own fill is the frame. Either way the video's edge clips
 * what a turn carries past it, as the picture's does. When the layout
 * changes, each pane glides to its new place and size, and one the layout
 * adds or hides fades in or out where it sits. */
export function SceneLayoutGlyph({
  drawing,
  isFilling = false,
}: {
  drawing: SceneDrawing;
  isFilling?: boolean;
}) {
  const reducedMotion = useReducedMotion() ?? false;
  const transition = {
    duration: reducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };
  const { aspect, camera, rotation, screen } = drawing;
  const small =
    aspect >= SMALL.width / SMALL.height
      ? { height: SMALL.width / aspect, width: SMALL.width }
      : { height: SMALL.height, width: SMALL.height * aspect };
  const fill = isFilling ? "bg-current/20" : "bg-current/50";
  return (
    // Laid out left to right, as the picture it draws is; a size container,
    // so a turn is seen from as far off as the video is large.
    <span
      aria-hidden
      className="relative block shrink-0 [container-type:size]"
      dir="ltr"
      style={isFilling ? { height: "100%", width: "100%" } : small}
    >
      {isFilling ? null : (
        <span
          className="absolute inset-0 border border-fill-strong"
          style={{ borderRadius: small.height * 0.2 }}
        />
      )}
      <span
        className="absolute inset-0 overflow-hidden"
        style={isFilling ? undefined : { borderRadius: small.height * 0.2 }}
      >
        <AnimatePresence initial={false}>
          {screen ? (
            <Pane
              box={screen}
              fill={fill}
              glyph={Monitor}
              key="screen"
              roundness={0.15}
              transition={transition}
              turn={rotation}
            />
          ) : null}
          {camera ? (
            <Pane
              box={camera}
              fill={fill}
              glyph={Camera}
              key="camera"
              roundness={camera.radius / 100}
              transition={transition}
            />
          ) : null}
        </AnimatePresence>
      </span>
    </span>
  );
}

/** One pane, placed as a share of the drawing: a fill rounded as the pane
 * is, its glyph in the middle, turned in depth by `turn` where it has one.
 * The fill sits in a size container, so its corners and glyph follow the
 * pane's own size as it glides. */
function Pane({
  box,
  fill,
  glyph: Glyph,
  roundness,
  transition,
  turn,
}: {
  box: Box;
  /** The fill's class. */
  fill: string;
  glyph: LucideIcon;
  /** The corners, as a share of the pane's shorter side. */
  roundness: number;
  transition: { duration: number; ease: typeof motionEasings.out };
  turn?: SceneRotation;
}) {
  const place = {
    height: percent(box.height),
    left: percent(box.x),
    top: percent(box.y),
    width: percent(box.width),
  };
  return (
    <motion.span
      animate={{ ...place, opacity: 1 }}
      className="absolute"
      exit={{ opacity: 0 }}
      initial={{ ...place, opacity: 0 }}
      transition={transition}
    >
      {/* The placed span is no size container, so the turn's units are the
       * drawing's; the turned span is one, for the fill's corners and glyph. */}
      <span
        className="absolute inset-0 [container-type:size]"
        style={turn ? turnStyle(turn) : undefined}
      >
        <span
          className={cn(
            "absolute inset-0 flex items-center justify-center",
            fill,
          )}
          style={{
            borderRadius: `calc(min(100cqw, 100cqh) * ${String(roundness)})`,
          }}
        >
          <Glyph aria-hidden className={GLYPH_ROOM} style={glyphStyle} />
        </span>
      </span>
    </motion.span>
  );
}
