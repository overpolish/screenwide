// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, useLayoutEffect, useRef, useState } from "react";

import { cn } from "../../../lib/styling";
import { MockApp } from "../ideas/mock-app";
import { H, W } from "../ideas/point-geometry";

import { place, type Box, type Project } from "./projects-model";
import { Audio, Browser, Camera, Code, Design, Slides } from "./screen-looks";

// A project's recorded screen at one of its steps, drawn at its full size
// and scaled to fit, so type and corners shrink with it. What the step is
// about is outlined in the orange with the pointer on it, as the editor's
// Steps board outlines it. Content, so left to right in every language.

const LOOKS: Record<
  Project["look"],
  (props: { project: Project; step: number }) => ReactNode
> = {
  audio: Audio,
  browser: Browser,
  camera: Camera,
  code: Code,
  design: Design,
  slides: Slides,
  walkthrough: () => <MockApp className="absolute inset-0 size-full" />,
};

/** The pointer, on what the step is about. */
function Pointer({ focus }: { focus: Box }) {
  return (
    <svg
      aria-hidden
      className="absolute size-[22px] drop-shadow-[0_1px_1px_rgb(0_0_0/0.4)]"
      style={place({
        h: 5,
        w: 3,
        x: focus.x + focus.w * 0.6,
        y: focus.y + focus.h * 0.55,
      })}
      viewBox="0 0 24 24"
    >
      <path
        d="M3 2 L3 19 L8 14 L12 22 L15 20 L11 13 L18 13 Z"
        fill="#111"
        stroke="#fff"
        strokeWidth="1.5"
      />
    </svg>
  );
}

export function ProjectScreen({
  className,
  project,
  showsFocus = true,
  step,
}: {
  project: Project;
  step: number;
  className?: string;
  /** Outlines what the step is about; off for a still at rest. */
  showsFocus?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setWidth(entry.contentRect.width);
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  const focus = project.steps[step]?.focus;
  const Look = LOOKS[project.look];
  return (
    <div
      className={cn("relative aspect-[16/10] overflow-hidden", className)}
      dir="ltr"
      ref={ref}
    >
      <div
        className="absolute start-0 top-0 origin-top-left text-[12px] select-none"
        style={{
          height: H,
          transform: `scale(${String(width / W)})`,
          width: W,
        }}
      >
        <Look project={project} step={step} />
        {showsFocus && focus ? (
          <>
            <span
              className="absolute rounded-[8px] outline-4 outline-offset-4 outline-emphasis outline-solid"
              style={place(focus)}
            />
            <Pointer focus={focus} />
          </>
        ) : null}
      </div>
    </div>
  );
}
