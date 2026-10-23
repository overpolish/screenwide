// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Hourglass, Undo2 } from "lucide-react";

import { cn } from "../../../../lib/styling";
import { describe, ICONS } from "../../ideas/action-script-lines";
import { Waveform } from "../../shared/picture";
import { type Beat, SPEECH, WORDS } from "../editor-data";
import { type Cut, type Piece } from "../model/timeline-model";
import { Still } from "../picture/still";

import { at, placed, reach, type StripScale } from "./strip-scale";

// What the strip is made of. A beat is something done, shown as the screen
// at that moment with what was done on it, so a recording with no voice is
// read by its pictures. Between beats is still time, a spring whose coils
// pack tighter as it is squeezed. A cut leaves a seam that puts it back.

/** Past this, still time reads as dead air and is flagged. */
const IDLE_S = 4;
const COILS = 6;

function Spring({ isIdle }: { isIdle: boolean }) {
  const points = Array.from({ length: COILS * 2 + 1 }, (_, index) => {
    const x = (index / (COILS * 2)) * 100;
    const y = index === 0 || index === COILS * 2 ? 50 : index % 2 ? 22 : 78;
    return `${String(x)},${String(y)}`;
  }).join(" ");
  return (
    <svg
      aria-hidden
      // A replaced element keeps its own width between two insets, so the
      // width is set outright for the coils to stretch with the pause.
      className={cn(
        "absolute start-0 top-1/2 h-1/2 w-full -translate-y-1/2 px-0.5",
        isIdle ? "text-fg-secondary" : "text-fg-tertiary",
      )}
      preserveAspectRatio="none"
      viewBox="0 0 100 100"
    >
      <polyline
        fill="none"
        points={points}
        stroke="currentColor"
        strokeWidth={1.25}
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}

/** Runs of still time between beats, as recorded spans, with how much of
 * each nobody spoke in. */
function stillRuns(pieces: Piece[]) {
  const runs: { end: number; quiet: number; start: number }[] = [];
  for (const piece of pieces) {
    if (piece.beat || piece.isCut) continue;
    const quiet = piece.isSpeech ? 0 : piece.end - piece.start;
    const last = runs[runs.length - 1] as (typeof runs)[number] | undefined;
    if (last && last.end === piece.start) {
      last.end = piece.end;
      last.quiet += quiet;
    } else runs.push({ end: piece.end, quiet, start: piece.start });
  }
  return runs;
}

export function Stills({
  pieces,
  scale,
  width,
}: {
  pieces: Piece[];
  scale: StripScale;
  width: number;
}) {
  return stillRuns(pieces).map((run) => {
    const isIdle = run.quiet > IDLE_S;
    const px = (scale.toX(run.end) - scale.toX(run.start)) * width;
    return (
      <div
        className={cn(
          "pointer-events-none absolute inset-y-1.5 rounded-[6px]",
          isIdle && "border border-dashed border-fg-tertiary",
        )}
        key={run.start}
        style={placed(scale, run.start, run.end)}
      >
        <Spring isIdle={isIdle} />
        {isIdle && px > 70 ? (
          <span className="absolute inset-x-0 -top-2 flex justify-center">
            <span className="flex items-center gap-0.5 rounded-full bg-recess px-1 text-caption text-fg-secondary">
              <Hourglass aria-hidden className="size-3" />
              {(run.end - run.start).toFixed(1)} s
            </span>
          </span>
        ) : null}
      </div>
    );
  });
}

export function BeatBlock({
  beat,
  isSelected,
  onPress,
  scale,
  width,
}: {
  beat: Beat;
  isSelected: boolean;
  onPress: () => void;
  scale: StripScale;
  width: number;
}) {
  const { pad, px, style } = reach(scale, beat, width);
  if (px < 1) return null;
  const Icon = beat.kind === "action" ? ICONS[beat.action.kind] : null;
  return (
    // The button reaches past a narrow block, so a beat a few pixels wide is
    // still easy to press; the block is drawn at its own width inside it.
    <button
      className="group absolute inset-y-0"
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={style}
      title={beat.kind === "action" ? describe(beat.action) : beat.text}
      type="button"
    >
      <span
        className={cn(
          "absolute inset-y-0 flex items-center gap-1.5 overflow-hidden rounded-[7px] px-1 text-start text-caption whitespace-nowrap transition-colors",
          isSelected
            ? "bg-primary text-on-primary"
            : "bg-fill-strong text-fg-secondary group-hover:bg-fg-tertiary group-hover:text-fg",
        )}
        style={{ insetInlineEnd: pad, insetInlineStart: pad }}
      >
        {beat.kind === "action" ? (
          <>
            {px > 64 ? (
              <Still
                className="rounded-[4px]"
                element={beat.action.element}
                width={56}
              />
            ) : null}
            {Icon ? <Icon aria-hidden className="size-3.5 shrink-0" /> : null}
            {px > 170 ? (
              <span className="truncate">{describe(beat.action)}</span>
            ) : null}
          </>
        ) : (
          <>
            <span className="absolute inset-x-1 bottom-1 flex h-3.5 opacity-40">
              <Waveform
                bars={Math.max(8, Math.round(px / 3))}
                seed={beat.start}
              />
            </span>
            <span className="relative mb-auto truncate pt-1">{beat.text}</span>
          </>
        )}
      </span>
    </button>
  );
}

/** Where a voice speaks, when there is one: a thin layer under the beats.
 * Where the lens gives a line room, its words spread out along it. */
export function VoiceRow({
  scale,
  width,
}: {
  scale: StripScale;
  width: number;
}) {
  return (
    <div className="relative h-4" dir="ltr">
      {SPEECH.map((line) => {
        const px = (scale.toX(line.end) - scale.toX(line.start)) * width;
        const words = WORDS.filter(
          ({ at: wordAt }) => wordAt >= line.start && wordAt < line.end,
        );
        const isSpread = px > line.text.length * 14;
        return (
          <span
            className="absolute top-0 flex h-4 items-center overflow-hidden rounded-full bg-fill px-1.5 text-caption whitespace-nowrap text-fg-tertiary"
            key={line.start}
            style={placed(scale, line.start, line.end)}
          >
            {isSpread ? (
              words.map(({ at: wordAt, word }) => (
                <span
                  className="absolute text-fg-secondary"
                  key={wordAt}
                  style={{
                    insetInlineStart: `${String(((scale.toX(wordAt) - scale.toX(line.start)) / (scale.toX(line.end) - scale.toX(line.start))) * 100)}%`,
                  }}
                >
                  {word}
                </span>
              ))
            ) : px > 90 ? (
              <span className="truncate">{line.text}</span>
            ) : null}
          </span>
        );
      })}
    </div>
  );
}

/** Where a cut closed up: press to put it back. */
export function Seams({
  cuts,
  onRestore,
  scale,
}: {
  cuts: Cut[];
  onRestore: (id: string) => void;
  scale: StripScale;
}) {
  return cuts.map((cut) => (
    <button
      aria-label="Restore"
      className="absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 items-start justify-center"
      key={cut.id}
      onClick={() => {
        onRestore(cut.id);
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={at(scale, cut.start)}
      title={`Restore ${(cut.end - cut.start).toFixed(1)} s`}
      type="button"
    >
      <span className="absolute inset-y-0 w-px border-s border-dashed border-fg-secondary" />
      <span className="relative -mt-1 flex size-4 items-center justify-center rounded-full bg-fg text-recess">
        <Undo2 aria-hidden className="size-2.5" />
      </span>
    </button>
  ));
}
