// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";
import { pictureAt, type Recording } from "../recording";

// A game's picture at a moment, sketched: a view that swings as the player
// turns while playing, and the screen the game shows when play stops (a
// menu, a load, the wait after dying), named by the recording's picture
// reading. Content, so the same in every direction and theme.

export function GameFrame({
  children,
  className,
  recording,
  seconds,
}: {
  recording: Recording;
  seconds: number;
  children?: ReactNode;
  className?: string;
}) {
  const stopped = pictureAt(recording, seconds);
  const isPlaying = !stopped || stopped.state === "moving";
  // The view swings with time, so each moment of play looks its own.
  const turn = Math.sin(seconds * 0.9) * 30 + seconds * 23;
  const tilt = Math.sin(seconds * 1.7) * 4;
  return (
    <div
      className={cn(
        "relative aspect-[16/10] overflow-hidden bg-[#1b2333] text-[12px] select-none",
        className,
      )}
      dir="ltr"
    >
      <div
        className="absolute -inset-[20%]"
        style={{
          background:
            "linear-gradient(#8fb3d9 0 46%, #c9b48a 46% 52%, #6b5a3e 52% 100%)",
          transform: `rotate(${String(tilt)}deg)`,
        }}
      >
        {[0, 1, 2, 3, 4, 5].map((index) => {
          const x = (((index * 190 - turn * 6) % 1140) + 1140) % 1140;
          return (
            <span
              className="absolute bottom-[48%] w-[60px] rounded-t-[4px] bg-[#4a4f63]"
              key={index}
              style={{
                height: 70 + ((index * 37) % 90),
                insetInlineStart: x,
              }}
            />
          );
        })}
      </div>
      {isPlaying ? (
        <>
          <span className="absolute start-1/2 top-1/2 size-4 -translate-1/2 rounded-full border-2 border-white/90" />
          <span className="absolute start-[4%] bottom-[5%] h-2 w-[18%] rounded-full bg-black/40">
            <span className="block h-full w-[70%] rounded-full bg-[#5fd36b]" />
          </span>
          <span className="absolute end-[4%] bottom-[4%] text-[20px] font-bold text-white">
            24 / 90
          </span>
        </>
      ) : (
        <span className="absolute inset-0 flex items-center justify-center bg-[#0b0f18]/85 text-[26px] font-semibold text-white">
          {stopped.label ?? ""}
        </span>
      )}
      {children}
    </div>
  );
}
