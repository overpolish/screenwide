// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Run with clang -fobjc-arc -framework AppKit -framework Metal
// scripts/test-osc-pixel-alignment.m -o /tmp/osc-pixels && /tmp/osc-pixels
#import "../src-tauri/src/editor/osc_pixel_alignment_macos.h"
#include <assert.h>

static void quad(ScreenwideRegionOscVertex *out, uint32_t kind) {
  ScreenwideRegionOscPoint points[6] = {
    {-0.793f, 0.517f}, {0.281f, 0.517f}, {0.281f, -0.329f},
    {-0.793f, 0.517f}, {0.281f, -0.329f}, {-0.793f, -0.329f},
  };
  for (NSUInteger i = 0; i < 6; i++)
    out[i] = (ScreenwideRegionOscVertex){points[i], {0.2f, 0.8f}, kind, 0};
}

int main(void) {
  for (NSUInteger s = 0; s < 4; s++) {
    CGFloat scale = (CGFloat[]){1.0, 1.25, 1.5, 2.0}[s];
    NSSize size = NSMakeSize(200, 100);
    ScreenwideRegionOscVertex vertices[6], original[6];
    quad(vertices, 11);
    memcpy(original, vertices, sizeof(vertices));
    screenwide_osc_align_vertices(vertices, 6, size, scale);
    for (NSUInteger i = 0; i < 6; i++) {
      double x = (vertices[i].position.x + 1.0) * 100 * scale;
      double y = (1.0 - vertices[i].position.y) * 50 * scale;
      assert(fabs(x - round(x)) < 0.0001);
      assert(fabs(y - round(y)) < 0.0001);
      assert(memcmp(&vertices[i].uv, &original[i].uv, sizeof(vertices[i].uv)) == 0);
    }
    memcpy(original, vertices, sizeof(vertices));
    screenwide_osc_align_vertices(vertices, 6, size, scale);
    assert(memcmp(vertices, original, sizeof(vertices)) == 0);
    // A thin axis-aligned border must not disappear during snapping.
    quad(vertices, 28);
    for (NSUInteger i = 0; i < 6; i++)
      vertices[i].position.y = vertices[i].position.y > 0 ? 0.111f : 0.110f;
    screenwide_osc_align_vertices(vertices, 6, size, scale);
    double thickness = (vertices[0].position.y - vertices[2].position.y) * 50 * scale;
    assert(fabs(thickness - 1.0) < 0.0001);
    // A sloping stroke must keep its shape, and snapshots their transform.
    quad(vertices, 28);
    vertices[1].position.y += 0.1f;
    memcpy(original, vertices, sizeof(vertices));
    screenwide_osc_align_vertices(vertices, 6, size, scale);
    assert(memcmp(vertices, original, sizeof(vertices)) == 0);
    quad(vertices, 33);
    memcpy(original, vertices, sizeof(vertices));
    screenwide_osc_align_vertices(vertices, 6, size, scale);
    assert(memcmp(vertices, original, sizeof(vertices)) == 0);
  }
  puts("OSC pixel alignment checks passed (1x, 1.25x, 1.5x, 2x).");
}
