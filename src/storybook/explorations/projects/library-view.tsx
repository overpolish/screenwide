// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { Button } from "../../../primitives/button/button";
import { Text } from "../../../primitives/text/text";

import { PickUpShelf } from "./pick-up-shelf";
import { ProjectCard } from "./project-card";
import { byDay, bytes, isReclaimable, type Project } from "./projects-model";
import { TakesStack } from "./takes-stack";

// A source's projects: what was left part way through on a shelf at the
// top of Recent, then the rest by day, takes of one thing as one stack. Or,
// asked from the storage strip, only what could be freed, with the one
// press that frees it.

const travel = { duration: motionDurations.travel, ease: motionEasings.out };
const GRID =
  "grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-x-separate gap-y-group";

type Unit = { project: Project } | { group: string; takes: Project[] };

/** Each project on its own, and each set of takes as one, where its
 * latest take falls. */
const unitsOf = (projects: Project[]): Unit[] => {
  const units: Unit[] = [];
  for (const project of projects) {
    if (!project.take) units.push({ project });
    else if (
      !units.some(
        (unit) => "group" in unit && unit.group === project.take?.group,
      )
    )
      units.push({
        group: project.take.group,
        takes: projects
          .filter((take) => take.take?.group === project.take?.group)
          .sort(
            (first, second) =>
              (first.take?.number ?? 0) - (second.take?.number ?? 0),
          ),
      });
  }
  return units;
};

export function LibraryView({
  isReclaiming,
  linkedId,
  onFree,
  onHoverChange,
  onKeepTake,
  onOpen,
  onPeek,
  onToggleTakes,
  openTakes,
  peekedId,
  projects,
  showsShelf,
}: {
  isReclaiming: boolean;
  linkedId: string | null;
  onFree: (projects: Project[]) => void;
  onHoverChange: (id: string | null) => void;
  onKeepTake: (take: Project) => void;
  onOpen: (project: Project, at: number) => void;
  onPeek: (project: Project, at: number) => void;
  onToggleTakes: (group: string) => void;
  openTakes: ReadonlySet<string>;
  peekedId: string | null;
  projects: Project[];
  showsShelf: boolean;
}) {
  const card = (project: Project) => (
    <ProjectCard
      isLinked={project.id === linkedId}
      isPeeked={project.id === peekedId}
      onHoverChange={(isHovered) => {
        onHoverChange(isHovered ? project.id : null);
      }}
      onOpen={(at) => {
        onOpen(project, at);
      }}
      onPeek={(at) => {
        onPeek(project, at);
      }}
      project={project}
    />
  );
  const unitView = (unit: Unit, index: number) => (
    <motion.div
      animate={{ opacity: 1, scale: 1 }}
      className={
        "group" in unit && openTakes.has(unit.group)
          ? "col-span-full"
          : "min-w-0"
      }
      exit={{ opacity: 0, scale: 0.9, transition: travel }}
      initial={{ opacity: 0, scale: 0.96 }}
      key={"group" in unit ? unit.group : unit.project.id}
      layout="position"
      transition={{ ...travel, delay: Math.min(index, 10) * 0.025 }}
    >
      {"group" in unit ? (
        <TakesStack
          isOpen={openTakes.has(unit.group)}
          onHoverChange={onHoverChange}
          onKeep={(take) => {
            onKeepTake(take);
            onToggleTakes(unit.group);
          }}
          onOpen={onOpen}
          onPeek={onPeek}
          onToggle={() => {
            onToggleTakes(unit.group);
          }}
          peekedId={peekedId}
          takes={unit.takes}
        />
      ) : (
        card(unit.project)
      )}
    </motion.div>
  );

  if (isReclaiming) {
    const freeable = projects.filter(isReclaimable);
    const total = freeable.reduce((sum, { size }) => sum + size, 0);
    return (
      <>
        <div className="flex items-center gap-separate rounded-panel bg-fill p-inset">
          <div className="flex grow flex-col">
            <Text variant="label">
              Exported a while ago and untouched since
            </Text>
            <Text variant="caption">
              Their exports stay where they are. Only the originals go, to
              Recently Deleted.
            </Text>
          </div>
          <Button
            color="primary"
            onPress={() => {
              onFree(freeable);
            }}
          >
            {`Free ${bytes(total)}`}
          </Button>
        </div>
        <div className={GRID}>
          {freeable.map((project) => (
            <div key={project.id}>{card(project)}</div>
          ))}
        </div>
      </>
    );
  }

  let index = 0;
  return (
    <>
      {showsShelf ? <PickUpShelf onOpen={onOpen} projects={projects} /> : null}
      <AnimatePresence initial={false} mode="popLayout">
        {byDay(projects).map((group) => (
          <motion.section
            aria-label={group.label}
            className="flex flex-col gap-separate"
            key={group.label}
            layout="position"
            transition={travel}
          >
            <Text as="h2" className="ps-0.5" variant="heading">
              {group.label}
            </Text>
            <div className={`relative ${GRID}`}>
              <AnimatePresence mode="popLayout">
                {unitsOf(group.projects).map((unit) => unitView(unit, index++))}
              </AnimatePresence>
            </div>
          </motion.section>
        ))}
      </AnimatePresence>
    </>
  );
}
