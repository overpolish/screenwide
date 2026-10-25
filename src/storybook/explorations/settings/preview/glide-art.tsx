// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CSSProperties } from "react";

import { lerp, progress } from "./loop-timing";
import { ToolScreen } from "./tool-screen";
import { useClock } from "./use-clock";

// Glide's picture in Tools: two windows, one dragged into the left half.

function Pointer({ x, y }: { x: string; y: string }) {
  return (
    <svg
      className="absolute"
      height={16}
      style={{ insetInlineStart: x, top: y }}
      viewBox="0 0 16 20"
      width={13}
    >
      <path
        d="M1 1 L1 16 L5 12 L8 19 L11 18 L8 11 L14 11 Z"
        fill="#000"
        stroke="#fff"
        strokeWidth={1.2}
      />
    </svg>
  );
}

function AppWindow({ style }: { style: CSSProperties }) {
  return (
    <span
      className="absolute rounded-[5px] bg-white/95 shadow-md"
      style={style}
    >
      <span className="absolute start-1.5 top-1.5 flex gap-0.5">
        {["#ff5f57", "#febc2e", "#28c840"].map((color) => (
          <span
            className="size-1 rounded-full"
            key={color}
            style={{ background: color }}
          />
        ))}
      </span>
    </span>
  );
}

/** A window dragged to the left half: with Glide on it lands there, the
 * space around it as set, and the pointer comes with it if it follows; off,
 * it stays where it was dropped. */
export function GlideArt({
  cursorFollows,
  gap,
  isOn,
}: {
  cursorFollows: boolean;
  gap: number;
  isOn: boolean;
}) {
  const time = useClock();
  const at = isOn ? progress(time % 3.6, 0.8, 1.6) : 0;
  const box = (
    from: [number, number, number, number],
    gapCount: [number, number, number, number],
  ) => {
    const [start, top, width, height] = from;
    const to = [0, 0, 50, 100];
    const px = (index: number) => gapCount[index] * gap * at;
    return {
      height: `calc(${String(lerp(height, to[3], at))}% - ${String(px(3))}px)`,
      insetInlineStart: `calc(${String(lerp(start, to[0], at))}% + ${String(px(0))}px)`,
      top: `calc(${String(lerp(top, to[1], at))}% + ${String(px(1))}px)`,
      width: `calc(${String(lerp(width, to[2], at))}% - ${String(px(2))}px)`,
    };
  };
  const dragged = box([18, 22, 36, 62], [1, 1, 1.5, 2]);
  return (
    <ToolScreen isOn={isOn}>
      <AppWindow
        style={
          isOn
            ? {
                height: `calc(100% - ${String(gap * 2)}px)`,
                insetInlineStart: `calc(50% + ${String(gap / 2)}px)`,
                top: gap,
                width: `calc(50% - ${String(gap * 1.5)}px)`,
              }
            : {
                height: "58%",
                insetInlineStart: "52%",
                top: "30%",
                width: "34%",
              }
        }
      />
      <AppWindow style={dragged} />
      <Pointer
        x={
          cursorFollows
            ? `calc(${dragged.insetInlineStart} + 24px)`
            : "calc(18% + 24px)"
        }
        y={cursorFollows ? `calc(${dragged.top} + 2px)` : "calc(22% + 2px)"}
      />
    </ToolScreen>
  );
}
