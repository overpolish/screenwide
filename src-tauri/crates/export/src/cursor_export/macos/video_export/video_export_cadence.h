// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

// The export's output clock and its readers' seeking are the shared Rust
// ones (`cursor_export/frame_clock.rs`), so every platform draws the same
// frames at the same moments.
uint32_t screenwide_export_frame_rate(double nominal);
uint64_t screenwide_export_duration_us(uint64_t source_duration_us,
                                       const ScreenwideTimelineRange *ranges, uint32_t count);
int screenwide_export_source_time_us(uint64_t output_us, const ScreenwideTimelineRange *ranges,
                                     uint32_t count, uint64_t *source_us);
int screenwide_export_should_reposition(uint64_t wanted_us, uint64_t held_us, uint64_t next_us,
                                        uint64_t frame_us);
uint64_t screenwide_export_rewind_preroll_us(void);

static inline uint64_t export_time_us(CMTime time) {
  return (uint64_t)llround(MAX(CMTimeGetSeconds(time), 0.0) * 1000000.0);
}

static inline uint64_t export_duration_us(uint64_t source_duration,
    const ScreenwideTimelineRange *ranges, uint32_t count) {
  return screenwide_export_duration_us(source_duration, ranges, count);
}

static inline CMTime export_source_time(CMTime output,
    const ScreenwideTimelineRange *ranges, uint32_t count) {
  uint64_t source_us = 0;
  if (!screenwide_export_source_time_us(export_time_us(output), ranges, count, &source_us))
    return kCMTimeInvalid;
  return CMTimeMake((int64_t)source_us, 1000000);
}
