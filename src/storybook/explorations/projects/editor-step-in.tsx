// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { useEffect, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { EditorWindow } from "../editor/editor-window";

import { PictureCaption } from "./project-card";
import { ProjectScreen } from "./project-screen";
import { type Project, stepAt } from "./projects-model";

// Opening a project is not a cut to another window: the picture it was
// shown by grows to fill the window, and the editor comes up around it,
// at the moment it was opened from. The editor here is the editor
// exploration, which edits the walkthrough whichever project was opened.
// Escape comes back to the projects.

const grow = { duration: 0.36, ease: motionEasings.out };
const HINT_MS = 2_400;

export function EditorStepIn({
  at,
  onBack,
  project,
}: {
  at: number;
  onBack: () => void;
  project: Project;
}) {
  const [isHinting, setIsHinting] = useState(true);
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setIsHinting(false);
    }, HINT_MS);
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onBack();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("keydown", onKey);
    };
  }, [onBack]);

  return (
    <motion.div
      className="absolute inset-0 z-40 overflow-hidden bg-surface"
      layoutId={`picture-${project.id}`}
      style={{ borderRadius: 0 }}
      transition={grow}
    >
      <motion.div
        animate={{ opacity: 1 }}
        className="absolute inset-0"
        initial={{ opacity: 0 }}
        transition={{
          delay: grow.duration * 0.6,
          duration: motionDurations.travel,
        }}
      >
        <EditorWindow kind={project.look === "audio" ? "audio" : "recording"} />
      </motion.div>
      {/* The picture it grew from, giving way to the editor. */}
      <motion.div
        animate={{ opacity: 0 }}
        className="pointer-events-none absolute inset-0"
        initial={{ opacity: 1 }}
        transition={{
          delay: grow.duration * 0.6,
          duration: motionDurations.travel,
        }}
      >
        <ProjectScreen
          className="absolute inset-0"
          project={project}
          step={stepAt(project, at)}
        />
      </motion.div>
      <AnimatePresence>
        {isHinting ? (
          <motion.span
            animate={{ opacity: 1, y: 0 }}
            className="absolute inset-x-0 top-14 flex justify-center"
            exit={{ opacity: 0, y: -6 }}
            initial={{ opacity: 0, y: -6 }}
            transition={{
              delay: grow.duration,
              duration: motionDurations.travel,
            }}
          >
            <PictureCaption className="text-label">
              Escape goes back to Projects
            </PictureCaption>
          </motion.span>
        ) : null}
      </AnimatePresence>
    </motion.div>
  );
}
