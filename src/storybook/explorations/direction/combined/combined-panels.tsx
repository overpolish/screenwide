// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CopyPlus,
  Mic,
  Monitor,
  Play,
  Trash2,
  Video,
  Volume2,
} from "lucide-react";

import { cn } from "../../../../lib/styling";
import { ANNOTATION_COLORS, ARROW_COLOR, PRESETS } from "../shared/mock-data";
import { ArrowGlyph, Waveform } from "../shared/mock-parts";

import { Btn, IconBtn, Row, Segmented, Toggle } from "./combined-controls";
import { ANNOTATIONS, at, clock, MOMENTS, span } from "./combined-data";

const ARROW = ANNOTATIONS.find(({ isSelected }) => isSelected);

/**
 * The tool panel for the selected arrow: a window of its own, hung from the
 * workspace's corner whenever something is selected, so it is always in the
 * same place. Its head says what it edits and when; the strip is the whole
 * recording, and dragging its ends changes when the arrow shows. Then the
 * house row order: styles, size, options, animation, colour, actions.
 */
export function ArrowPanel() {
  return (
    <div className="combined-card flex w-[272px] flex-col gap-3 rounded-xl p-3 text-(--fg)">
      <div className="flex items-center gap-1">
        <span className="text-[13px] font-semibold">Arrow 1</span>
        <span className="ms-1.5 text-[11px] text-(--fg-3) tabular-nums">
          {ARROW ? `${clock(ARROW.start)} to ${clock(ARROW.end)}` : null}
        </span>
        <span className="ms-auto flex">
          <IconBtn label="Duplicate">
            <CopyPlus />
          </IconBtn>
          <IconBtn label="Delete">
            <Trash2 />
          </IconBtn>
        </span>
      </div>
      {ARROW ? (
        <span className="relative h-1.5 rounded-full bg-(--fill-2)" dir="ltr">
          <span
            className="absolute inset-y-0 rounded-full bg-(--accent)"
            style={span(ARROW.start, ARROW.end)}
          />
          <span
            className="combined-knob absolute top-1/2 h-3 w-1.5 -translate-1/2 rounded-full"
            style={{ insetInlineStart: at(ARROW.start) }}
          />
          <span
            className="combined-knob absolute top-1/2 h-3 w-1.5 -translate-1/2 rounded-full"
            style={{ insetInlineStart: at(ARROW.end) }}
          />
        </span>
      ) : null}
      <div className="grid grid-cols-4 gap-1.5">
        {PRESETS.map(({ color, isHandDrawn, name, width }, index) => (
          <span
            className={cn(
              "flex flex-col items-center gap-1 rounded-lg p-1 text-[10px] text-(--fg-2)",
              index === 0 && "combined-selected text-(--fg)",
            )}
            key={name}
          >
            <span className="flex size-10 items-center justify-center rounded-lg bg-[#1c1c1e]">
              <ArrowGlyph
                className="size-7"
                color={color}
                isHandDrawn={isHandDrawn}
                width={width}
              />
            </span>
            {name}
          </span>
        ))}
      </div>
      <div className="flex flex-col gap-1">
        <Row isChanged label="Width">
          <span
            className="relative h-1 w-28 rounded-full bg-(--fill-2)"
            dir="ltr"
          >
            <span className="absolute inset-y-0 start-0 w-[45%] rounded-full bg-(--accent)" />
            <span className="combined-knob absolute top-1/2 start-[45%] size-3.5 -translate-1/2 rounded-full" />
          </span>
        </Row>
        <Row label="Head">
          <Segmented
            className="w-36"
            items={["None", "End", "Both"]}
            selected={1}
          />
        </Row>
        <Row label="Hand-drawn">
          <Toggle isOn={false} />
        </Row>
        <Row label="Animate">
          <Toggle isOn />
        </Row>
      </div>
      <div className="flex justify-between">
        {ANNOTATION_COLORS.map((color) => (
          <span
            className={cn(
              "size-[22px] rounded-md shadow-[inset_0_0_0_1px_rgb(0_0_0/12%)]",
              color === ARROW_COLOR &&
                "ring-2 ring-(--accent) ring-offset-2 ring-offset-(--surface)",
            )}
            key={color}
            style={{ background: color }}
          />
        ))}
      </div>
      <div className="flex justify-end">
        <Btn>Reverse</Btn>
      </div>
    </div>
  );
}

const TRACKS = [
  { icon: Monitor, isIncluded: true, name: "Screen" },
  { icon: Video, isIncluded: false, name: "Camera" },
  { icon: Volume2, isIncluded: true, name: "System audio" },
  { icon: Mic, isIncluded: true, name: "Microphone" },
];

/** Which tracks the export carries: the switches the lanes no longer hold.
 * A native menu in the app; drawn here to show what it holds. */
export function TracksMenu() {
  return (
    <div className="combined-card flex w-56 flex-col gap-0.5 rounded-xl p-1.5 text-(--fg)">
      <span className="px-1.5 pt-0.5 pb-1 text-[11px] font-semibold text-(--fg-3)">
        In the export
      </span>
      {TRACKS.map(({ icon: Icon, isIncluded, name }) => (
        <span
          className="flex h-7 items-center gap-2 rounded-md px-1.5 text-[12px]"
          key={name}
        >
          <Icon className={cn("size-3.5", !isIncluded && "text-(--fg-3)")} />
          <span className={cn("grow", !isIncluded && "text-(--fg-3)")}>
            {name}
          </span>
          <Toggle isOn={isIncluded} />
        </span>
      ))}
    </div>
  );
}

const NOTE = MOMENTS.find(({ note }) => note);

/** A moment's voice note, opened from its pin on the ruler. */
export function MomentNote() {
  if (!NOTE?.note) return null;
  return (
    <div className="combined-card flex w-64 flex-col gap-2 rounded-xl p-3 text-(--fg)">
      <span className="flex items-center gap-1.5 text-[12px]">
        <span
          className="size-2 rounded-full"
          style={{ background: NOTE.color }}
        />
        <span className="font-semibold">{NOTE.name}</span>
        <span className="text-(--fg-3) tabular-nums">{clock(NOTE.at)}</span>
      </span>
      <span className="flex items-center gap-2">
        <span className="combined-primary flex size-6 shrink-0 items-center justify-center rounded-full">
          <Play className="ms-px size-3 fill-current" />
        </span>
        <span className="flex h-5 grow text-(--fg-3)">
          <Waveform bars={48} seed={7} />
        </span>
        <span className="text-[11px] text-(--fg-3) tabular-nums">
          {NOTE.note.duration}
        </span>
      </span>
      <p className="text-[12px] leading-4 text-(--fg-2)">
        {NOTE.note.transcript}
      </p>
    </div>
  );
}
