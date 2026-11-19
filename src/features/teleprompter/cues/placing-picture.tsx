// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { MomentPicture } from "./picker/cue-moment";

/** From the pointer to the picture's corner, in points, so the picture
 * never covers the word being pointed at. */
const OFFSET = 16;

// The cue being placed, small, beside the pointer over the text, so it is
// plain what is being pinned; an empty card until its picture is drawn.
export function PlacingPicture({
  picture,
  x,
  y,
}: {
  /** A recording's moment, or a screenshot's still's address. */
  picture: ImageBitmap | string | null;
  /** Where the pointer is in the text's frame, in points. */
  x: number;
  y: number;
}) {
  return (
    <div
      className="pointer-events-none absolute z-20 w-40 overflow-hidden rounded-control bg-fill-strong shadow-lg"
      style={{
        aspectRatio:
          picture instanceof ImageBitmap
            ? picture.width / picture.height
            : "16 / 10",
        insetInlineStart: x + OFFSET,
        top: y + OFFSET,
      }}
    >
      {picture instanceof ImageBitmap ? (
        <MomentPicture picture={picture} />
      ) : picture ? (
        <img
          alt=""
          className="size-full object-contain"
          draggable={false}
          src={picture}
        />
      ) : null}
    </div>
  );
}
