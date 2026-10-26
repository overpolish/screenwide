// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";

import { type Project } from "./projects-model";

// A recording at a glance, in place of a length: its steps side by side, as
// long as they ran, with what was said as bars, what was done as ticks, and
// time where nothing happened as a squiggle, the way the editor's strip
// draws dead air. Where a search found something it glows in the orange,
// and the playhead is where the pointer or the player is. Time runs left to
// right in every language.

const pct = (seconds: number, duration: number) =>
  `${String((seconds / duration) * 100)}%`;

export function ActivityRail({
  at = null,
  className,
  hits = [],
  project,
  size = "small",
  step = null,
}: {
  project: Project;
  /** The playhead, in seconds; none while nothing is pointed at. */
  at?: number | null;
  className?: string;
  /** Where a search found something, in seconds. */
  hits?: number[];
  size?: "large" | "small";
  /** The step to lift above the others, as the one being shown. */
  step?: number | null;
}) {
  const { duration } = project;
  const isLarge = size === "large";
  return (
    <div
      aria-hidden
      className={cn(
        "relative flex w-full gap-[2px]",
        isLarge ? "h-7" : "h-2.5",
        className,
      )}
      dir="ltr"
    >
      {project.steps.map(({ end, start, title }, index) => (
        <div
          className={cn(
            "relative h-full overflow-hidden rounded-[3px] transition-colors",
            step === index ? "bg-fill-strong" : "bg-fill",
          )}
          key={title}
          style={{ flexBasis: 0, flexGrow: end - start }}
        >
          {project.lines
            .filter((line) => line.start < end && line.end > start)
            .map((line) => (
              <span
                className="absolute inset-y-[30%] rounded-full bg-fg-tertiary"
                key={line.start}
                style={{
                  insetInlineStart: pct(
                    Math.max(line.start, start) - start,
                    end - start,
                  ),
                  width: pct(
                    Math.min(line.end, end) - Math.max(line.start, start),
                    end - start,
                  ),
                }}
              />
            ))}
          {project.beats
            .filter((beat) => beat.at >= start && beat.at < end)
            .map((beat) =>
              beat.kind === "idle" ? (
                <svg
                  className="absolute inset-y-0 h-full text-fg-tertiary"
                  key={beat.at}
                  preserveAspectRatio="none"
                  style={{
                    insetInlineStart: pct(beat.at - start, end - start),
                    width: pct(
                      Math.min(beat.seconds ?? 0, end - beat.at),
                      end - start,
                    ),
                  }}
                  viewBox="0 0 40 10"
                >
                  <polyline
                    fill="none"
                    points="0,5 5,2 10,8 15,2 20,8 25,2 30,8 35,2 40,5"
                    stroke="currentColor"
                    strokeWidth="1.2"
                    vectorEffect="non-scaling-stroke"
                  />
                </svg>
              ) : (
                <span
                  className={cn(
                    "absolute inset-y-[15%] w-[2px] -translate-x-1/2 rounded-full bg-fg",
                    isLarge && "inset-y-[20%] w-[3px]",
                  )}
                  key={beat.at}
                  style={{
                    insetInlineStart: pct(beat.at - start, end - start),
                  }}
                />
              ),
            )}
        </div>
      ))}
      {hits.map((hit) => (
        <span
          className={cn(
            "absolute top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-emphasis shadow-[0_0_8px_var(--color-emphasis)]",
            isLarge ? "size-2.5" : "size-2",
          )}
          key={hit}
          style={{ insetInlineStart: pct(hit, duration) }}
        />
      ))}
      {at === null ? null : (
        <span
          className={cn(
            "absolute w-0.5 -translate-x-1/2 rounded-full bg-emphasis",
            isLarge ? "-inset-y-1" : "-inset-y-0.5",
          )}
          style={{ insetInlineStart: pct(at, duration) }}
        />
      )}
    </div>
  );
}
