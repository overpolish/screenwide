// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AppWindow, Flag, Volume2, Undo2 } from "lucide-react";

import { cn } from "../../../../lib/styling";
import { ICONS } from "../../ideas/action-script-lines";
import { Waveform } from "../../shared/picture";
import { type Cut } from "../model/timeline-model";
import { Still } from "../picture/still";
import {
  type Beat,
  beatAt,
  beatTitle,
  type Cue,
  type Recording,
} from "../recording";

import { at, placed, reach, type StripScale } from "./strip-scale";

// What the strip is made of, besides its pauses (`pauses.tsx`). A beat is
// something that holds its time: done, flagged, loud or newly shown, drawn
// as the screen at that moment with what happened on it, so a recording with
// no voice is read by its pictures; or a line said, drawn as its words. A
// cut leaves a seam that puts it back.

const CUE_ICONS: Record<Cue, typeof Flag> = {
  moment: Flag,
  new: AppWindow,
  sound: Volume2,
};

/** The narrowest a beat carries its icon at, in px: the icon and padding. */
const ICON_PX = 18;
/** The least width, in px, a moment's picture is drawn at: wide enough to
 * fill the block's height, cropped at its sides where the block is
 * narrower. */
const MOMENT_STILL_PX = 72;

export function BeatBlock({
  beat,
  isHidden = false,
  isSelected,
  onPress,
  recording,
  scale,
  width,
}: {
  beat: Beat;
  isSelected: boolean;
  onPress: () => void;
  recording: Recording;
  scale: StripScale;
  width: number;
  /** Whether what it draws over the picture, such as a shortcut's keys, is
   * hidden: it is drawn as an outline, still there to press. */
  isHidden?: boolean;
}) {
  const { pad, px, style } = reach(scale, beat, width);
  if (px < 1) return null;
  const Icon =
    beat.kind === "action"
      ? ICONS[beat.action.kind]
      : beat.kind === "cue"
        ? CUE_ICONS[beat.cue]
        : null;
  // A flagged moment and a line said are drawn over the picture at that
  // moment, so what each is about can be seen along the strip.
  const isMoment = beat.kind === "cue" && beat.cue === "moment";
  const isPictured = isMoment || beat.kind === "speech";
  return (
    // The button reaches past a narrow block, so a beat a few pixels wide is
    // still easy to press; the block is drawn at its own width inside it.
    <button
      aria-label={beatTitle(beat)}
      className="group absolute inset-y-0"
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={style}
      title={beatTitle(beat)}
      type="button"
    >
      <span
        className={cn(
          "absolute inset-y-0 flex items-center gap-1.5 overflow-hidden rounded-[7px] text-start text-caption whitespace-nowrap transition-colors",
          // Padding would hold a narrow block wider than its time, over
          // its neighbours; too narrow for its icon, it is drawn bare.
          px >= ICON_PX && "px-1",
          isPictured
            ? cn(
                "bg-black text-white",
                // Chosen, it is ringed clear of its picture, which may be as
                // light as the primary.
                isSelected
                  ? "outline-2 outline-offset-2 outline-primary outline-solid"
                  : "ring-1 ring-fg-tertiary group-hover:ring-fg-secondary",
              )
            : isSelected
              ? "bg-primary text-on-primary"
              : isHidden
                ? "border border-dashed border-fg-tertiary text-fg-tertiary group-hover:bg-fill"
                : "bg-fill-strong text-fg-secondary group-hover:bg-fg-tertiary group-hover:text-fg",
        )}
        style={{ insetInlineEnd: pad, insetInlineStart: pad }}
      >
        {isPictured ? (
          <Still
            className="absolute start-1/2 top-1/2 -translate-1/2"
            recording={recording}
            seconds={beatAt(beat)}
            width={Math.max(px, MOMENT_STILL_PX)}
          />
        ) : null}
        {isMoment ? (
          px >= ICON_PX ? (
            <span className="relative mb-auto flex size-4 items-center justify-center rounded-[4px] bg-black/60">
              <Flag aria-hidden className="size-3" />
            </span>
          ) : null
        ) : beat.kind === "speech" ? (
          <>
            {/* Dimmed under the words and their wave, which read on it. */}
            <span className="absolute inset-0 bg-black/60" />
            <span className="absolute inset-x-1 bottom-1 flex h-3.5 opacity-60">
              <Waveform
                bars={Math.max(8, Math.round(px / 3))}
                seed={beat.start}
              />
            </span>
            <span className="relative mb-auto truncate pt-1">{beat.text}</span>
          </>
        ) : (
          <>
            {px > 64 ? (
              <Still
                className="rounded-[4px]"
                element={
                  beat.kind === "action" ? beat.action.element : undefined
                }
                recording={recording}
                seconds={beatAt(beat)}
                width={56}
              />
            ) : null}
            {Icon && px >= ICON_PX ? (
              <Icon aria-hidden className="size-3.5 shrink-0" />
            ) : null}
            {px > 170 ? (
              <span className="truncate">{beatTitle(beat)}</span>
            ) : null}
          </>
        )}
      </span>
    </button>
  );
}

