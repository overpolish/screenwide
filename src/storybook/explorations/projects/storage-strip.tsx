// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";

import { bytes, isReclaimable, type Project } from "./projects-model";

// The disk the library takes, as a picture of it: one segment per project,
// as wide as what it takes, largest first. Pointing at a segment lights up
// its card, so "what is taking all this space" is answered by looking.
// Projects exported long ago and not touched since are hatched: their
// originals could go and the exports stay, and the line under the strip
// offers to show them.

const travel = { duration: motionDurations.travel, ease: motionEasings.out };

export function StorageStrip({
  isReclaiming,
  linkedId,
  onLink,
  onReclaim,
  projects,
}: {
  isReclaiming: boolean;
  linkedId: string | null;
  onLink: (id: string | null) => void;
  onReclaim: () => void;
  projects: Project[];
}) {
  const total = projects.reduce((sum, { size }) => sum + size, 0);
  const reclaimable = projects
    .filter(isReclaimable)
    .reduce((sum, { size }) => sum + size, 0);
  const linked = projects.find(({ id }) => id === linkedId);
  return (
    <div className="flex flex-col gap-inline px-inset-sm">
      <div className="flex items-baseline justify-between gap-inline">
        <Text as="span" className="truncate" variant="caption">
          {linked ? linked.title : "On this Mac"}
        </Text>
        <Text as="span" className="shrink-0 tabular-nums" variant="caption">
          {bytes(linked ? linked.size : total)}
        </Text>
      </div>
      <div
        className="flex h-3 gap-px overflow-hidden rounded-full"
        dir="ltr"
        onPointerLeave={() => {
          onLink(null);
        }}
      >
        {[...projects]
          .sort((first, second) => second.size - first.size)
          .map((project) => (
            <motion.span
              className={cn(
                "h-full min-w-[3px] transition-colors",
                project.id === linkedId
                  ? "bg-fg"
                  : isReclaimable(project)
                    ? "bg-[repeating-linear-gradient(135deg,var(--color-fg-tertiary)_0_2px,transparent_2px_4px)]"
                    : "bg-fg-tertiary",
              )}
              key={project.id}
              layout
              onPointerEnter={() => {
                onLink(project.id);
              }}
              style={{ flexBasis: 0, flexGrow: project.size }}
              transition={travel}
            />
          ))}
      </div>
      {reclaimable > 0 ? (
        <button
          className={cn(
            "text-start text-caption transition-colors",
            isReclaiming ? "text-fg" : "text-fg-secondary hover:text-fg",
          )}
          onClick={onReclaim}
          type="button"
        >
          {isReclaiming
            ? "Show everything"
            : `${bytes(reclaimable)} could be freed`}
        </button>
      ) : null}
    </div>
  );
}
