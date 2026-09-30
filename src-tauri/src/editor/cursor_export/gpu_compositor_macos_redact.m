// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The records the redaction passes in `gpu_compositor_macos_redact_encode.m`
// apply, packed from a composition's annotations.

#import "gpu_compositor_macos_redact.h"
#import "../annotations/geometry.h"

#include <math.h>

/// A corner coordinate as a whole pixel inside `[0, limit]`. The box is
/// snapped outward - down at its top-left, up at its bottom-right - so no
/// covered pixel is left partly showing along the edge. The twin of Rust's
/// `redact::native::painted_bounds`, which samples the fill colour just
/// outside these same pixels.
static uint32_t redaction_edge(float value, BOOL far, uint32_t limit) {
  float edge = far ? ceilf(value) : floorf(value);
  if (!(edge > 0.0f)) return 0;
  return edge >= (float)limit ? limit : (uint32_t)edge;
}

/// A whole count a record carries as a float, or zero for one it cannot.
static uint32_t redaction_count(float value) {
  return value >= 1.0f && value < 16777216.0f ? (uint32_t)value : 0;
}

/// The most cells the cells pass averages for one classically pixelated box.
/// Rust sizes the cells well under this; a ramp's first small cells are grown
/// to stay within it.
static const uint64_t MAX_CELLS = 1u << 20;

/// How far an animated redaction has arrived, from its reveal: 0 at the
/// start of its clip and 1 once whole, which every still and every
/// redaction that does not animate is.
static float redaction_arrived(const ScreenwideAnnotation *item) {
  float arrived = item->reveal.opacity;
  return isfinite(arrived) ? fminf(fmaxf(arrived, 0.0f), 1.0f) : 1.0f;
}

/// How far a spotlight's blur has arrived: its presence, and of that the
/// share its reveal's `scale` carries, which is less than all of it only
/// while its light glides from or to a spotlight that does not blur. The twin
/// of Rust's `spotlight::native::blur_presence`.
static float spotlight_blur_arrived(const ScreenwideAnnotation *item) {
  float share = item->reveal.scale;
  share = isfinite(share) ? fminf(fmaxf(share, 0.0f), 1.0f) : 1.0f;
  return redaction_arrived(item) * share;
}

float screenwide_spotlight_blur_strength(const ScreenwideAnnotations *annotations) {
  float strength = 0.0f;
  if (annotations == NULL || annotations->items == NULL) return strength;
  for (uint32_t index = 0; index < annotations->count; ++index) {
    const ScreenwideAnnotation *item = &annotations->items[index];
    if (item->kind == SCREENWIDE_ANNOTATION_SPOTLIGHT &&
        (item->flags & SCREENWIDE_ANNOTATION_FLAG_BLUR) != 0)
      strength = fmaxf(strength, spotlight_blur_arrived(item));
  }
  return strength;
}

/// Appends the spotlights' blur over a `width` by `height` source to
/// `redactions`: one record over the whole source, as wide as the most
/// present spotlight that blurs has let it grow, followed by every
/// spotlight's hole - its box's corners, its rounding and fade, and its
/// presence beside the blur's - prepared in source pixels by the same Rust
/// the canvas passes are. Nothing where no spotlight showing blurs. The twin
/// of Rust's `redact::records::spotlight_record`.
static void append_spotlight_blur(NSMutableData *redactions,
                                  const ScreenwideAnnotations *annotations, uint32_t width,
                                  uint32_t height) {
  float strength = screenwide_spotlight_blur_strength(annotations);
  uint32_t holes = 0;
  for (uint32_t index = 0; index < annotations->count; ++index)
    if (annotations->items[index].kind == SCREENWIDE_ANNOTATION_SPOTLIGHT) holes += 1;
  if (strength <= 0.0f || width == 0 || height == 0) return;
  ScreenwideRedaction redaction = {
    .x1 = width,
    .y1 = height,
    .source_width = width,
    .mode = SCREENWIDE_REDACT_SPOTLIGHT,
    .size = screenwide_spotlight_blur_deviation(strength, width, height),
    .color = {0.0f, 0.0f, 0.0f, 1.0f},
    .entry_count = holes * 4u,
  };
  [redactions appendBytes:&redaction length:sizeof(redaction)];
  for (uint32_t index = 0; index < annotations->count; ++index) {
    const ScreenwideAnnotation *item = &annotations->items[index];
    if (item->kind != SCREENWIDE_ANNOTATION_SPOTLIGHT) continue;
    AnnotationArrowGeometry hole;
    screenwide_annotation_prepare(item->kind, item->p0[0], item->p0[1], item->p1[0],
                                  item->p1[1], item->p2[0], item->p2[1], item->width,
                                  item->head, item->reveal, &hole);
    const float entries[4][2] = {
        {hole.a.x, hole.a.y},
        {hole.b.x, hole.b.y},
        {hole.rounding, hole.width},
        {fminf(redaction_arrived(item) / strength, 1.0f), 0.0f},
    };
    [redactions appendBytes:entries length:sizeof(entries)];
  }
}

