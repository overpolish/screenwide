// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate } from "motion/react";
import { useMemo, useRef, useState } from "react";

import { cn } from "../../../lib/styling";
import { waveform } from "../shared/mock-data";

import { IdeaFrame } from "./idea-frame";
import { ACTIONS, clock, RECORDING_S, SEGMENTS } from "./ideas-data";

// Precision without zoom controls: time swells under the pointer the way the
// Dock does, so the moment being aimed at has room for its words and tenths
// of a second while the whole recording stays in view. Scrubbing sticks for
// an instant to word starts, clicks and pause edges.

const W = 880;
const SAMPLES = 600;
const MAX_MAGNIFY = 12;
const SPREAD = 0.035;
const DETENT_S = 0.12;

/** Every word with the moment it starts, spread over its sentence. */
const WORDS = SEGMENTS.flatMap((segment) => {
  if (segment.kind !== "speech") return [];
  const words = segment.text.split(" ");
  const step = (segment.end - segment.start) / words.length;
  return words.map((word, index) => ({
    at: segment.start + index * step,
    word,
  }));
});

const DETENTS = [
  ...WORDS.map(({ at }) => at),
  ...ACTIONS.map(({ at }) => at),
  ...SEGMENTS.flatMap(({ end, start }) => [start, end]),
];

const LEVELS = waveform(SAMPLES, 3);

/** Cumulative positions for time samples, given the lens' focus and strength. */
function mapping(focus: number, strength: number) {
  const density = Array.from({ length: SAMPLES + 1 }, (_, index) => {
    const u = index / SAMPLES;
    return (
      1 + MAX_MAGNIFY * strength * Math.exp(-(((u - focus) / SPREAD) ** 2))
    );
  });
  const cumulative = [0];
  for (let index = 1; index <= SAMPLES; index += 1) {
    cumulative.push(
      cumulative[index - 1] + (density[index - 1] + density[index]) / 2,
    );
  }
  const total = cumulative[SAMPLES];
  const positions = cumulative.map((value) => (value / total) * W);
  const toX = (seconds: number) => {
    const exact = (seconds / RECORDING_S) * SAMPLES;
    const low = Math.min(Math.floor(exact), SAMPLES - 1);
    return (
      positions[low] + (positions[low + 1] - positions[low]) * (exact - low)
    );
  };
  const toSeconds = (x: number) => {
    let low = 0;
    let high = SAMPLES;
    while (high - low > 1) {
      const middle = (low + high) >> 1;
      if (positions[middle] <= x) low = middle;
      else high = middle;
    }
    const part =
      (x - positions[low]) / Math.max(positions[high] - positions[low], 1e-6);
    return ((low + part) / SAMPLES) * RECORDING_S;
  };
  /** Pixels per second around a moment. */
  const scale = (seconds: number) =>
    (toX(seconds + 0.05) - toX(seconds - 0.05)) * 10;
  return { scale, toSeconds, toX };
}

