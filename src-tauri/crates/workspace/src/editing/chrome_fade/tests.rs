// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn chrome(x: f32, kinds: &[u32]) -> Vec<Vertex> {
  kinds
    .iter()
    .map(|&kind| Vertex {
      position: [x, 0.0],
      uv: [0.0, 0.0],
      kind,
      fade: 0,
    })
    .collect()
}

fn shown(vertex: &Vertex) -> f32 {
  1.0 - vertex.fade as f32 / FADED_OUT as f32
}

/// One draw of `vertices` keyed `key`, where motion is not reduced.
fn draw(
  fade: &mut ChromeFade,
  key: u64,
  vertices: Vec<Vertex>,
  rings: Option<turn::Rings>,
  held: bool,
  at: Instant,
) -> (Vec<Vertex>, Option<turn::Rings>) {
  fade.ease(key, vertices, rings, held, || true, at)
}

const HALFWAY: Duration = Duration::from_millis(100);

#[test]
fn chrome_drawn_alike_glides_onto_its_new_subject() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  draw(&mut fade, 1, chrome(0.0, &[2, 3]), None, false, start);
  let (halfway, _) = draw(&mut fade, 2, chrome(1.0, &[2, 3]), None, false, start);
  assert_eq!(halfway[0].position[0], 0.0);
  let (halfway, _) = draw(
    &mut fade,
    2,
    chrome(1.0, &[2, 3]),
    None,
    false,
    start + HALFWAY,
  );
  assert_eq!(halfway.len(), 2);
  assert!(halfway[0].position[0] > 0.5 && halfway[0].position[0] < 1.0);
  assert_eq!(shown(&halfway[0]), 1.0);
  let (done, _) = draw(
    &mut fade,
    2,
    chrome(1.0, &[2, 3]),
    None,
    false,
    start + CHANGE,
  );
  assert_eq!(done, chrome(1.0, &[2, 3]));
  assert!(fade.is_settled());
}

#[test]
fn chrome_drawn_differently_crossfades() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  draw(&mut fade, 1, chrome(0.0, &[2, 3]), None, false, start);
  draw(&mut fade, 2, chrome(1.0, &[8]), None, false, start);
  let (halfway, _) = draw(
    &mut fade,
    2,
    chrome(1.0, &[8]),
    None,
    false,
    start + HALFWAY,
  );
  assert_eq!(halfway.len(), 3);
  let (leaving, coming) = halfway.split_at(2);
  assert!(leaving.iter().all(|vertex| vertex.position[0] == 0.0));
  let (gone, here) = (shown(&leaving[0]), shown(&coming[0]));
  assert!(gone > 0.0 && gone < 1.0 && (gone + here - 1.0).abs() < 0.01);
}

#[test]
fn chrome_coming_in_shows_little_at_first() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  draw(&mut fade, 1, chrome(0.0, &[2, 3]), None, false, start);
  draw(&mut fade, 2, chrome(1.0, &[8]), None, true, start);
  let (first, _) = draw(&mut fade, 2, chrome(1.0, &[8]), None, true, start + FRAME);
  assert!(shown(&first[2]) < 0.1);
}

#[test]
fn a_press_takes_the_chrome_where_it_is() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  draw(&mut fade, 1, chrome(0.0, &[2, 3]), None, false, start);
  draw(&mut fade, 2, chrome(1.0, &[2, 3]), None, true, start);
  let (halfway, _) = draw(
    &mut fade,
    2,
    chrome(1.0, &[2, 3]),
    None,
    true,
    start + HALFWAY,
  );
  assert_eq!(halfway.len(), 4);
  assert!(halfway[2..].iter().all(|vertex| vertex.position[0] == 1.0));
}

#[test]
fn less_motion_crossfades_what_would_glide() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  fade.ease(1, chrome(0.0, &[2, 3]), None, false, || false, start);
  fade.ease(2, chrome(1.0, &[2, 3]), None, false, || false, start);
  let (halfway, _) = fade.ease(
    2,
    chrome(1.0, &[2, 3]),
    None,
    false,
    || false,
    start + HALFWAY,
  );
  assert_eq!(halfway.len(), 4);
  assert!(halfway
    .iter()
    .all(|vertex| vertex.position[0] == 0.0 || vertex.position[0] == 1.0));
}

#[test]
fn chrome_shown_at_first_and_kept_does_not_change_over() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  let (first, _) = draw(&mut fade, 1, chrome(0.0, &[2]), None, false, start);
  assert_eq!(first, chrome(0.0, &[2]));
  let (moved, _) = draw(&mut fade, 1, chrome(5.0, &[2, 3]), None, false, start);
  assert_eq!(moved, chrome(5.0, &[2, 3]));
  assert!(fade.is_settled());
}

#[test]
fn rings_fade_out_where_they_were() {
  let start = Instant::now();
  let mut fade = ChromeFade::default();
  let mut rings = turn::Rings {
    rows: [[1.0; 4]; screenwide_osc::gpu::pipeline::TURN_RING_ROWS],
    quad: [0.0, 0.0, 10.0, 10.0],
  };
  rings.rows[7][3] = 1.0;
  draw(&mut fade, 1, Vec::new(), Some(rings), false, start);
  let shown_at = start + CHANGE;
  let (_, shown) = draw(&mut fade, 1, Vec::new(), Some(rings), false, shown_at);
  assert_eq!(shown.map(|rings| rings.rows[7][3]), Some(1.0));
  draw(&mut fade, 1, Vec::new(), None, false, shown_at);
  let (_, halfway) = draw(&mut fade, 1, Vec::new(), None, false, shown_at + HALFWAY);
  let halfway = halfway.expect("the rings keep drawing as they fade");
  assert_eq!(halfway.quad, rings.quad);
  assert!(halfway.rows[7][3] > 0.0 && halfway.rows[7][3] < 1.0);
  let (_, gone) = draw(&mut fade, 1, Vec::new(), None, false, shown_at + CHANGE);
  assert!(gone.is_none());
  assert!(fade.is_settled());
}
