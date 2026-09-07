// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { stepRows, type GlideRegion } from "./glide-regions";
import { TurnPointTracker } from "./glide-travel";

/** One deliberate vertical turn after a horizontal fold, matching native intent. */
export class GlideRefinement {
  #allowed = false;
  #history: { deltaX: number; deltaY: number; timestamp: number }[] = [];
  readonly #vertical: TurnPointTracker;

  constructor(hysteresis: number) {
    this.#vertical = new TurnPointTracker(hysteresis);
  }

  allow(region: GlideRegion | null, canRefine: boolean) {
    this.clear();
    this.#allowed =
      canRefine &&
      region !== null &&
      region.rowSpan === 2 &&
      region.colSpan < region.gridCols;
  }

  clear() {
    this.#allowed = false;
    this.#vertical.reset();
    this.#history = [];
  }

  update({
    deltaX,
    deltaY,
    region,
    threshold,
    timestamp,
  }: {
    deltaX: number;
    deltaY: number;
    region: GlideRegion | null;
    threshold: number;
    timestamp: number;
  }) {
    if (!this.#allowed || !region) return null;
    this.#history.push({ deltaX, deltaY, timestamp });
    this.#history = this.#history.filter(
      (sample) => timestamp - sample.timestamp <= 60,
    );
    const vertical = this.#history.reduce(
      (sum, sample) => sum + sample.deltaY,
      0,
    );
    const horizontal = this.#history.reduce(
      (sum, sample) => sum + Math.abs(sample.deltaX),
      0,
    );
    if (Math.abs(vertical) <= horizontal * 1.5) {
      this.#vertical.reset();
      return null;
    }
    this.#vertical.update(deltaY);
    const step = this.#vertical.step(threshold);
    if (step === 0) return null;
    this.clear();
    return stepRows(region, step);
  }
}
