// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { BAR_PX } from "../bar-stack/stack-lanes";
import { baseOf } from "../strip/strip-part";

import {
  AnnotationLane,
  type AnnotationLaneProps,
  type TimelineAnnotation,
} from "./annotation-lane";
import {
  INCOMING_PICTURE,
  isPlacingPicture,
  onPictureCarry,
  setPictureLanding,
  takePlacedPicture,
} from "./picture-carry";

// The annotation lanes, shown while they hold annotations or a picture is
// carried over the editor, which they take as a bar of its own the moment
// it reaches them: it follows the pointer in time and up and down the
// lanes, the others gliding round it, as a bar picked up does. Its length
// is what a placed image starts with. Let go, the image is placed from that
// moment (`use-picture-drop.ts`) and, once on the strip, moved into the
// lane it was let go in.

/** How long a placed image shows at first, in s, as the editor places it
 * (`provisional_clips`). */
const PLACED_S = 3;

/** How far, in px, past the lanes' top and bottom a picture still lands
 * in them. */
const REACH_PX = BAR_PX / 2;

export function AnnotationDropLane(props: AnnotationLaneProps) {
  const boxRef = useRef<HTMLDivElement>(null);
  const [incoming, setIncoming] = useState<{
    bar: TimelineAnnotation;
    y: number;
  } | null>(null);
  const atMsRef = useRef(0);
  const { annotations, onRestack, scale } = props;
  const scaleRef = useRef(scale);
  scaleRef.current = scale;
  // Measured as the picture moves, against where the lanes are then.
  useEffect(
    () =>
      onPictureCarry((point) => {
        const box = boxRef.current?.getBoundingClientRect();
        if (
          !point ||
          !box ||
          box.width === 0 ||
          point.x < box.left ||
          point.x > box.right ||
          point.y < box.top - REACH_PX ||
          point.y > box.bottom + REACH_PX
        ) {
          setIncoming(null);
          return;
        }
        const shown = scaleRef.current;
        const start = shown.toSeconds((point.x - box.left) / box.width);
        atMsRef.current = shown.map.toPlayed(start) * 1_000;
        setIncoming({
          bar: {
            end: start + PLACED_S,
            id: INCOMING_PICTURE,
            kind: "image",
            lane: 0,
            start,
          },
          y: point.y - box.top,
        });
      }),
    [],
  );
  // An image placed from a picture let go here arrives as the one id not
  // on the lanes before, and goes into the lane the picture was let go in.
  const knownRef = useRef(new Set(annotations.map(({ id }) => baseOf(id))));
  useEffect(() => {
    const known = knownRef.current;
    const ids = annotations.map(({ id }) => baseOf(id));
    knownRef.current = new Set(ids);
    const arrived = ids.filter((id) => !known.has(id));
    if (arrived.length !== 1 || !isPlacingPicture()) return;
    const lanes = takePlacedPicture();
    if (!lanes) return;
    const { [INCOMING_PICTURE]: lane, ...others } = lanes;
    onRestack({
      ...others,
      ...Object.fromEntries(
        annotations
          .filter(({ id }) => baseOf(id) === arrived[0])
          .map(({ id }) => [id, lane]),
      ),
    });
  }, [annotations, onRestack]);
  return (
    <div ref={boxRef}>
      <AnnotationLane
        {...props}
        incoming={incoming}
        onIncomingLanes={(lanes) => {
          setPictureLanding(lanes ? { atMs: atMsRef.current, lanes } : null);
        }}
      />
    </div>
  );
}
