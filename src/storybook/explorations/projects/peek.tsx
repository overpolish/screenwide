// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { Text } from "../../../primitives/text/text";

import { ActivityRail } from "./activity-rail";
import { PictureCaption } from "./project-card";
import { ProjectScreen } from "./project-screen";
import { lifeLine, lineAt, mmss, type Project, stepAt } from "./projects-model";

// A project looked at without opening it, as Quick Look does: its picture
// grows out of the card into the middle of the window and plays, with what
// is said along the bottom and its steps as chapters under it. Return goes
// on into the editor from where it is; Escape puts it back in its card.
// Space plays and pauses, the arrows go a step back or on.

const travel = { duration: motionDurations.travel, ease: motionEasings.out };

function usePlayback(project: Project, from: number) {
  const [at, setAt] = useState(from);
  const [isPlaying, setIsPlaying] = useState(true);
  const lastRef = useRef<number | null>(null);
  useEffect(() => {
    if (!isPlaying) return;
    let frame = 0;
    const tick = (now: number) => {
      const last = lastRef.current ?? now;
      lastRef.current = now;
      setAt((current) => {
        const next = current + (now - last) / 1_000;
        if (next >= project.duration) {
          setIsPlaying(false);
          return project.duration;
        }
        return next;
      });
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      lastRef.current = null;
    };
  }, [isPlaying, project.duration]);
  return { at, isPlaying, seek: setAt, setIsPlaying };
}

export function Peek({
  from,
  onClose,
  onOpen,
  project,
}: {
  from: number;
  onClose: () => void;
  onOpen: (at: number) => void;
  project: Project;
}) {
  const { at, isPlaying, seek, setIsPlaying } = usePlayback(project, from);
  const step = stepAt(project, at);
  const said = lineAt(project, at);

  const keysRef = useRef<(event: KeyboardEvent) => void>(() => undefined);
  useEffect(() => {
    keysRef.current = (event) => {
      if (event.key === "Escape") onClose();
      else if (event.key === "Enter") onOpen(at);
      else if (event.key === " ") setIsPlaying(!isPlaying);
      else if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
        const next = step + (event.key === "ArrowRight" ? 1 : -1);
        const target = project.steps[next] as
          (typeof project.steps)[number] | undefined;
        if (target) seek(target.start);
      } else return;
      event.preventDefault();
      event.stopPropagation();
    };
  });
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      keysRef.current(event);
    };
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
    };
  }, []);

  return (
    <>
      <motion.div
        animate={{ opacity: 1 }}
        className="absolute inset-0 z-20 bg-island/40 backdrop-blur-md"
        exit={{ opacity: 0 }}
        initial={{ opacity: 0 }}
        onClick={onClose}
        transition={travel}
      />
      <div className="pointer-events-none absolute inset-0 z-30 flex items-center justify-center p-group">
        <div className="flex w-full max-w-[40rem] flex-col gap-separate">
          <motion.div
            className="pointer-events-auto relative aspect-[16/10] overflow-hidden rounded-panel shadow-2xl"
            layoutId={`picture-${project.id}`}
            transition={travel}
          >
            <AnimatePresence initial={false}>
              <motion.div
                animate={{ opacity: 1 }}
                className="absolute inset-0"
                exit={{ opacity: 0 }}
                initial={{ opacity: 0 }}
                key={step}
                transition={travel}
              >
                <ProjectScreen
                  className="absolute inset-0"
                  project={project}
                  step={step}
                />
              </motion.div>
            </AnimatePresence>
            <span className="absolute start-3 top-3 flex">
              <PictureCaption className="text-label">
                {`${String(step + 1)}/${String(project.steps.length)}  ${project.steps[step].title}`}
              </PictureCaption>
            </span>
            {said ? (
              <span className="absolute inset-x-3 bottom-3 flex justify-center">
                <PictureCaption className="text-label">
                  {said.text}
                </PictureCaption>
              </span>
            ) : null}
          </motion.div>
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            className="pointer-events-auto dark flex flex-col gap-separate text-fg"
            exit={{ opacity: 0, y: 8 }}
            initial={{ opacity: 0, y: 8 }}
            transition={{ ...travel, delay: motionDurations.feedback }}
          >
            {/* The steps as chapters, each as wide as it ran. */}
            <div className="flex gap-[2px]" dir="ltr">
              {project.steps.map(({ end, start, title }, index) => (
                <button
                  className={cn(
                    "min-w-0 truncate text-start text-caption transition-colors",
                    index === step
                      ? "text-fg"
                      : "text-fg-tertiary hover:text-fg-secondary",
                  )}
                  key={title}
                  onClick={() => {
                    seek(start);
                  }}
                  style={{ flexBasis: 0, flexGrow: end - start }}
                  type="button"
                >
                  {title}
                </button>
              ))}
            </div>
            <div
              onClick={(event) => {
                const box = event.currentTarget.getBoundingClientRect();
                seek(
                  ((event.clientX - box.left) / box.width) * project.duration,
                );
              }}
            >
              <ActivityRail
                at={at}
                project={project}
                size="large"
                step={step}
              />
            </div>
            <div className="flex items-center gap-separate">
              <IconButton
                aria-label={isPlaying ? "Pause" : "Play"}
                onPress={() => {
                  setIsPlaying(!isPlaying);
                }}
                variant="ghost"
              >
                {isPlaying ? <Pause /> : <Play />}
              </IconButton>
              <Text as="span" className="tabular-nums" variant="label">
                {`${mmss(at)} / ${mmss(project.duration)}`}
              </Text>
              <div className="flex min-w-0 grow flex-col">
                <Text as="span" className="truncate" variant="label">
                  {project.title}
                </Text>
                <Text as="span" className="truncate" variant="caption">
                  {lifeLine(project)}
                </Text>
              </div>
              <Button
                color="primary"
                onPress={() => {
                  onOpen(at);
                }}
              >
                Open in Editor
              </Button>
            </div>
          </motion.div>
        </div>
      </div>
    </>
  );
}
