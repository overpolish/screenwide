// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, useMotionValue, useReducedMotion } from "motion/react";
import { useCallback, useEffect, useSyncExternalStore } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";

export type CropTurn = {
  /** The crop is shown: the tool is in hand and the layer stands upright. */
  isFlat: boolean;
  /** How much of the layer's turn the preview draws, from upright at zero to
   * as turned at one. */
  share: number;
};

/**
 * How much of a turned layer's turn the preview draws as the crop tool comes
 * and goes. The crop overlay and the magnifier work on the upright picture, so
 * taking the tool up eases the turn out with the layer still cropped, and the
 * crop is only shown once it stands upright; putting the tool down takes the
 * crop away at once and eases the turn back in. The turn eases about the
 * layer's own middle, which the whole uncropped picture would move. A layer
 * that is not turned, or reduced motion, switches at once.
 */
export function useCropTurn(isCropping: boolean, isTurned: boolean): CropTurn {
  const reduceMotion = useReducedMotion() === true;
  const target = isCropping ? 0 : 1;
  const eases = isTurned && !reduceMotion;
  const value = useMotionValue(target);
  const subscribe = useCallback(
    (onChange: () => void) => value.on("change", onChange),
    [value],
  );
  const share = useSyncExternalStore(subscribe, () => value.get());
  useEffect(() => {
    if (!eases) {
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
  }, [eases, target, value]);
  const drawn = eases ? share : target;
  return { isFlat: isCropping && drawn === 0, share: drawn };
}