NSData *screenwide_redactions(const ScreenwideAnnotations *annotations,
                              uint32_t width, uint32_t height) {
  NSMutableData *redactions = [NSMutableData data];
  if (annotations == NULL || annotations->items == NULL) return redactions;
  for (uint32_t index = 0; index < annotations->count; ++index) {
    const ScreenwideAnnotation *item = &annotations->items[index];
    if (item->kind != SCREENWIDE_ANNOTATION_REDACT) continue;
    // `p0` is the top-left corner and `p2` the bottom-right.
    uint32_t mode = (item->flags & SCREENWIDE_ANNOTATION_FLAG_PIXELATE) != 0
        ? SCREENWIDE_REDACT_PIXELATE
        : (item->flags & SCREENWIDE_ANNOTATION_FLAG_BLUR) != 0   ? SCREENWIDE_REDACT_BLUR
        : (item->flags & SCREENWIDE_ANNOTATION_FLAG_MOSAIC) != 0 ? SCREENWIDE_REDACT_MOSAIC
                                                                 : SCREENWIDE_REDACT_FLAT;
    ScreenwideRedaction redaction = {
      .x0 = redaction_edge(item->p0[0], NO, width),
      .y0 = redaction_edge(item->p0[1], NO, height),
      .x1 = redaction_edge(item->p2[0], YES, width),
      .y1 = redaction_edge(item->p2[1], YES, height),
      .source_width = width,
      .mode = mode,
      // The seed rides in two float halves, each exact.
      .seed = ((uint32_t)item->params[1] << 16) | ((uint32_t)item->params[2] & 0xffffu),
      .size = item->params[0],
    };
    if (redaction.x1 <= redaction.x0 || redaction.y1 <= redaction.y0) continue;
    memcpy(redaction.color, item->color, sizeof(redaction.color));
    // The record's width is the radius as a share of the painted box's
    // shorter side, so the halo drawn from the unsnapped box rounds alike.
    uint32_t wide = redaction.x1 - redaction.x0, tall = redaction.y1 - redaction.y0;
    float share = isfinite(item->width) ? fminf(fmaxf(item->width, 0.0f), 50.0f) : 0.0f;
    redaction.radius = (float)MIN(wide, tall) * share / 100.0f;
    // An animated redaction arrives by covering more: a classic pixelation's
    // blocks grow from a pixel to their size, a blur widens from nothing,
    // and every other fill fades in, its share riding in the colour's alpha,
    // which an opaque fill has no other use for.
    float arrived = redaction_arrived(item);
    const float(*zones)[2] = NULL;
    if (mode == SCREENWIDE_REDACT_BLUR) {
      redaction.color[3] = 1.0f;
      redaction.size = isfinite(redaction.size) ? fmaxf(redaction.size, 0.0f) * arrived : 0.0f;
    } else if (mode == SCREENWIDE_REDACT_MOSAIC) {
      redaction.color[3] = 1.0f;
      // Cells are averaged on the GPU from the pixels themselves, so their
      // grid follows from the snapped box and the cell's size.
      float size = isfinite(redaction.size) ? fmaxf(redaction.size, 1.0f) : 1.0f;
      size = 1.0f + (size - 1.0f) * arrived;
      size = fmaxf(size, sqrtf((float)wide * (float)tall / (float)MAX_CELLS));
      redaction.size = size;
      redaction.grid[0] = (uint32_t)MAX(ceilf((float)wide / size), 1.0f);
      redaction.grid[1] = (uint32_t)MAX(ceilf((float)tall / size), 1.0f);
    } else {
      redaction.color[3] = arrived;
      if (mode == SCREENWIDE_REDACT_PIXELATE && annotations->data.points != NULL &&
          (uint64_t)item->data_offset + item->data_count <= annotations->data.point_count) {
        // A pixelated box's zones, which a list whose side buffer does not
        // hold them all goes without.
        redaction.grid[0] = redaction_count(item->p3[0]);
        redaction.grid[1] = redaction_count(item->p3[1]);
        redaction.entry_count =
            redaction.grid[0] == 0 || redaction.grid[1] == 0 ? 0 : item->data_count;
        zones = annotations->data.points + item->data_offset;
      }
    }
    [redactions appendBytes:&redaction length:sizeof(redaction)];
    if (redaction.entry_count > 0)
      [redactions appendBytes:zones
                       length:(NSUInteger)redaction.entry_count * sizeof(float[2])];
  }
  // The spotlights' blur comes last, so it only softens what the redactions
  // left: a blur reaching into a box before the box was painted would carry
  // what the box hides out past its edge.
  append_spotlight_blur(redactions, annotations, width, height);
  return redactions;
}