/** Where a voice speaks, when there is one: a thin layer under the beats,
 * its lines the captions. Where a line has room, its words spread out
 * along it. Pressing a line opens the captions on it. Hidden, the captions
 * are drawn as outlines, still there to press and show again. */
export function VoiceRow({
  isHidden,
  isOpen,
  lines,
  onPress,
  recording,
  scale,
  width,
}: {
  /** Whether the captions are hidden from the picture. */
  isHidden: boolean;
  /** Whether the captions are open, which marks the lines as chosen. */
  isOpen: boolean;
  /** The lines to draw: those not drawn as beats. */
  lines: Recording["speech"];
  onPress: (line: Recording["speech"][number]) => void;
  recording: Recording;
  scale: StripScale;
  width: number;
}) {
  return (
    <div className="relative h-4" dir="ltr">
      {lines.map((line) => {
        const px = (scale.toX(line.end) - scale.toX(line.start)) * width;
        const words = recording.words.filter(
          ({ at: wordAt }) => wordAt >= line.start && wordAt < line.end,
        );
        const isSpread = px > line.text.length * 14;
        return (
          <button
            aria-label={line.text}
            aria-pressed={isOpen}
            className={cn(
              "absolute top-0 flex h-4 cursor-default items-center overflow-hidden rounded-full px-1.5 text-caption whitespace-nowrap transition-colors",
              isHidden
                ? cn(
                    "border border-dashed border-fg-tertiary text-fg-tertiary/60",
                    isOpen ? "bg-fill" : "hover:bg-fill",
                  )
                : isOpen
                  ? "bg-fill-strong text-fg-secondary"
                  : "bg-fill text-fg-tertiary hover:bg-fill-strong hover:text-fg-secondary",
            )}
            key={line.start}
            onClick={() => {
              onPress(line);
            }}
            onPointerDown={(event) => {
              event.stopPropagation();
            }}
            style={placed(scale, line.start, line.end)}
            type="button"
          >
            {isSpread ? (
              words.map(({ at: wordAt, word }) => (
                <span
                  className={cn("absolute", !isHidden && "text-fg-secondary")}
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
          </button>
        );
      })}
    </div>
  );
}

/** Cuts that touch or overlap, as runs: they close up at one point, so they
 * are one seam, which puts all of them back. */
function runsOf(cuts: Cut[]) {
  const runs: { end: number; ids: string[]; start: number }[] = [];
  const sorted = [...cuts].sort((first, second) => first.start - second.start);
  for (const cut of sorted) {
    const last = runs[runs.length - 1] as (typeof runs)[number] | undefined;
    if (last && cut.start <= last.end + 0.001) {
      last.end = Math.max(last.end, cut.end);
      last.ids.push(cut.id);
    } else runs.push({ end: cut.end, ids: [cut.id], start: cut.start });
  }
  return runs;
}

/** Where a cut closed up: press to put it back. */
export function Seams({
  cuts,
  onRestore,
  scale,
}: {
  cuts: Cut[];
  onRestore: (ids: string[]) => void;
  scale: StripScale;
}) {
  return runsOf(cuts).map((run) => (
    <button
      aria-label="Restore"
      className="absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 items-start justify-center"
      key={run.ids[0]}
      onClick={() => {
        onRestore(run.ids);
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={at(scale, run.start)}
      title="Put back"
      type="button"
    >
      <span className="absolute inset-y-0 w-px border-s border-dashed border-fg-secondary" />
      <span className="relative -mt-1 flex size-4 items-center justify-center rounded-full bg-fg text-recess">
        <Undo2 aria-hidden className="size-2.5" />
      </span>
    </button>
  ));
}
