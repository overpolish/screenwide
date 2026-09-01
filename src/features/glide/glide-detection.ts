// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  defaultGlideFoldOptions,
  detectFirstFold,
  foldHorizontal,
  type GlideAction,
  type GlideFoldInput,
  type GlideFoldOptions,
  stepLadder,
} from "./glide-folds";
import {
  bottomRowRegion,
  type GlideRegion,
  regridRegion,
  sameRegion,
  stepColumns,
  stepRows,
} from "./glide-regions";
import {
  type GlidePhase,
  type GlideRestOptions,
  RestGate,
} from "./glide-settling";
import { TurnPointTracker } from "./glide-travel";

export type { GlideAction };

export type GlideDetection = {
  /** This result completed a rest: the gesture is listening again. */
  becameReady: boolean;
  changed: boolean;
  pending: GlideAction | null;
  phase: GlidePhase;
  region: GlideRegion | null;
};

export type GlideDetectorOptions = GlideFoldOptions &
  GlideRestOptions & {
    /** Counter-movement that confirms a reversal and moves the turn point. */
    reversalHysteresis: number;
  };

export const defaultGlideDetectorOptions: GlideDetectorOptions = {
  ...defaultGlideFoldOptions,
  motionNoiseFloor: 2,
  restMs: 60,
  reversalHysteresis: 10,
};

/** One movement sample, with the grid policy and clock reading it arrived on. */
type GlideSample = {
  deltaX: number;
  deltaY: number;
  thirds: boolean;
  timestamp: number;
};

/**
 * Recognizes Glide's folding gesture from normalized screen-space deltas.
 * Positive X moves right and positive Y moves down; platform adapters own any
 * natural-scroll inversion before samples reach this class.
 *
 * One gesture buys exactly one transition: whatever else the flick had left,
 * and the other axis with it, is dropped as the detector settles. The single
 * exception is the opening sideways fold, whose settle stays porous to the
 * vertical axis for one row step - the L-curve that turns without stopping.
 */
export class GlideDetector {
  readonly #horizontal: TurnPointTracker;
  readonly options: GlideDetectorOptions;
  #pending: GlideAction | null = null;
  /** Whether the settle in progress is still open to the vertical axis. */
  #porous = false;
  #region: GlideRegion | null = null;
  readonly #rest: RestGate;
  #thirds = false;
  readonly #vertical: TurnPointTracker;

  constructor(options: Partial<GlideDetectorOptions> = {}) {
    this.options = { ...defaultGlideDetectorOptions, ...options };
    this.#horizontal = new TurnPointTracker(this.options.reversalHysteresis);
    this.#rest = new RestGate(this.options);
    this.#vertical = new TurnPointTracker(this.options.reversalHysteresis);
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

  /** The travel and policy a fold is decided from, as the folds see it. */
  get #folding(): GlideFoldInput {
    return {
      across: this.#horizontal.travel,
      down: this.#vertical.travel,
      options: this.options,
      thirds: this.#thirds,
    };
  }

  #applyThirds(thirds: boolean) {
    if (thirds === this.#thirds) return;
    this.#thirds = thirds;
    if (!this.#region) return;
    this.#region = regridRegion(this.#region, thirds);
    this.#horizontal.rebase();
  }

  /**
   * The porous settle's one conversion: vertical travel measured from the
   * opening fold folds that half into the corner mid-motion. This is a
   * settling transition, not a ready one - nothing became ready, the rest
   * re-holds from here, and the porosity closes behind it, so whatever else
   * the same motion had left is discarded as usual.
   */
  #convertPorous() {
    if (!this.#region) return false;
    const down = this.#vertical.step(this.options.verticalThreshold);
    if (down === 0) return false;

    this.#region = stepRows(this.#region, down);
    return true;
  }

  /** Reports the current state, flagging a transition of either half of it. */
  #detection(
    previousRegion: GlideRegion | null,
    previousPending: GlideAction | null,
    becameReady: boolean,
  ): GlideDetection {
    const changed =
      previousPending !== this.#pending ||
      !sameRegion(previousRegion, this.#region);
    return {
      becameReady,
      changed,
      pending: this.#pending,
      phase: this.#rest.phase,
      region: this.#region,
    };
  }

  /**
   * Sideways is no resolution the arm knows, so the flick behaves as if it had
   * never armed: the pending clears and the horizontal travel lands as usual -
   * a ladder step over a retained region, the first fold where there is none.
   */
  #escapePending() {
    if (this.#region) {
      const across = this.#horizontal.step(this.options.horizontalThreshold);
      if (across === 0) return false;
      this.#region = stepColumns(this.#region, across);
    } else {
      // An escape is not an opening fold, so its settle is never porous.
      const folded = foldHorizontal(this.#folding);
      if (!folded) return false;
      this.#region = folded;
    }
    this.#pending = null;
    return true;
  }

  /** Forgets both axes' travel: every transition starts the next one level. */
  #rebase() {
    this.#horizontal.rebase();
    this.#vertical.rebase();
  }

