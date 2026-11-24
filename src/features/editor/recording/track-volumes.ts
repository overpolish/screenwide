// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AudioTrackVolume } from "../types";

/** `volumes` with the stream `streamIndex` played `decibels` louder or
 * quieter, in whole decibels; a stream at its recorded level is left out. */
export const withTrackVolume = (
  volumes: AudioTrackVolume[],
  streamIndex: number,
  decibels: number,
): AudioTrackVolume[] => [
  ...volumes.filter((volume) => volume.streamIndex !== streamIndex),
  ...(decibels === 0 ? [] : [{ decibels: Math.round(decibels), streamIndex }]),
];
