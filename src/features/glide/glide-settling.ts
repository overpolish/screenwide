// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Whether the detector acts on travel, or is waiting for the fingers to rest. */
export type GlidePhase = "ready" | "settling";

export type GlideRestOptions = {
  /** Movement in one sample that still counts as fingers moving, not resting. */
  motionNoiseFloor: number;
  /** Quiet time that re-arms the detector after a transition. */
  restMs: number;
};

/**
 * One gesture, one transition: a transition locks the detector until the
 * fingers have held still for `restMs`, so a long flick cannot chain through
 * several phases. Any motion pushes the rest back, and the return to `ready`
 * is what the trackpad ticks - the gesture announcing it will listen again.
 */
export class RestGate {
  #lastActiveAt = 0;
  readonly #options: GlideRestOptions;
  #phase: GlidePhase = "ready";

  constructor(options: GlideRestOptions) {
    this.#options = options;
  }

  get phase() {
    return this.#phase;
  }

  /** Locks after a transition, starting the quiet clock at `timestamp`. */
  hold(timestamp: number) {
    this.#lastActiveAt = timestamp;
    this.#phase = "settling";
  }

  /** Milliseconds of quiet still owed before the gesture is ready again. */
  remaining(timestamp: number) {
    if (this.#phase === "ready") return 0;
    return Math.max(this.#options.restMs - (timestamp - this.#lastActiveAt), 0);
  }

  reset() {
    this.#lastActiveAt = 0;
    this.#phase = "ready";
  }

  /**
   * Unlocks once the rest is complete, reporting whether this call is the one
   * that did it. The phase is the single source of truth, so the UI's timer and
   * an event's own unlock can race without the readiness being announced twice.
   */
  settle(timestamp: number) {
    if (this.#phase === "ready" || this.remaining(timestamp) > 0) return false;
    this.#phase = "ready";
    return true;
  }

  /** Movement worth more than the noise floor pushes the rest back. */
  stir(timestamp: number, motion: number) {
    if (
      this.#phase === "settling" &&
      motion >= this.#options.motionNoiseFloor
    ) {
      this.#lastActiveAt = timestamp;
    }
  }
}
