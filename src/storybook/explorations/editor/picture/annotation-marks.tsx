// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { isLight } from "../../../../composites/color-grid/palette";
import { corner, type Point } from "../../ideas/look-pens-model";
import { draw, pop } from "../../ideas/looks";
import { MockApp } from "../../ideas/mock-app";
import { H, px, W } from "../../ideas/point-geometry";
import { aimedAngle, type AnnotationStyle } from "../model/annotation-style";
import { type TimedMark } from "../model/marks";

// The annotations on the picture, in the one house style: sharp strokes
// lifted off the screen by a soft two-layer shadow, each in its own colour
// and sizes. Strokes draw on as a hand would and counters pop in, so a new
// annotation arrives rather than appears. A focus draws nothing round its
// box: it takes the rest away, dimmed or blurred.

/** The picture is the recording drawn small: px per point of a style. */
const PX_PER_POINT = 0.25;
/** Between a focused element and the edge of its box, in px. */
const FOCUS_PAD = 6;
/** The counter pin's outline, pointing at twelve o'clock, its disc centred
 * at 12,12 with a radius of 8 (`CounterPinIcon`'s shape). */
const PIN =
  "M4 12c0-4.993 5.539-10.193 7.399-11.799a1 1 0 0 1 1.202 0C14.461 1.807 20 7.007 20 12a8 8 0 0 1-16 0";
/** How far the pin's tip lies from its disc's middle, in disc radii. */
const PIN_REACH = 11.8 / 8;

function Arrow({
  from,
  style,
  to,
}: {
  from: Point;
  style: AnnotationStyle;
  to: Point;
}) {
  const angle = Math.atan2(to.y - from.y, to.x - from.x);
  const bend = {
    x: (from.x + to.x) / 2 - Math.sin(angle) * 26,
    y: (from.y + to.y) / 2 + Math.cos(angle) * 26,
  };
  const stroke = style.width * PX_PER_POINT;
  const back = 4 + stroke * 4;
  const spread = 3 + stroke * 2.6;
  // A head at a tip, opening back along the curve as it arrives there.
  const head = (tip: Point) => {
    const along = Math.atan2(tip.y - bend.y, tip.x - bend.x);
    const wing = (side: number) =>
      `${(tip.x - Math.cos(along) * back - Math.sin(along) * spread * side).toFixed(1)} ${(tip.y - Math.sin(along) * back + Math.cos(along) * spread * side).toFixed(1)}`;
    return `M${wing(1)} L${String(tip.x)} ${String(tip.y)} L${wing(-1)}`;
  };
  const tips =
    style.head === "both" ? [to, from] : style.head === "end" ? [to] : [];
  const path = `M${String(from.x)} ${String(from.y)} Q${String(bend.x)} ${String(bend.y)} ${String(to.x)} ${String(to.y)}`;
  return (
    <g filter="url(#house-lift)">
      {/* Keyed by layer, so a new colour repaints the arrow rather than
          drawing it on again. */}
      {[
        { color: "#fff", layer: "edge", width: stroke + 3.5 },
        { color: style.color, layer: "ink", width: stroke },
      ].map(({ color, layer, width }) => (
        <motion.path
          animate={{ pathLength: 1 }}
          d={path}
          fill="none"
          initial={{ pathLength: 0 }}
          key={layer}
          stroke={color}
          strokeLinecap="round"
          strokeWidth={width}
          transition={draw}
        />
      ))}
      {tips.map((tip) => (
        <path
          d={head(tip)}
          fill="none"
          key={`${String(tip.x)}-${String(tip.y)}`}
          stroke={style.color}
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={stroke}
        />
      ))}
    </g>
  );
}

function Counter({ mark, number }: { mark: TimedMark; number: number }) {
  const { style } = mark;
  const { at } = corner(mark.element);
  const radius = style.counterSize * PX_PER_POINT * 1.6;
  const degrees = style.counterAngle ?? aimedAngle(mark.element);
  const turn = (degrees * Math.PI) / 180;
  const centre = {
    x: at.x - Math.sin(turn) * radius * PIN_REACH,
    y: at.y + Math.cos(turn) * radius * PIN_REACH,
  };
  return (
    <motion.g
      animate={{ scale: 1 }}
      filter="url(#house-lift)"
      initial={{ scale: 0 }}
      style={{ transformBox: "fill-box", transformOrigin: "center" }}
      transition={pop}
    >
      <path
        d={PIN}
        fill={style.color}
        stroke="#fff"
        strokeWidth={12 / radius}
        transform={`translate(${String(centre.x)} ${String(centre.y)}) rotate(${String(degrees)}) scale(${String(radius / 8)}) translate(-12 -12)`}
      />
      <text
        dominantBaseline="central"
        fill={isLight(style.color) ? "#000" : "#fff"}
        fontSize={radius * 1.1}
        fontWeight={600}
        textAnchor="middle"
        x={centre.x}
        y={centre.y + 0.5}
      >
        {number}
      </text>
    </motion.g>
  );
}

