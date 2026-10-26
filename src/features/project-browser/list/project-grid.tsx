// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode, useEffect, useState } from "react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { Text } from "../../../primitives/text/text";

import type { ProjectSummary } from "../types";

/** How many cards come in one after another when a source opens; the rest
 * arrive with the last of them, so a long list is not slow to fill. */
const STAGGERED = 12;
const STAGGER_S = 0.025;

const travel = {
  duration: motionDurations.travel,
  ease: motionEasings.out,
};

type Group = { label: string | null; projects: ProjectSummary[] };

/**
 * The cards in their date groups. A source opening brings its cards in one
 * after another, lifted out of a blur; after that, a card that goes (deleted,
 * moved, filtered out) shrinks away while those after it slide into its
 * place, and one that comes back grows in where it belongs. Under reduced
 * motion they only fade.
 */
export function ProjectGrid({
  children,
  groups,
}: {
  /** Draws one project's card. */
  children: (project: ProjectSummary) => ReactNode;
  groups: Group[];
}) {
  const reducedMotion = useReducedMotion() ?? false;
  // Mounted afresh for each source, so this is the source opening: only
  // then do the cards come in one after another.
  const [isOpening, setIsOpening] = useState(true);
  useEffect(() => {
    const timer = window.setTimeout(
      () => {
        setIsOpening(false);
      },
      (STAGGERED * STAGGER_S + motionDurations.travel) * 1_000,
    );
    return () => {
      window.clearTimeout(timer);
    };
  }, []);
  const away = reducedMotion
    ? { filter: "blur(0px)", opacity: 0, scale: 1 }
    : { filter: "blur(6px)", opacity: 0, scale: 0.94 };
  const layout = reducedMotion ? false : "position";
  let index = 0;
  return (
    // Room at the foot for the island, so it never covers the last row.
    <div className="relative flex flex-col gap-group pe-inset pb-24">
      <AnimatePresence initial={false} mode="popLayout">
        {groups.map((group) => (
          <motion.section
            animate={{ opacity: 1 }}
            aria-label={group.label ?? t("project-browser-search-results")}
            className="flex flex-col gap-separate"
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            key={group.label ?? "results"}
            layout={layout}
            transition={travel}
          >
            {group.label ? (
              <Text as="h2" className="ps-0.5" variant="heading">
                {group.label}
              </Text>
            ) : null}
            <div className="relative grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-x-separate gap-y-group">
              <AnimatePresence mode="popLayout">
                {group.projects.map((project) => {
                  const appear = {
                    ...travel,
                    delay: isOpening
                      ? Math.min(index, STAGGERED) * STAGGER_S
                      : 0,
                  };
                  index += 1;
                  return (
                    <motion.div
                      animate={{
                        filter: "blur(0px)",
                        opacity: 1,
                        scale: 1,
                        // A filter left on keeps every card on a layer of
                        // its own, which a long list pays for in scrolling.
                        transitionEnd: { filter: "none" },
                      }}
                      className="min-w-0"
                      exit={{ ...away, transition: travel }}
                      initial={away}
                      key={project.file}
                      layout={layout}
                      transition={{
                        ...travel,
                        filter: appear,
                        opacity: appear,
                        scale: appear,
                      }}
                    >
                      {children(project)}
                    </motion.div>
                  );
                })}
              </AnimatePresence>
            </div>
          </motion.section>
        ))}
      </AnimatePresence>
    </div>
  );
}
