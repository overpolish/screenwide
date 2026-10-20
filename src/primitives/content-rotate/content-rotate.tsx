// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnimatePresence,
  motion,
  MotionProps,
  useReducedMotion,
} from "motion/react";
import { ReactNode, useEffect, useRef } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";

type ContentRotateProps = MotionProps & {
  children: ReactNode;
  contentKey: string;
  className?: string;
  containerClassName?: string;
};

// New content drops in from above as the old drops out below, like a
// counter's wheel. Each travels its own height, so a digit at any type size
// clears the clipping box in the same time. With reduced motion the two only
// cross-fade. The first content is shown as it is, without coming in.
export const ContentRotate = ({
  children,
  className,
  containerClassName,
  contentKey,
  ...props
}: ContentRotateProps) => {
  const isFirstMountRef = useRef(true);
  const reducedMotion = useReducedMotion() ?? false;
  const travel = (distance: string) => (reducedMotion ? 0 : distance);

  useEffect(() => {
    isFirstMountRef.current = false;
  }, []);

  return (
    <div className={cn("relative overflow-hidden", containerClassName)}>
      <AnimatePresence mode="popLayout">
        <motion.div
          animate={{ opacity: 1, y: 0 }}
          className={className}
          exit={{ opacity: 0, y: travel("100%") }}
          initial={
            isFirstMountRef.current ? false : { opacity: 0, y: travel("-100%") }
          }
          key={contentKey}
          transition={{
            duration: motionDurations.state,
            ease: motionEasings.out,
          }}
          {...props}
        >
          {children}
        </motion.div>
      </AnimatePresence>
    </div>
  );
};
