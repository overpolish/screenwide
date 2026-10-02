// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

// Capture may omit unchanged screen frames. Effects still need an output
// clock: sampling only captured frames freezes the cursor and annotations.
static uint64_t export_duration_us(uint64_t source_duration,
    const ScreenwideTimelineRange *ranges, uint32_t count) {
  if (ranges == NULL || count == 0) return source_duration;
  const ScreenwideTimelineRange *last = &ranges[count - 1];
  return last->output_start_us + (uint64_t)llround(
      (last->source_end_us - last->source_start_us) / last->playback_rate);
}

static CMTime export_source_time(CMTime output,
    const ScreenwideTimelineRange *ranges, uint32_t count) {
  if (ranges == NULL || count == 0) return output;
  uint64_t output_us = (uint64_t)llround(CMTimeGetSeconds(output) * 1000000.0);
  for (uint32_t i = 0; i < count; i++) {
    const ScreenwideTimelineRange *range = &ranges[i];
    uint64_t end = range->output_start_us + (uint64_t)llround(
        (range->source_end_us - range->source_start_us) / range->playback_rate);
    if (output_us >= range->output_start_us && output_us < end) {
      uint64_t source = range->source_start_us + (uint64_t)llround(
          (output_us - range->output_start_us) * range->playback_rate);
      return CMTimeMake((int64_t)MIN(source, range->source_end_us - 1), 1000000);
    }
  }
  return kCMTimeInvalid;
}
