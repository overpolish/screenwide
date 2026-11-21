// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, useMotionValue, useReducedMotion } from "motion/react";
import { useCallback, useEffect, useSyncExternalStore } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";

export type CropTurn = {
  /** The crop is shown: the tool is in hand, the layer stands upright and
   * the whole picture is open around the crop. */
  isFlat: boolean;
  /** How far the picture is opened out around its crop, from as cropped at
   * zero to whole at one. */
  open: number;
  /** How much of the layer's turn the preview draws, from upright at zero to
   * as turned at one. */
  share: number;
};

/**
 * How the picture moves as the crop tool comes and goes. The crop overlay
 * and the magnifier work on the whole, upright picture, so taking the tool
 * up opens the picture out around its crop while a turned layer's turn eases
 * out, and the crop is only shown once both are done; putting the tool down
 * takes the crop away and eases both back, so the picture is seen closing
 * onto its crop rather than jumping (user asked). The turn eases about the
 * layer's own middle, which the whole uncropped picture would move. Reduced
 * motion switches at once.
 */
export function useCropTurn(isCropping: boolean, isTurned: boolean): CropTurn {
  const reduceMotion = useReducedMotion() === true;
  const target = isCropping ? 1 : 0;
  const value = useMotionValue(target);
  const subscribe = useCallback(
    (onChange: () => void) => value.on("change", onChange),
    [value],
  );
  const progress = useSyncExternalStore(subscribe, () => value.get());
  useEffect(() => {
    if (reduceMotion) {
      value.jump(target);
      return;
    }
    const controls = animate(value, target, {
      duration: motionDurations.travel,
      ease: motionEasings.out,
    });
    return () => {
      controls.stop();
    };
  }, [reduceMotion, target, value]);
  const open = reduceMotion ? target : progress;
  return {
    isFlat: isCropping && open === 1,
    open,
    share: isTurned ? 1 - open : 1,
  };
}
