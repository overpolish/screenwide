// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { cn } from "../../../../lib/styling";

import { ARROW_COLOR, ARROW_ENDS, SELECTION_BOX, waveform } from "./mock-data";

/** Stands in for the recorded screen: an app window on a styled background,
 * as the export would frame it. Content, so the same in every direction. */
export function FakeScreen({ className }: { className?: string }) {
  return (
    <div
      className={cn("relative overflow-hidden p-[7%]", className)}
      dir="ltr"
      style={{ background: "linear-gradient(135deg, #f6d365, #fda085)" }}
    >
      <div className="flex size-full flex-col overflow-hidden rounded-[8px] bg-white shadow-2xl">
        <div className="flex h-[7%] shrink-0 items-center gap-[1.2%] bg-[#f1f1f3] px-[2%]">
          {["#ff5f57", "#febc2e", "#28c840"].map((color) => (
            <span
              className="aspect-square h-[40%] rounded-full"
              key={color}
              style={{ background: color }}
            />
          ))}
        </div>
        <div className="flex min-h-0 grow">
          <div className="flex w-[22%] flex-col gap-[6%] bg-[#f7f7f9] p-[3%]">
            {[60, 80, 50, 70, 40].map((width) => (
              <span
                className="h-[4%] rounded-full bg-[#dcdce2]"
                key={width}
                style={{ width: `${String(width)}%` }}
              />
            ))}
          </div>
          <div className="flex grow flex-col gap-[5%] p-[4%]">
            <span className="h-[6%] w-[40%] rounded-full bg-[#1d1d1f]" />
            <div className="grid grow grid-cols-3 gap-[4%]">
              {["#e8f0ff", "#fff1e6", "#eafbf0"].map((color) => (
                <span
                  className="rounded-[6px]"
                  key={color}
                  style={{ background: color }}
                />
              ))}
            </div>
            <span className="h-[30%] rounded-[6px] bg-[#f4f4f6]" />
          </div>
        </div>
      </div>
    </div>
  );
}

/** An arrow at the shape and colour an annotation preview or preset uses. */
export function ArrowGlyph({
  className,
  color = ARROW_COLOR,
  head = "end",
  isHandDrawn,
  width = 3,
}: {
  className?: string;
  color?: string;
  head?: "both" | "end" | "none";
  isHandDrawn?: boolean;
  width?: number;
}) {
  const line = isHandDrawn ? "M6 26 C 12 22, 18 15, 24 13" : "M6 26 L 24 8";
  return (
    <svg
      aria-hidden
      className={className}
      fill="none"
      stroke={color}
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={width}
      viewBox="0 0 32 32"
    >
      <path d={line} />
      {head === "none" ? null : <path d="M14 8 L 25 7 L 24 18" />}
      {head === "both" ? <path d="M7 15 L 6 26 L 17 25" /> : null}
    </svg>
  );
}

/** The arrows on the recording: the selected one first; the second joins it
 * when a story shows two selected. Paths are in each box's 142 × 100 space. */
const ARROWS = [
  {
    box: SELECTION_BOX,
    ends: ARROW_ENDS,
    head: "M104 16 L 128 14 L 116 35",
    line: "M10 90 L 128 14",
  },
  {
    box: { height: 24, left: 66, top: 16, width: 18 },
    ends: [
      { x: 90, y: 12 },
      { x: 10, y: 86 },
    ],
    head: "M38 84 L 14 86 L 26 65",
    line: "M128 12 L 14 86",
  },
];

/**
 * The recording with selected arrows on it. `selection` decorates each
 * arrow's bounds from its ends (handles, an outline, a beam); `toolbar`
 * floats above the first, as the contextual tools for the selection, and
 * `beside` opens next to it, for a tool panel that sits by what it edits.
 */
export function Scene({
  beside,
  className,
  count = 1,
  selection,
  toolbar,
}: {
  beside?: ReactNode;
  className?: string;
  count?: 1 | 2;
  selection?: (ends: { x: number; y: number }[]) => ReactNode;
  toolbar?: ReactNode;
}) {
  return (
    <div className={cn("relative aspect-video", className)} dir="ltr">
      <FakeScreen className="size-full" />
      {ARROWS.slice(0, count).map(({ box, ends, head, line }, index) => (
        <div
          className="absolute"
          key={line}
          style={{
            height: `${String(box.height)}%`,
            insetInlineStart: `${String(box.left)}%`,
            top: `${String(box.top)}%`,
            width: `${String(box.width)}%`,
          }}
        >
          <svg
            aria-hidden
            className="absolute inset-0 size-full overflow-visible"
            fill="none"
            preserveAspectRatio="none"
            stroke={ARROW_COLOR}
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={4}
            viewBox="0 0 142 100"
          >
            <path d={line} />
            <path d={head} />
          </svg>
          {selection?.(ends)}
          {toolbar && index === 0 ? (
            <div className="absolute -inset-x-40 bottom-full mb-3 flex justify-center">
              {toolbar}
            </div>
          ) : null}
          {beside && index === 0 ? (
            <div className="absolute start-full -top-16 z-10 ms-10">
              {beside}
            </div>
          ) : null}
        </div>
      ))}
    </div>
  );
}

/** Level bars along a lane; the bar colour is the element's text colour. */
export function Waveform({
  bars = 160,
  className,
  seed = 1,
}: {
  bars?: number;
  className?: string;
  seed?: number;
}) {
  return (
    <svg
      aria-hidden
      className={cn("size-full", className)}
      preserveAspectRatio="none"
      viewBox={`0 0 ${String(bars * 2)} 100`}
    >
      {waveform(bars, seed).map((level, index) => (
        <rect
          fill="currentColor"
          height={level * 80}
          // A fixed sample: the bars never reorder.
          // eslint-disable-next-line @eslint-react/no-array-index-key
          key={index}
          rx={0.6}
          width={1.2}
          x={index * 2}
          y={50 - level * 40}
        />
      ))}
    </svg>
  );
}