export function FisheyeTimeline() {
  const [focus, setFocus] = useState(0.3);
  const [strength, setStrength] = useState(0);
  const [playhead, setPlayhead] = useState(12.4);
  const [detent, setDetent] = useState<number | null>(null);
  const scrubbingRef = useRef(false);
  const lens = useMemo(() => mapping(focus, strength), [focus, strength]);

  const swell = (to: number) => {
    animate(strength, to, {
      bounce: 0,
      duration: 0.35,
      onUpdate: setStrength,
      type: "spring",
    });
  };

  const scrubTo = (x: number) => {
    const seconds = lens.toSeconds(x);
    const near = DETENTS.find((at) => Math.abs(at - seconds) < DETENT_S);
    setDetent(near ?? null);
    setPlayhead(near ?? seconds);
  };

  return (
    <IdeaFrame
      hint="Move over the timeline: time swells under the pointer. Press and drag to scrub; the playhead sticks to words, clicks and pause edges."
      pitch="Precision without zoom controls. The timeline magnifies where you point, like the Dock, so the moment you aim at has room for its words and tenths of a second, and the whole recording never leaves the screen."
      title="Fisheye scrubbing"
    >
      <div
        className="flex flex-col gap-separate"
        dir="ltr"
        style={{ width: W }}
      >
        <div className="flex items-baseline justify-between">
          <span className="text-heading tabular-nums">{clock(playhead)}</span>
          <span className="text-caption text-fg-tertiary">
            {detent === null ? "Scrubbing freely" : "Held at a detent"}
          </span>
        </div>
        <div
          className="relative h-36 cursor-col-resize touch-none overflow-hidden rounded-panel bg-fill"
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            scrubbingRef.current = true;
            scrubTo(
              event.clientX - event.currentTarget.getBoundingClientRect().left,
            );
          }}
          onPointerEnter={() => {
            swell(1);
          }}
          onPointerLeave={() => {
            if (!scrubbingRef.current) swell(0);
          }}
          onPointerMove={(event) => {
            const x =
              event.clientX - event.currentTarget.getBoundingClientRect().left;
            // While scrubbing, the lens stays put under the pointer's time.
            if (!scrubbingRef.current)
              setFocus(Math.min(Math.max(x / W, 0), 1));
            else scrubTo(x);
          }}
          onPointerUp={() => {
            scrubbingRef.current = false;
            setDetent(null);
          }}
        >
          {/* The lens: a brighter band where time is magnified. */}
          <span
            className="pointer-events-none absolute inset-y-0 bg-fill-strong"
            style={{
              insetInlineStart: lens.toX(
                Math.max(focus * RECORDING_S - 2.4, 0),
              ),
              opacity: strength,
              width:
                lens.toX(Math.min(focus * RECORDING_S + 2.4, RECORDING_S)) -
                lens.toX(Math.max(focus * RECORDING_S - 2.4, 0)),
            }}
          />
          {/* Second and tenth ticks, shown only where there is room. */}
          {Array.from({ length: RECORDING_S * 10 }, (_, index) => {
            const seconds = index / 10;
            const spacing = lens.scale(seconds) / 10;
            const isSecond = index % 10 === 0;
            const opacity = isSecond
              ? 1
              : Math.min(Math.max((spacing - 3) / 6, 0), 1);
            if (opacity <= 0) return null;
            return (
              <span
                className={cn(
                  "absolute top-0 w-px bg-fg-tertiary",
                  isSecond ? "h-3" : "h-1.5",
                )}
                key={index}
                style={{ insetInlineStart: lens.toX(seconds), opacity }}
              />
            );
          })}
          {/* Words, each shown once there is room for it before the next. */}
          {WORDS.map(({ at, word }, index) => {
            const next =
              index + 1 < WORDS.length ? WORDS[index + 1].at : at + 0.6;
            const room = lens.toX(next) - lens.toX(at);
            const needed = word.length * 6.5 + 6;
            const opacity = Math.min(Math.max((room - needed) / 12, 0), 1);
            if (opacity <= 0) return null;
            return (
              <span
                className="absolute top-5 text-caption whitespace-nowrap text-fg-secondary"
                key={at}
                style={{ insetInlineStart: lens.toX(at), opacity }}
              >
                {word}
              </span>
            );
          })}
          {/* The sound, its detail opening up under the lens. */}
          <span className="absolute inset-x-0 bottom-6 h-14">
            {LEVELS.map((level, index) => {
              const seconds = (index / SAMPLES) * RECORDING_S;
              const x = lens.toX(seconds);
              const width = Math.max(
                lens.toX(((index + 1) / SAMPLES) * RECORDING_S) - x - 0.5,
                0.5,
              );
              const isSpeech = SEGMENTS.some(
                (segment) =>
                  segment.kind === "speech" &&
                  seconds >= segment.start &&
                  seconds < segment.end,
              );
              return (
                <span
                  className="absolute bottom-1/2 rounded-full bg-fg-secondary"
                  key={seconds}
                  style={{
                    height: `${String((isSpeech ? level : level * 0.12) * 100)}%`,
                    insetInlineStart: x,
                    transform: "translateY(50%)",
                    width,
                  }}
                />
              );
            })}
          </span>
          {/* What was done, along the bottom. */}
          {ACTIONS.map(({ at, kind, label }) => (
            <span
              className={cn(
                "absolute bottom-1.5 size-2 -translate-x-1/2 rounded-full",
                kind === "click" ? "bg-fg" : "bg-fg-tertiary",
              )}
              key={at}
              style={{ insetInlineStart: lens.toX(at) }}
              title={label}
            />
          ))}
          <span
            className="pointer-events-none absolute inset-y-0 w-0.5 -translate-x-1/2 bg-emphasis"
            style={{ insetInlineStart: lens.toX(playhead) }}
          >
            {detent === null ? null : (
              <span className="absolute inset-0 m-auto size-4 animate-ping rounded-full border border-emphasis" />
            )}
          </span>
        </div>
      </div>
    </IdeaFrame>
  );
}
