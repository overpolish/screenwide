// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Where `value` sits in `steps`: its own place, or for a number none of
 * them is, as one saved before the steps were, the nearest. Anything else
 * lands on the first.
 */
export const stepIndex = <T extends number | string>(
  steps: readonly T[],
  value: T,
) => {
  const exact = steps.indexOf(value);
  if (exact >= 0 || typeof value !== "number" || !Number.isFinite(value))
    return Math.max(exact, 0);
  let nearest = 0;
  for (let index = 1; index < steps.length; index++) {
    const distance = Math.abs(Number(steps[index]) - value);
    if (distance < Math.abs(Number(steps[nearest]) - value)) nearest = index;
  }
  return nearest;
};
