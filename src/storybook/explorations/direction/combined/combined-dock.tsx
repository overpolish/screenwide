// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import "./combined.css";

import { Pause, Square, Trash2 } from "lucide-react";

import { TargetStill } from "../../recorder/shared/recorder-parts";

import { IconBtn } from "./combined-controls";

// The dock's content only: the window draws its own material and corners.
// What today's dock holds, in this look: the camera as it is being recorded,
// the input levels, the time and size so far, pause, stop and discard.

function Level({ value }: { value: number }) {
  return (
    <span className="relative h-5 w-1 overflow-hidden rounded-full bg-(--fill-2)">
      <span
        className="absolute inset-x-0 bottom-0 rounded-full bg-(--level)"
        style={{ height: `${String(value * 100)}%` }}
      />
    </span>
  );
}

export function CombinedDock() {
  return (
    <div className="look-combined flex h-10 w-max items-center gap-2 px-1.5 font-sans text-[13px] text-(--fg)">
      <TargetStill
        className="aspect-video h-7 overflow-hidden rounded-md"
        target="camera"
      />
      <span className="flex gap-0.5" dir="ltr">
        <Level value={0.55} />
        <Level value={0.3} />
      </span>
      <span className="flex items-baseline gap-1.5 tabular-nums">
        <span className="size-2 self-center rounded-full bg-(--record)" />
        00:42
        <span className="text-[11px] text-(--fg-3)">48 MB</span>
      </span>
      <IconBtn label="Pause">
        <Pause />
      </IconBtn>
      <span
        aria-label="Stop"
        className="combined-primary flex size-7 items-center justify-center rounded-lg"
      >
        <Square className="size-3 fill-current" />
      </span>
      <IconBtn label="Discard">
        <Trash2 />
      </IconBtn>
    </div>
  );
}
