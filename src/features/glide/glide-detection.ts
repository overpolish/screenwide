// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  defaultGlideDetectorOptions,
  type GlideDetectorOptions,
} from "./glide-detection-options";
import {
  detectFirstFold,
  foldHorizontal,
  type GlideAction,
  type GlideFoldInput,
  stepLadder,
} from "./glide-folds";
import { GlideRefinement } from "./glide-refinement";
import {
  bottomRowRegion,
  type GlideRegion,
  regridRegion,
  sameRegion,
  stepColumns,
} from "./glide-regions";
import { type GlidePhase, RestGate } from "./glide-settling";
import { TurnPointTracker } from "./glide-travel";

export { defaultGlideDetectorOptions };
export type { GlideAction, GlideDetectorOptions };

export type GlideDetection = {
  /** A rest completed and the detector still accepts the next step. */
  becameReady: boolean;
  changed: boolean;
  pending: GlideAction | null;
  phase: GlidePhase;
  region: GlideRegion | null;
};

type GlideSample = {
  deltaX: number;
  deltaY: number;
  thirds: boolean;
  timestamp: number;
};

/** Mirrors the native opening grace, single corner refinement and rest gate. */
export class GlideDetector {
  readonly #horizontal: TurnPointTracker;
  #lastMotion = 0;
  #openingSince: number | null = null;
  readonly options: GlideDetectorOptions;
  #pending: GlideAction | null = null;
  readonly #refinement: GlideRefinement;
  #region: GlideRegion | null = null;
  readonly #rest: RestGate;
  #thirds = false;
  readonly #vertical: TurnPointTracker;

  constructor(options: Partial<GlideDetectorOptions> = {}) {
    this.options = { ...defaultGlideDetectorOptions, ...options };
    this.#horizontal = new TurnPointTracker(this.options.reversalHysteresis);
    this.#vertical = new TurnPointTracker(this.options.reversalHysteresis);
    this.#refinement = new GlideRefinement(this.options.reversalHysteresis);
    this.#rest = new RestGate(this.options);
  }

  get pending() {
    return this.#pending;
  }
  get phase() {
    return this.#rest.phase;
  }
  get region() {
    return this.#region;
  }

  get #folding(): GlideFoldInput {
    return {
      across: this.#horizontal.travel,
      down: this.#vertical.travel,
      options: this.options,
      thirds: this.#thirds,
    };
  }

  #applyThirds(thirds: boolean) {
    this.#thirds = thirds;
    if (this.#region && thirds !== (this.#region.gridCols === 3)) {
      this.#region = regridRegion(this.#region, thirds);
      this.#horizontal.rebase();
    }
  }

  #detection(
    previous: GlideRegion | null,
    pending: GlideAction | null,
    ready: boolean,
  ): GlideDetection {
    return {
      becameReady: ready && this.phase === "ready",
      changed: pending !== this.#pending || !sameRegion(previous, this.#region),
      pending: this.#pending,
      phase: this.phase,
      region: this.#region,
    };
  }

  finishOpening(timestamp: number): GlideDetection {
    const previous = this.#region;
    const pending = this.#pending;
    if (this.#openingSince !== null && this.#transition(timestamp, true)) {
      this.#rebase();
      this.#rest.hold(this.#lastMotion);
    }
    return this.#detection(previous, pending, false);
  }

  #rebase() {
    this.#horizontal.rebase();
    this.#vertical.rebase();
  }

  reset(): GlideDetection {
    const changed = this.#region !== null || this.#pending !== null;
    this.#pending = null;
    this.#region = null;
    this.#openingSince = null;
    this.#lastMotion = 0;
    this.#refinement.clear();
    this.#horizontal.reset();
    this.#vertical.reset();
    this.#rest.reset();
    return {
      becameReady: false,
      changed,
      pending: null,
      phase: this.phase,
      region: null,
    };
  }

  #resolvePending() {
    const step = this.#vertical.step(this.options.verticalThreshold);
    if (step !== 0) {
      if (step < 0) this.#pending = null;
      else if (!this.#region) {
        this.#pending = null;
        this.#region = bottomRowRegion(this.#thirds ? 3 : 2);
      }
      return true;
    }
    if (this.#region) {
      const across = this.#horizontal.step(this.options.horizontalThreshold);
      if (across === 0) return false;
      this.#region = stepColumns(this.#region, across);
    } else {
      const folded = foldHorizontal(this.#folding);
      if (!folded) return false;
      this.#region = folded;
    }
    this.#pending = null;
    return true;
  }

  restRemaining(timestamp: number) {
    return this.#rest.remaining(timestamp);
  }

  setThirds(thirds: boolean): GlideDetection {
    const previous = this.#region;
    this.#applyThirds(thirds);
    return this.#detection(previous, this.#pending, false);
  }

  settle(timestamp: number): GlideDetection {
    const previous = this.#region;
    const pending = this.#pending;
    if (this.#openingSince !== null && this.#transition(timestamp)) {
      this.#rebase();
      this.#rest.hold(this.#lastMotion);
    }
    return this.#detection(previous, pending, this.#settleRest(timestamp));
  }

  #settleRest(timestamp: number) {
    const ready = this.#rest.settle(timestamp);
    if (ready) {
      this.#rebase();
      this.#refinement.clear();
    }
    return ready;
  }

  #transition(timestamp: number, force = false) {
    if (this.#pending) return this.#resolvePending();
    const fold = this.#region
      ? stepLadder(this.#region, this.#folding)
      : detectFirstFold(this.#folding);
    if (!fold) return false;
    if (!this.#region && !force && fold.region?.rowSpan !== 1) {
      this.#openingSince ??= timestamp;
      if (timestamp - this.#openingSince < this.options.openingGraceMs)
        return false;
    }
    this.#openingSince = null;
    this.#pending = fold.pending;
    this.#region = fold.region;
    this.#refinement.allow(this.#region, fold.canRefine);
    return true;
  }

  update({ deltaX, deltaY, thirds, timestamp }: GlideSample): GlideDetection {
    const previous = this.#region;
    const pending = this.#pending;
    const ready = this.#settleRest(timestamp);
    if (Math.abs(deltaX) + Math.abs(deltaY) >= this.options.motionNoiseFloor)
      this.#lastMotion = timestamp;
    this.#applyThirds(thirds);
    this.#horizontal.update(deltaX);
    this.#vertical.update(deltaY);
    if (this.phase === "settling") {
      this.#rest.stir(timestamp, Math.abs(deltaX) + Math.abs(deltaY));
      const refined = this.#refinement.update({
        deltaX,
        deltaY,
        region: this.#region,
        threshold: this.options.verticalThreshold,
        timestamp,
      });
      if (refined) {
        this.#region = refined;
        this.#rest.hold(timestamp);
      }
      this.#rebase();
    } else if (this.#transition(timestamp)) {
      this.#rebase();
      this.#rest.hold(timestamp);
    }
    return this.#detection(previous, pending, ready);
  }
}
