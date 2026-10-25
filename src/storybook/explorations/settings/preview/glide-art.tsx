// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CSSProperties } from "react";

import { sampleAppIcon } from "../../../../features/glide/glide-sample-icon";
import { GlideIsland } from "../../../../features/glide/island/glide-island";
import { cell } from "../../../../features/glide/island/glide-island-script";
import { MAP_SHAPE } from "../../../../features/glide/island/glide-island-shapes";
import { type GlideIslandPhase } from "../../../../features/glide/island/glide-island-types";

import { lerp, progress } from "./loop-timing";
import { ToolScreen } from "./tool-screen";
import { useClock } from "./use-clock";

// Glide's picture in Tools: the pointer takes a window by its top bar, the
// real island comes out where the gesture began and opens onto the left
// half as the hand heads that way, then closes round its check as the
// window is sent there.

/** One pass, in seconds. */
const LOOP = 4.6;
/** The window before it is moved: start, top, width and height, in % of
 * the screen. */
const FROM = [18, 22, 36, 62] as const;
/** Where the pointer takes the window's top bar, in px from its corner. */
const GRIP = { x: 28, y: 6 };
/** When the gesture starts, and the pointer is pinned and hidden, and when
 * the pointer comes back: a beat into the island's fade, as the app returns
 * it 90 ms in. */
const PINNED = 0.8;
const RETURNED = 2.7 + 0.09;
/** The island's open size, which its centre is kept inside the screen by. */
const ISLAND = {
  height: MAP_SHAPE.height / 2 + 6,
  width: MAP_SHAPE.width / 2 + 6,
};

const leftHalf = cell({ gridCols: 2 });

function Pointer({ style }: { style: CSSProperties }) {
  return (
    <svg
      className="absolute"
      height={16}
      style={style}
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

/** Where the island is in the gesture at `at` seconds into a pass. */
function islandAt(at: number): {
  phase: GlideIslandPhase;
  pulse: number;
  region: typeof leftHalf | null;
} {
  if (at < 0.8 || at >= 2.7) return { phase: "idle", pulse: 0, region: null };
  if (at >= 2.2) return { phase: "committed", pulse: 1, region: null };
  return {
    phase: "gliding",
    // The hand rests on the left half a moment before it is sent.
    pulse: at >= 1.8 ? 1 : 0,
    region: at >= 1.2 ? leftHalf : null,
  };
}

/** A window taken by its top bar and sent to the left half: with Glide on,
 * the island shows where it is going and it lands there, the space around
 * it as set, the pointer coming with it if it follows; off, nothing comes
 * and the window stays where it is. */
export function GlideArt({
  cursorFollows,
  gap,
  isOn,
}: {
  cursorFollows: boolean;
  gap: number;
  isOn: boolean;
}) {
  const time = useClock(isOn ? "on" : "off");
  const at = isOn ? time % LOOP : 0;
  const sent = isOn ? progress(at, 2.2, 2.9) : 0;
  const reach = isOn ? progress(at, 0.1, 0.7) : 1;
  const hidden = isOn && at >= PINNED && at < RETURNED;
  const island = islandAt(at);

  const box = (
    gapCount: [number, number, number, number],
  ): Record<"height" | "insetInlineStart" | "top" | "width", string> => {
    const [start, top, width, height] = FROM;
    const to = [0, 0, 50, 100];
    const px = (index: number) => gapCount[index] * gap * sent;
    return {
      height: `calc(${String(lerp(height, to[3], sent))}% - ${String(px(3))}px)`,
      insetInlineStart: `calc(${String(lerp(start, to[0], sent))}% + ${String(px(0))}px)`,
      top: `calc(${String(lerp(top, to[1], sent))}% + ${String(px(1))}px)`,
      width: `calc(${String(lerp(width, to[2], sent))}% - ${String(px(2))}px)`,
    };
  };
  const dragged = box([1, 1, 1.5, 2]);

  // Where the gesture began: the grip on the window's top bar.
  const gripX = `calc(${String(FROM[0])}% + ${String(GRIP.x)}px)`;
  const gripY = `calc(${String(FROM[1])}% + ${String(GRIP.y)}px)`;
  // The pointer comes in from below and takes the grip. Once the gesture
  // starts it is pinned and hidden; it comes back on the moved window if it
  // follows, or where it was pinned.
  const pointer =
    reach < 1
      ? {
          x: `calc(${String(lerp(46, FROM[0], reach))}% + ${String(GRIP.x * reach)}px)`,
          y: `calc(${String(lerp(86, FROM[1], reach))}% + ${String(GRIP.y * reach)}px)`,
        }
      : cursorFollows && sent > 0
        ? {
            x: `calc(${dragged.insetInlineStart} + ${String(GRIP.x)}px)`,
            y: `calc(${dragged.top} + ${String(GRIP.y)}px)`,
          }
        : { x: gripX, y: gripY };

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
      {isOn ? (
        // Centred where the gesture began, and kept inside the screen as
        // the app keeps it inside the display.
        <div
          className="absolute -translate-1/2"
          style={{
            insetInlineStart: `clamp(${String(ISLAND.width)}px, ${gripX}, calc(100% - ${String(ISLAND.width)}px))`,
            top: `clamp(${String(ISLAND.height)}px, ${gripY}, calc(100% - ${String(ISLAND.height)}px))`,
          }}
        >
          <GlideIsland
            fit={null}
            iconSrc={sampleAppIcon}
            pending={null}
            phase={island.phase}
            pulse={island.pulse}
            region={island.region}
          />
        </div>
      ) : null}
      {/* The system draws the pointer over every window, the island too. */}
      {hidden ? null : (
        <Pointer style={{ insetInlineStart: pointer.x, top: pointer.y }} />
      )}
    </ToolScreen>
  );
}
