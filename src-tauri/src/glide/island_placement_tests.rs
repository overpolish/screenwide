// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::contained_origin;

#[test]
fn preview_is_contained_at_each_edge() {
  let area_origin = (-1_920.0, 40.0);
  let area_size = (1_920.0, 1_040.0);
  let preview = (72.0, 48.0);

  assert_eq!(
    contained_origin((-1_956.0, 16.0), preview, 0.0, area_origin, area_size),
    (-1_920.0, 40.0),
  );
  assert_eq!(
    contained_origin((-20.0, 1_060.0), preview, 0.0, area_origin, area_size),
    (-72.0, 1_032.0),
  );
}

/// A gesture on a title bar near the top of the screen: the transparent room
/// round the island may reach past the edge, so the island stays centred on
/// the anchor instead of being pushed down by the whole window.
#[test]
fn only_the_island_inside_its_room_is_contained() {
  let screen = ((0.0, 0.0), (1_512.0, 982.0));
  let window = (212.0, 173.0);
  let room = 48.0;
  let centred = (700.0 - 106.0, 60.0 - 86.5);

  assert_eq!(
    contained_origin(centred, window, room, screen.0, screen.1),
    centred,
  );
  // Nearer still, the island itself would leave the screen: it stops at the
  // edge, its room past it.
  assert_eq!(
    contained_origin((594.0, -80.0), window, room, screen.0, screen.1),
    (594.0, -room),
  );
}

#[test]
fn preview_larger_than_its_area_pins_to_the_area_origin() {
  assert_eq!(
    contained_origin(
      (40.0, 50.0),
      (400.0, 300.0),
      0.0,
      (100.0, 80.0),
      (200.0, 100.0),
    ),
    (100.0, 80.0),
  );
}
