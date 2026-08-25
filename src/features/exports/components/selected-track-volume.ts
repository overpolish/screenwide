// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioTrackVolume } from "../types";

export const selectedTrackVolume = (
  volumes: AudioTrackVolume[],
  selectedTrack: string | null | undefined,
) =>
  volumes.find(
    (volume) => `audio:${volume.streamIndex.toString()}` === selectedTrack,
  )?.decibels ?? 0;
