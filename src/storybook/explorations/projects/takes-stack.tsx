// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { motion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { Text } from "../../../primitives/text/text";

import { PictureCaption, ProjectCard } from "./project-card";
import { type Project } from "./projects-model";
import { bestTake, flaws } from "./takes";

// Takes of the same thing, recorded one after another, are one card: the
// best of them on top, the others as sheets behind it. Opened, they stand
// side by side, each saying what sets it apart (a pause, an "erm"), and
// keeping one folds the others away.

const travel = { duration: motionDurations.travel, ease: motionEasings.out };

type Actions = {
  onHoverChange: (id: string | null) => void;
  onOpen: (project: Project, at: number) => void;
  onPeek: (project: Project, at: number) => void;
  peekedId: string | null;
};

export function TakesStack({
  isOpen,
  onKeep,
  onToggle,
  takes,
  ...actions
}: Actions & {
  isOpen: boolean;
  onKeep: (take: Project) => void;
  onToggle: () => void;
  takes: Project[];
}) {
  const best = bestTake(takes);
  const card = (take: Project, corner?: React.ReactNode) => (
    <ProjectCard
      corner={corner}
      isPeeked={actions.peekedId === take.id}
      onHoverChange={(isHovered) => {
        actions.onHoverChange(isHovered ? take.id : null);
      }}
      onOpen={(at) => {
        actions.onOpen(take, at);
      }}
      onPeek={(at) => {
        actions.onPeek(take, at);
      }}
      project={take}
    />
  );

  if (!isOpen)
    return (
      <motion.div className="relative" layout="position" transition={travel}>
        {/* The other takes, as sheets showing above the top one. */}
        <span className="absolute inset-x-6 -top-4 aspect-[16/10] rounded-panel bg-fill-strong opacity-60" />
        <span className="absolute inset-x-3 -top-2 aspect-[16/10] rounded-panel bg-fill-strong" />
        <div className="relative">
          {card(
            best,
            <button onClick={onToggle} type="button">
              <PictureCaption className="hover:bg-island/85">
                {`${String(takes.length)} takes`}
              </PictureCaption>
            </button>,
          )}
        </div>
      </motion.div>
    );

  return (
    <motion.div
      className="col-span-full flex flex-col gap-separate rounded-panel bg-recess p-inset"
      layout
      transition={travel}
    >
      <div className="flex items-center justify-between">
        <Text variant="heading">{`${best.title} · ${String(takes.length)} takes`}</Text>
        <Button onPress={onToggle} variant="ghost">
          Done
        </Button>
      </div>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-x-separate gap-y-group">
        {takes.map((take) => {
          const against = flaws(take);
          const isBest = take.id === best.id;
          return (
            <motion.div
              animate={{ opacity: 1, scale: 1 }}
              className="flex flex-col gap-inline"
              initial={{ opacity: 0, scale: 0.94 }}
              key={take.id}
              transition={travel}
            >
              {card(
                take,
                <PictureCaption>{`Take ${String(take.take?.number ?? 0)}`}</PictureCaption>,
              )}
              <div className="flex items-center gap-inline">
                <span
                  className={cn(
                    "flex grow flex-wrap gap-1 text-caption",
                    isBest ? "text-success" : "text-fg-secondary",
                  )}
                >
                  {isBest ? (
                    <>
                      <Check className="size-icon-sm" />
                      Cleanest take
                    </>
                  ) : (
                    against.join(" · ")
                  )}
                </span>
                <Button
                  onPress={() => {
                    onKeep(take);
                  }}
                >
                  Keep
                </Button>
              </div>
            </motion.div>
          );
        })}
      </div>
    </motion.div>
  );
}
