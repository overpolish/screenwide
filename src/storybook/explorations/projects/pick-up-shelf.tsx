// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { Button } from "../../../primitives/button/button";
import { Text } from "../../../primitives/text/text";

import { ActivityRail } from "./activity-rail";
import { PictureCaption } from "./project-card";
import { ProjectScreen } from "./project-screen";
import {
  leftAt,
  lineAt,
  mmss,
  type Project,
  since,
  stepAt,
} from "./projects-model";

// What was being worked on, at the top of Recent: each project left part
// way through in the editor, shown where it was left (the step, what was
// being said, the playhead on its rail), so going back to it is one press
// and lands there.

const travel = { duration: motionDurations.travel, ease: motionEasings.out };

/** Shows the projects left part way through, most recent first. */
export function PickUpShelf({
  onOpen,
  projects: library,
}: {
  onOpen: (project: Project, at: number) => void;
  projects: Project[];
}) {
  const projects = library
    .filter((project) => leftAt(project) !== undefined)
    .sort((first, second) => second.editedAt - first.editedAt)
    .slice(0, 2);
  if (projects.length === 0) return null;
  return (
    <section
      aria-label="Pick up where you left off"
      className="flex flex-col gap-separate"
    >
      <Text as="h2" className="ps-0.5" variant="heading">
        Pick up where you left off
      </Text>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(20rem,1fr))] gap-separate">
        {projects.map((project, index) => {
          const at = leftAt(project) ?? 0;
          const step = stepAt(project, at);
          const said = lineAt(project, at);
          return (
            // The whole card answers the pointer; Continue is its control.
            <motion.div
              animate={{ opacity: 1, y: 0 }}
              className="group/shelf flex cursor-default items-stretch gap-separate rounded-panel bg-fill p-inset-sm text-start transition-colors hover:bg-fill-strong"
              initial={{ opacity: 0, y: 6 }}
              key={project.id}
              onClick={(event) => {
                // Continue answers its own press.
                if ((event.target as Element).closest("button")) return;
                onOpen(project, at);
              }}
              transition={{ ...travel, delay: index * 0.05 }}
            >
              <span className="relative w-[46%] shrink-0 overflow-hidden rounded-control">
                <ProjectScreen
                  className="w-full"
                  project={project}
                  step={step}
                />
                {said ? (
                  <span className="absolute inset-x-1.5 bottom-1.5 flex justify-center">
                    <PictureCaption className="line-clamp-1">
                      {said.text}
                    </PictureCaption>
                  </span>
                ) : null}
              </span>
              <span className="flex min-w-0 grow flex-col justify-between gap-inline py-1 pe-1">
                <span className="flex min-w-0 flex-col">
                  <Text as="span" className="truncate" variant="label">
                    {project.title}
                  </Text>
                  <Text as="span" className="truncate" variant="caption">
                    {`${project.steps[step].title} · ${since(project.editedAt)}`}
                  </Text>
                </span>
                <ActivityRail at={at} project={project} step={step} />
                <span className="flex items-center justify-between gap-inline">
                  <Text as="span" className="tabular-nums" variant="caption">
                    {`Left at ${mmss(at)}`}
                  </Text>
                  <Button
                    onPress={() => {
                      onOpen(project, at);
                    }}
                  >
                    Continue
                  </Button>
                </span>
              </span>
            </motion.div>
          );
        })}
      </div>
    </section>
  );
}
