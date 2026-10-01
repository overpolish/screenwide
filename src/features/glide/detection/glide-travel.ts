// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The direction of a step of `size` worth of travel, or 0 when it is short. */
export const axisStep = (travel: number, size: number): -1 | 0 | 1 => {
  if (Math.abs(travel) < size) return 0;
  return travel < 0 ? -1 : 1;
};

/**
 * Tracks one axis as travel since its last turn point. Movement extends the
 * extremum; counter-movement past the hysteresis confirms a reversal and
 * re-origins at the extremum, so a swipe back always measures from the turn
 * rather than the gesture's start. Travel is not spent but wiped: the detector
 * rebases both axes the moment one of them buys a transition.
 */
export class TurnPointTracker {
  #direction = 0;
  #extremum = 0;
  readonly #hysteresis: number;
  #origin = 0;
  #position = 0;

  constructor(hysteresis: number) {
    this.#hysteresis = hysteresis;
  }

  /** Signed distance from the last turn point. */
  get travel() {
    return this.#position - this.#origin;
  }

  /** Drops the current travel without forgetting where the pointer is. */
  rebase() {
    this.#direction = 0;
    this.#extremum = this.#position;
    this.#origin = this.#position;
  }

  reset() {
    this.#position = 0;
    this.rebase();
  }

  /** Reports the direction of a step of `size`, or 0 when travel is short. */
  step(size: number) {
    return axisStep(this.travel, size);
  }

  update(delta: number) {
    this.#position += delta;
    const offset = this.#position - this.#extremum;

    // The first movement, or one extending the current direction, only pushes
    // the extremum out; counter-movement past the hysteresis turns instead.
    if (this.#direction === 0 || offset * this.#direction > 0) {
      if (offset !== 0) this.#direction = Math.sign(offset);
      this.#extremum = this.#position;
      return;
    }
    if (Math.abs(offset) <= this.#hysteresis) return;

    this.#direction = -this.#direction;
    this.#origin = this.#extremum;
    this.#extremum = this.#position;
  }
}
