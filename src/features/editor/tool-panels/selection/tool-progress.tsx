// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { useEffect, useState } from "react";

import { ProgressBar } from "../../../../components/base/progress-bar/progress-bar";
import { motionDurations, motionEasings } from "../../../../lib/motion";

/** Work shorter than this finishes without a bar. */
const SHOW_AFTER_MS = 500;
/** How long a finished bar rests full before it closes. */
const FULL_FOR_MS = 400;

/**
 * A bar under a tool's row while the tool works on the track. It opens only
 * for work that lasts, so a tool with its result kept from before, or one
 * done in a moment, never flashes a bar; and once open it fills before it
 * closes rather than vanishing part-way.
 */
export function ToolProgress({
  isWorking,
  label,
  value,
}: {
  isWorking: boolean;
  label: string;
  /** How far the work has got, 0 to 1. */
  value: number;
}) {
  const [isShown, setIsShown] = useState(false);
  const [isFinishing, setIsFinishing] = useState(false);
  const [wasWorking, setWasWorking] = useState(isWorking);
  if (wasWorking !== isWorking) {
    setWasWorking(isWorking);
    setIsFinishing(!isWorking && isShown);
  }

  useEffect(() => {
    if (!isWorking || isShown) return;
    const timer = window.setTimeout(() => {
      setIsShown(true);
    }, SHOW_AFTER_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [isShown, isWorking]);

  useEffect(() => {
    if (!isFinishing) return;
    const timer = window.setTimeout(() => {
      setIsFinishing(false);
      setIsShown(false);
    }, FULL_FOR_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [isFinishing]);

  const isVisible = isShown && (isWorking || isFinishing);
  return (
    <AnimatePresence initial={false}>
      {isVisible ? (
        <motion.div
          animate={{ height: "auto", opacity: 1 }}
          className="overflow-hidden"
          exit={{ height: 0, opacity: 0 }}
          initial={{ height: 0, opacity: 0 }}
          transition={{
            duration: motionDurations.travel,
            ease: motionEasings.out,
          }}
        >
          <ProgressBar
            aria-label={label}
            className="mt-control"
            value={(isWorking ? value : 1) * 100}
          />
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
