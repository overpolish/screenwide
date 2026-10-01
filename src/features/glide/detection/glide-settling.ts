// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Whether the detector acts on travel, or is waiting for the hand to rest. */
export type GlidePhase = "ready" | "settling";

export type GlideRestOptions = {
  /** Combined absolute axis movement per sample that resets the rest timer. */
  motionNoiseFloor: number;
  /** Quiet time that re-arms the detector after a transition. */
  restMs: number;
};

/**
 * Each transition waits for stillness, allowing small input jitter;
 * the return to `ready` drives the trackpad tick and preview pulse.
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

  /** Milliseconds remaining before the gesture is ready again. */
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
  /** Movement at or above the noise floor restarts the quiet period. */
  stir(timestamp: number, motion: number) {
    if (
      this.#phase === "settling" &&
      motion >= this.#options.motionNoiseFloor
    ) {
      this.#lastActiveAt = timestamp;
    }
  }
}
