// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";

import { ActivityRail } from "./activity-rail";
import { ProjectScreen } from "./project-screen";
import {
  bytes,
  leftAt,
  lifeLine,
  lineAt,
  mmss,
  type Project,
  stepAt,
} from "./projects-model";

// A project as the steps it walks through. Moving across the picture moves
// through the recording, but it lands on steps, not pixels: the still
// becomes that step's, with what was done in it outlined, its name and time
// on the picture and what was being said along the bottom. The rail under
// it is the whole take at a glance. Press to peek; double-click to open.

const fade = { duration: motionDurations.state, ease: motionEasings.out };

/** A caption on the picture, in the island's black so it reads on any
 * still. */
export function PictureCaption({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "rounded-full bg-island/65 px-2 py-0.5 text-caption text-white backdrop-blur-md",
        className,
      )}
    >
      {children}
    </span>
  );
}

export function ProjectCard({
  className,
  corner,
  hits = [],
  isLinked = false,
  isPeeked = false,
  onHoverChange,
  onOpen,
  onPeek,
  previewAt = null,
  project,
}: {
  onOpen: (at: number) => void;
  onPeek: (at: number) => void;
  project: Project;
  className?: string;
  /** Something laid on the picture's top end, such as a count of takes. */
  corner?: ReactNode;
  /** Where a search found something, in seconds. */
  hits?: number[];
  /** Pointed at elsewhere, such as in the storage strip. */
  isLinked?: boolean;
  /** Open in the peek, which has taken the picture. */
  isPeeked?: boolean;
  onHoverChange?: (isHovered: boolean) => void;
  /** A moment to show, from outside, such as a search hit pointed at. */
  previewAt?: number | null;
}) {
  const [pointerAt, setPointerAt] = useState<number | null>(null);
  const { steps } = project;
  const restAt =
    (hits[0] as number | undefined) ??
    leftAt(project) ??
    steps[Math.min(1, steps.length - 1)].start;
  const at = pointerAt ?? previewAt ?? restAt;
  const step = stepAt(project, at);
  const isAiming = pointerAt !== null || previewAt !== null;
  const said = isAiming ? lineAt(project, at) : undefined;

  return (
    <article
      aria-label={project.title}
      className={cn("group/card flex min-w-0 flex-col gap-inline", className)}
      onPointerEnter={() => onHoverChange?.(true)}
      onPointerLeave={() => onHoverChange?.(false)}
    >
      <div className="relative aspect-[16/10]">
        {isPeeked ? (
          <div className="absolute inset-0 rounded-panel bg-recess" />
        ) : (
          <motion.button
            className={cn(
              "absolute inset-0 cursor-default overflow-hidden rounded-panel bg-recess outline-2 outline-offset-2 transition-[outline-color,box-shadow] focus-visible:outline-focus",
              isLinked ? "outline-fg shadow-lg" : "outline-transparent",
              "hover:shadow-lg",
            )}
            layoutId={`picture-${project.id}`}
            onClick={() => {
              onPeek(at);
            }}
            onDoubleClick={() => {
              onOpen(at);
            }}
            onPointerLeave={() => {
              setPointerAt(null);
            }}
            onPointerMove={(event) => {
              const box = event.currentTarget.getBoundingClientRect();
              const share = (event.clientX - box.left) / box.width;
              setPointerAt(
                Math.min(0.999, Math.max(0, share)) * project.duration,
              );
            }}
            transition={{
              duration: motionDurations.travel,
              ease: motionEasings.out,
            }}
            type="button"
          >
            <AnimatePresence initial={false}>
              <motion.div
                animate={{ opacity: 1 }}
                className="absolute inset-0"
                exit={{ opacity: 0 }}
                initial={{ opacity: 0 }}
                key={step}
                transition={fade}
              >
                <ProjectScreen
                  className="absolute inset-0"
                  project={project}
                  showsFocus={isAiming || hits.length > 0}
                  step={step}
                />
              </motion.div>
            </AnimatePresence>
            <span className="absolute inset-0 rounded-panel inset-ring inset-ring-divider" />
            <AnimatePresence>
              {isAiming ? (
                <motion.span
                  animate={{ opacity: 1, y: 0 }}
                  className="absolute start-2 top-2 flex"
                  exit={{ opacity: 0, y: -4 }}
                  initial={{ opacity: 0, y: -4 }}
                  transition={fade}
                >
                  <PictureCaption className="tabular-nums">
                    {`${String(step + 1)}/${String(steps.length)}  ${steps[step].title} · ${mmss(at)}`}
                  </PictureCaption>
                </motion.span>
              ) : null}
            </AnimatePresence>
            {corner ? (
              <span className="absolute end-2 top-2 flex">{corner}</span>
            ) : null}
            {said ? (
              <span className="absolute inset-x-2 bottom-2 flex justify-center">
                <PictureCaption className="line-clamp-1 text-center">
                  {said.text}
                </PictureCaption>
              </span>
            ) : null}
          </motion.button>
        )}
      </div>
      <ActivityRail
        at={isAiming ? at : null}
        hits={hits}
        project={project}
        step={isAiming ? step : null}
      />
      <div className="flex min-w-0 flex-col ps-0.5">
        <Text as="h3" className="truncate" dir="auto" variant="body">
          {project.title}
        </Text>
        <Text className="truncate tabular-nums" variant="caption">
          {lifeLine(project)}
        </Text>
        <Text
          className="truncate text-fg-tertiary tabular-nums"
          variant="caption"
        >
          {`${String(steps.length)} steps · ${mmss(project.duration)} · ${bytes(project.size)}`}
        </Text>
      </div>
    </article>
  );
}
