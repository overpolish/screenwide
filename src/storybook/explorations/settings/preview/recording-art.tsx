// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SamplePicture } from "../../../../features/recording-controls/island/island-stage-picture";
import { cn } from "../../../../lib/styling";
import { MockApp } from "../../ideas/mock-app";
import { RecordedCursor } from "../../ideas/recorded-cursor";

import { useClock } from "./use-clock";

// The Recording pane's pictures for the settings that are not on the island:
// each plays what the recording will look like with the value set.

/** The same motion at each frame rate, the chosen one ringed: the ball
 * steps in thirtieths or sixtieths of a second, so the difference is seen
 * rather than described. */
export function FrameRateArt({ fps }: { fps: 30 | 60 }) {
  const time = useClock();
  return (
    <div
      className="absolute inset-0 flex flex-col justify-center gap-separate px-group"
      dir="ltr"
    >
      {([30, 60] as const).map((rate) => {
        const stepped = Math.floor(time * rate) / rate;
        const along = Math.sin(stepped * 2.6) * 0.5 + 0.5;
        return (
          <div
            className={cn(
              "relative h-10 rounded-control transition-[box-shadow,background-color]",
              fps === rate
                ? "bg-fill-strong ring-2 ring-primary"
                : "bg-fill opacity-60",
            )}
            key={rate}
          >
            <span className="absolute inset-y-0 start-3 flex items-center text-caption text-fg-secondary tabular-nums">
              {rate} fps
            </span>
            <span
              className="absolute top-1/2 size-4 -translate-y-1/2 rounded-full bg-fg"
              style={{
                insetInlineStart: `calc(56px + (100% - 76px) * ${String(along)})`,
              }}
            />
          </div>
        );
      })}
    </div>
  );
}

/** A recording of someone clicking Create: with zoom on, the picture
 * follows the pointer in and back out; off, it stays the whole screen. */
export function AutoZoomArt({ isOn }: { isOn: boolean }) {
  const time = useClock();
  const at = time % 4;
  const zoom = isOn
    ? 1 +
      0.9 *
        Math.max(
          0,
          Math.sin(Math.min(Math.max((at - 1.2) / 2.2, 0), 1) * Math.PI),
        )
    : 1;
  return (
    <div className="absolute inset-0 flex items-center justify-center">
      <div className="h-[90px] w-[144px] overflow-hidden rounded-control shadow-md">
        <div className="w-[480px] origin-top-left scale-[0.3]">
          <MockApp
            style={{
              transform: `scale(${String(zoom)})`,
              transformOrigin: "83.5% 65.5%",
            }}
          >
            <RecordedCursor seconds={24.9 + Math.min(at, 1.25)} />
          </MockApp>
        </div>
      </div>
    </div>
  );
}

/** The camera under 50 Hz lights: off, bands roll down the picture as they
 * do on a real camera; on, the picture is steady. */
export function AntiFlickerArt({ isOn }: { isOn: boolean }) {
  const time = useClock();
  return (
    <div className="absolute inset-0 flex items-center justify-center">
      <div className="relative h-[108px] w-[144px] overflow-hidden rounded-control shadow-md">
        <SamplePicture isFlipped={false} />
        {isOn ? null : (
          <span
            className="absolute inset-x-0 -top-full h-[300%]"
            style={{
              backgroundImage:
                "repeating-linear-gradient(to bottom, rgb(0 0 0 / 0) 0, rgb(0 0 0 / 0) 14px, rgb(0 0 0 / 0.28) 22px, rgb(0 0 0 / 0) 30px)",
              transform: `translateY(${String((time * 40) % 30)}px)`,
            }}
          />
        )}
      </div>
    </div>
  );
}
