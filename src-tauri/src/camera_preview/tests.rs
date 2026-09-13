// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use nokhwa::utils::Resolution;

#[test]
fn preserves_mjpeg_frames_with_a_binary_header() {
  let frame = Buffer::new(
    Resolution::new(2, 1),
    &[0xff, 0xd8, 0xff, 0xd9],
    FrameFormat::MJPEG,
  );
  let payload = frame_payload(frame).unwrap();
  assert_eq!(&payload[..4], &2_u32.to_le_bytes());
  assert_eq!(&payload[4..8], &1_u32.to_le_bytes());
  assert_eq!(payload[8], 0);
  assert_eq!(&payload[9..], &[0xff, 0xd8, 0xff, 0xd9]);
}

#[test]
fn bounds_large_previews_without_changing_their_aspect() {
  assert_eq!(preview_dimensions(3_840, 2_160), (384, 216));
  assert_eq!(preview_dimensions(1_080, 1_920), (135, 240));
  assert_eq!(preview_dimensions(320, 180), (320, 180));
}

#[test]
fn stops_delivery_while_a_leaked_sender_is_still_alive() {
  let delivery =
    PreviewDelivery::spawn_with_sink(|_: InvokeResponseBody| Ok::<(), ()>(())).unwrap();
  // Stands in for the sender clone that a leaked nokhwa frame callback keeps
  // alive: the channel never disconnects, so only the cancel flag can end
  // the delivery thread.
  let leaked = delivery.sender();
  let (stopped_tx, stopped) = mpsc::channel();
  std::thread::spawn(move || {
    delivery.stop();
    let _ = stopped_tx.send(());
  });
  assert!(
    stopped.recv_timeout(Duration::from_secs(2)).is_ok(),
    "the delivery thread did not exit while a sender clone was alive"
  );
  drop(leaked);
}