/** A focus's box: its element padded, its corners as round as set. */
const focusBox = (mark: TimedMark) => {
  const rect = px(mark.element);
  const box = {
    h: rect.h + FOCUS_PAD * 2,
    w: rect.w + FOCUS_PAD * 2,
    x: rect.x - FOCUS_PAD,
    y: rect.y - FOCUS_PAD,
  };
  return {
    ...box,
    r: (mark.style.focusRadius / 100) * Math.min(box.w, box.h),
  };
};

/** The picture outside a focus's box, as a clip path with a hole in it. */
const outside = (mark: TimedMark) => {
  const { h, r, w, x, y } = focusBox(mark);
  const hole = [
    `M${String(x + r)} ${String(y)}`,
    `H${String(x + w - r)}`,
    `A${String(r)} ${String(r)} 0 0 1 ${String(x + w)} ${String(y + r)}`,
    `V${String(y + h - r)}`,
    `A${String(r)} ${String(r)} 0 0 1 ${String(x + w - r)} ${String(y + h)}`,
    `H${String(x + r)}`,
    `A${String(r)} ${String(r)} 0 0 1 ${String(x)} ${String(y + h - r)}`,
    `V${String(y + r)}`,
    `A${String(r)} ${String(r)} 0 0 1 ${String(x + r)} ${String(y)}`,
    "Z",
  ].join(" ");
  return `M0 0H${String(W)}V${String(H)}H0Z ${hole}`;
};

export function AnnotationMarks({
  draft,
  draftStyle,
  isSelecting,
  marks,
  onSelect,
}: {
  draft: { from: Point; to: Point } | null;
  /** The arrow pen's style, for an arrow being drawn. */
  draftStyle: AnnotationStyle;
  isSelecting: boolean;
  marks: TimedMark[];
  onSelect: (id: number) => void;
}) {
  const counters = marks.filter(({ pen }) => pen === "counter");
  const focuses = marks.filter(({ pen }) => pen === "focus");
  return (
    <>
      {focuses.map((mark) => (
        // The rest of the picture round the focus, dimmed, or blurred: a
        // softened copy of the screen laid over it, cut away at the focus,
        // as a backdrop filter would ignore the cut.
        <motion.div
          animate={{ opacity: 1 }}
          className="pointer-events-none absolute inset-0 overflow-hidden"
          initial={{ opacity: 0 }}
          key={mark.id}
          style={{
            background: mark.style.focusDims ? "rgba(0,0,0,0.42)" : undefined,
            clipPath: `path(evenodd, "${outside(mark)}")`,
          }}
          transition={{ duration: 0.3 }}
        >
          {mark.style.focusDims ? null : (
            <MockApp className="size-full blur-[4px]" />
          )}
        </motion.div>
      ))}
      <svg
        aria-hidden
        className="pointer-events-none absolute inset-0"
        height={H}
        width={W}
      >
        <defs>
          <filter height="200%" id="house-lift" width="200%" x="-50%" y="-50%">
            <feDropShadow
              dx={0}
              dy={1}
              floodOpacity={0.28}
              stdDeviation={0.8}
            />
            <feDropShadow dx={0} dy={4} floodOpacity={0.16} stdDeviation={5} />
          </filter>
        </defs>
        {marks.map((mark) => (
          <g
            className="cursor-pointer"
            key={mark.id}
            onPointerDown={(event) => {
              if (!isSelecting) return;
              event.stopPropagation();
              onSelect(mark.id);
            }}
            style={{ pointerEvents: isSelecting ? "auto" : "none" }}
          >
            {mark.pen === "focus" ? (
              // A focus is chosen by its box, not by the whole picture it
              // takes away.
              <rect
                fill="transparent"
                height={focusBox(mark).h}
                rx={focusBox(mark).r}
                width={focusBox(mark).w}
                x={focusBox(mark).x}
                y={focusBox(mark).y}
              />
            ) : mark.pen === "counter" ? (
              <Counter mark={mark} number={counters.indexOf(mark) + 1} />
            ) : mark.from && mark.to ? (
              <Arrow from={mark.from} style={mark.style} to={mark.to} />
            ) : null}
          </g>
        ))}
        {draft ? (
          <Arrow from={draft.from} style={draftStyle} to={draft.to} />
        ) : null}
      </svg>
    </>
  );
}
