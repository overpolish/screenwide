// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown, ChevronsLeftRight, Folder, User } from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { Scene, Waveform } from "../direction/shared/mock-parts";

import { FILE_NAME, type QualityStep } from "./export-data";

/** A setting as a chip: on or off at a press, or opening its menu. */
export function Chip({
  children,
  hasMenu = false,
  icon: Icon,
  isDisabled = false,
  isOn = false,
}: {
  children: ReactNode;
  icon: typeof Folder;
  hasMenu?: boolean;
  isDisabled?: boolean;
  isOn?: boolean;
}) {
  return (
    <span
      aria-disabled={isDisabled || undefined}
      className={cn(
        "flex h-7 items-center gap-1.5 rounded-full px-2.5 text-[12px] whitespace-nowrap",
        isOn ? "combined-accent" : "bg-(--fill) text-(--fg-2)",
        isDisabled && "text-(--fg-3) opacity-60",
      )}
    >
      <Icon className="size-3.5 shrink-0" />
      {children}
      {hasMenu ? <ChevronDown className="-me-1 size-3 opacity-70" /> : null}
    </span>
  );
}

/** A labelled line of chips; the label says when they apply. */
export function ChipLine({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="w-[76px] shrink-0 text-[11px] text-(--fg-3)">
        {label}
      </span>
      {children}
    </div>
  );
}

/** The quality ladder: each step shows what it makes, so the choice is by
 * result rather than by setting. */
export function QualitySteps({
  selected,
  steps,
}: {
  selected: number;
  steps: QualityStep[];
}) {
  return (
    <div className="grid grid-cols-4 gap-0.5 rounded-xl bg-(--fill) p-0.5">
      {steps.map(({ dimensions, label, size }, index) => (
        <span
          className={cn(
            "flex flex-col items-center rounded-[10px] py-1.5 text-(--fg-2)",
            index === selected && "combined-raised text-(--fg)",
          )}
          key={label}
        >
          <span className="text-[12px] font-medium">{label}</span>
          <span className="text-[10px] text-(--fg-3) tabular-nums">
            {dimensions}
          </span>
          <span
            className={cn(
              "text-[11px] tabular-nums",
              index === selected ? "text-(--fg)" : "text-(--fg-2)",
            )}
          >
            {size}
          </span>
        </span>
      ))}
    </div>
  );
}

/** The name to save under, editable in place, with its extension, which
 * is the format. */
export function FileTitle({ extension }: { extension: string }) {
  return (
    <span className="-mx-1 flex min-w-0 items-baseline rounded-md px-1 text-[15px] font-semibold hover:bg-(--fill)">
      <span className="truncate" dir="auto">
        {FILE_NAME}
      </span>
      <span className="shrink-0 font-normal text-(--fg-3)">.{extension}</span>
    </span>
  );
}

/** Where it goes and how big it comes out, on one line. */
export function Destination({ size }: { size?: string }) {
  return (
    <span className="flex items-center gap-1.5 text-[12px] text-(--fg-2)">
      <span className="-ms-1 flex items-center gap-1 rounded-md px-1 py-0.5 hover:bg-(--fill)">
        <Folder className="size-3.5" />
        <span dir="auto">Desktop</span>
        <ChevronDown className="size-3 opacity-70" />
      </span>
      {size ? (
        <>
          <span className="text-(--fg-3)">·</span>
          <span className="tabular-nums">{size}</span>
        </>
      ) : null}
    </span>
  );
}

/** The frame under the playhead, the original against the export, split
 * where the divider sits. */
export function ComparePreview({
  isComparing = true,
}: {
  isComparing?: boolean;
}) {
  return (
    <div
      className="relative overflow-hidden rounded-lg shadow-[0_0_0_0.5px_var(--divider)]"
      dir="ltr"
    >
      <Scene className="w-full" />
      {isComparing ? (
        <>
          {/* The export's side, a touch softer, as compression leaves it. */}
          <span className="absolute inset-y-0 end-0 w-1/2 backdrop-blur-[0.8px]" />
          <span className="absolute inset-y-0 start-1/2 w-0.5 -translate-x-1/2 bg-white/90" />
          <span className="combined-knob absolute start-1/2 top-1/2 flex size-6 -translate-1/2 items-center justify-center rounded-full text-[#303030]">
            <ChevronsLeftRight className="size-3.5" />
          </span>
          <PreviewTag className="start-2">Original</PreviewTag>
          <PreviewTag className="end-2">Export</PreviewTag>
        </>
      ) : null}
    </div>
  );
}

function PreviewTag({
  children,
  className,
}: {
  children: ReactNode;
  className: string;
}) {
  return (
    <span
      className={cn(
        "absolute top-2 rounded-md bg-black/50 px-1.5 py-0.5 text-[10px] font-medium text-white",
        className,
      )}
    >
      {children}
    </span>
  );
}

/** The camera's own frame, for a camera saved as its own file. */
export function CameraFrame() {
  return (
    <span className="flex aspect-video items-end justify-center overflow-hidden rounded-lg bg-linear-to-b from-[#5b6b7d] to-[#2f3a47] shadow-[0_0_0_0.5px_var(--divider)]">
      <User className="size-[70%] translate-y-[18%] stroke-1 text-white/70" />
    </span>
  );
}

/** A recording with no picture: its two voices. */
export function SoundPreview() {
  return (
    <div className="flex h-28 flex-col gap-1 rounded-xl bg-(--recess) p-3">
      <span className="flex min-h-0 grow text-(--fg-3)">
        <Waveform bars={110} seed={1} />
      </span>
      <span className="flex min-h-0 grow text-(--level)">
        <Waveform bars={110} seed={4} />
      </span>
    </div>
  );
}