  reset(): GlideDetection {
    const changed = this.#region !== null || this.#pending !== null;
    this.#pending = null;
    this.#porous = false;
    this.#region = null;
    this.#horizontal.reset();
    this.#rest.reset();
    this.#vertical.reset();
    return {
      becameReady: false,
      changed,
      pending: null,
      phase: this.#rest.phase,
      region: null,
    };
  }

  /**
   * Vertical travel resolves an armed action: a further down step converts it
   * into the full-width bottom row of the grid in force, while an up step
   * disarms back to the region underneath - the one the arm was taken over, or
   * none. Down again holds the arm, the ladder's bottom having nowhere lower to
   * go; travel resolving nothing falls through to the sideways escape.
   */
  #resolvePending() {
    const step = this.#vertical.step(this.options.verticalThreshold);
    if (step === 0) return this.#escapePending();

    if (step < 0) this.#pending = null;
    else if (!this.#region) {
      this.#pending = null;
      this.#region = bottomRowRegion(this.#thirds ? 3 : 2);
    }
    return true;
  }

  /** Milliseconds of rest still owed before the gesture is ready again. */
  restRemaining(timestamp: number) {
    return this.#rest.remaining(timestamp);
  }

  /** Completes the rest when it is due; a completed rest closes porosity. */
  #restSettled(timestamp: number) {
    const becameReady = this.#rest.settle(timestamp);
    if (becameReady) this.#porous = false;
    return becameReady;
  }

  /** Re-grids for a modifier change while the fingers rest. */
  setThirds(thirds: boolean): GlideDetection {
    const previous = this.#region;
    this.#applyThirds(thirds);
    return this.#detection(previous, this.#pending, false);
  }

  /**
   * Completes a rest from the outside, for the UI timer that ticks during a
   * stillness no event reports. At-most-once by construction: a settle after
   * the same rest was already completed inside `update` reports nothing.
   */
  settle(timestamp: number): GlideDetection {
    return this.#detection(
      this.#region,
      this.#pending,
      this.#restSettled(timestamp),
    );
  }

  /** Applies at most one transition, reporting whether it took the window. */
  #transition() {
    if (this.#pending) return this.#resolvePending();
    const fold = this.#region
      ? stepLadder(this.#region, this.#folding)
      : detectFirstFold(this.#folding);
    if (!fold) return false;

    this.#pending = fold.pending;
    this.#porous = fold.porous;
    this.#region = fold.region;
    return true;
  }

  update({ deltaX, deltaY, thirds, timestamp }: GlideSample): GlideDetection {
    const previous = this.#region;
    const previousPending = this.#pending;
    // A sample arriving after the rest is complete unlocks first, then counts
    // in full: the flick that follows the pause is not itself the pause.
    const becameReady = this.#restSettled(timestamp);
    this.#applyThirds(thirds);
    this.#horizontal.update(deltaX);
    this.#vertical.update(deltaY);

    if (this.#rest.phase === "settling") {
      // Motion of either axis pushes the rest back, porous or not.
      this.#rest.stir(timestamp, Math.abs(deltaX) + Math.abs(deltaY));
      this.#horizontal.rebase();
      if (!this.#porous) this.#vertical.rebase();
      else if (this.#convertPorous()) {
        this.#porous = false;
        this.#rebase();
        this.#rest.hold(timestamp);
      }
    } else if (this.#transition()) {
      this.#rebase();
      this.#rest.hold(timestamp);
    }

    return this.#detection(previous, previousPending, becameReady);
  }
}
