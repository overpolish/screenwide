// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const RETINA: DesktopDisplay = DesktopDisplay {
  id: 1,
  origin: Point { x: 0.0, y: 0.0 },
  size: Size {
    width: 1000.0,
    height: 600.0,
  },
  scale: 2.0,
};
const EXTERNAL: DesktopDisplay = DesktopDisplay {
  id: 2,
  origin: Point { x: 1000.0, y: 0.0 },
  size: Size {
    width: 1920.0,
    height: 1080.0,
  },
  scale: 1.0,
};

fn binding() -> DesktopBinding {
  DesktopBinding {
    displays: vec![RETINA, EXTERNAL],
    anchor_id: RETINA.id,
    size: Size {
      width: 2920.0,
      height: 1080.0,
    },
    layout_changed: false,
  }
}

#[test]
fn binding_resolves_the_display_containing_desktop_input() {
  let binding = binding();
  assert_eq!(binding.display_at(Point { x: 900.0, y: 500.0 }), Some(1));
  assert_eq!(
    binding.display_at(Point {
      x: 2500.0,
      y: 500.0,
    }),
    Some(2)
  );
  assert_eq!(
    binding.display_at(Point {
      x: 1100.0,
      y: -20.0,
    }),
    None
  );
}

#[test]
fn round_trips_through_mixed_scale_desktop_coordinates() {
  let local = Rect {
    origin: Point { x: 100.0, y: 50.0 },
    size: Size {
      width: 400.0,
      height: 300.0,
    },
  };
  assert_eq!(
    local_projection(RETINA, global_region(RETINA, local)),
    local
  );
}

#[test]
fn majority_overlap_changes_owner_and_ties_keep_the_current_one() {
  let mostly_external = Rect {
    origin: Point { x: 800.0, y: 100.0 },
    size: Size {
      width: 600.0,
      height: 300.0,
    },
  };
  assert_eq!(
    owner_for_region(&[RETINA, EXTERNAL], Some(RETINA.id), mostly_external).map(|d| d.id),
    Some(EXTERNAL.id)
  );
  let tied = Rect {
    origin: Point { x: 800.0, y: 100.0 },
    size: Size {
      width: 400.0,
      height: 300.0,
    },
  };
  assert_eq!(
    owner_for_region(&[RETINA, EXTERNAL], Some(RETINA.id), tied).map(|d| d.id),
    Some(RETINA.id)
  );
}

#[test]
fn projection_preserves_one_physical_region_across_display_windows() {
  let global = Rect {
    origin: Point { x: 800.0, y: 200.0 },
    size: Size {
      width: 500.0,
      height: 400.0,
    },
  };
  assert_eq!(local_projection(RETINA, global).origin.x, 800.0);
  assert_eq!(local_projection(EXTERNAL, global).origin.x, -200.0);
  assert_eq!(local_projection(RETINA, global).size.width, 500.0);
  assert_eq!(local_projection(EXTERNAL, global).size.width, 500.0);
}

#[test]
fn anchor_projection_preserves_cross_display_extents() {
  let global = Rect {
    origin: Point { x: 900.0, y: 100.0 },
    size: Size {
      width: 600.0,
      height: 400.0,
    },
  };
  assert_eq!(
    local_projection(RETINA, global),
    Rect {
      origin: Point { x: 900.0, y: 100.0 },
      size: Size {
        width: 600.0,
        height: 400.0,
      },
    }
  );
}

#[test]
fn topology_reconciliation_preserves_a_region_covered_across_a_seam() {
  let region = Rect {
    origin: Point { x: 800.0, y: 100.0 },
    size: Size {
      width: 600.0,
      height: 300.0,
    },
  };
  assert_eq!(
    reconcile_region(&[RETINA, EXTERNAL], Some(RETINA.id), region),
    Some((region, EXTERNAL))
  );
}

#[test]
fn topology_reconciliation_moves_a_partly_stranded_region_onto_its_owner() {
  let region = Rect {
    origin: Point { x: 700.0, y: 100.0 },
    size: Size {
      width: 600.0,
      height: 300.0,
    },
  };
  assert_eq!(
    reconcile_region(&[RETINA], Some(RETINA.id), region),
    Some((
      Rect {
        origin: Point { x: 400.0, y: 100.0 },
        size: region.size,
      },
      RETINA,
    ))
  );
}

#[test]
fn topology_reconciliation_uses_the_nearest_display_and_shrinks_if_needed() {
  let stranded = Rect {
    origin: Point {
      x: 3200.0,
      y: 100.0,
    },
    size: Size {
      width: 2200.0,
      height: 1200.0,
    },
  };
  assert_eq!(
    reconcile_region(&[RETINA, EXTERNAL], Some(RETINA.id), stranded),
    Some((
      Rect {
        origin: EXTERNAL.origin,
        size: EXTERNAL.size,
      },
      EXTERNAL,
    ))
  );
}
