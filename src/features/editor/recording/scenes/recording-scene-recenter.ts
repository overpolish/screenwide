// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { sourceRect, SourceRect } from "../../screenshot/screenshot-geometry";
import { ScreenshotLayout } from "../../screenshot/screenshot-layout";
import {
  ScreenshotContentBounds,
  getRecordingRecenterAnalysis,
} from "../../screenshot/screenshot-recenter";

import { customRecordingSceneClip } from "./recording-scene-custom";
import { croppedCustomPane, pictureRect } from "./recording-scene-custom-crop";
import { SceneRect } from "./recording-scene-geometry";
import { scenePaneFraming } from "./recording-scene-pane-framing";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip, SceneBox, SceneFraming } from "./recording-scenes";

import type { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

/**
 * Recenter over a scene, as the recording's own Recenter does outside every
 * scene: the content found in what the scene's screen shows is cropped to
 * and put in the middle of where its box was, the picture moving with it.
 * The recording's own Recenter pulls the content to the middle of its
 * padded crop; a scene pads its box outwards (`inset`), so its box becomes
 * the content alone and the padding stays round it.
 */

/** A pane of a scene: its picture's aspect, its box in output pixels, and
 * the framing that places its picture there. */
type ScenePane = { aspect: number; box: SceneRect; framing: SceneFraming };

type Size = { height: number; width: number };

/** How the recording's own crop, which a scene shows as its screen's
 * picture, is placed by `pane`: where its corner lands and how much larger
 * it is drawn. */
const pictureScale = (pane: ScenePane, layout: ScreenshotLayout) => {
  const picture = pictureRect(pane);
  return {
    picture,
    scale: picture.width / Math.max(Number.EPSILON, layout.crop.width),
  };
};

/**
 * What of the recording's source `pane`'s box shows, as shares of the
 * source, cut to the recording's own crop of it: the part Recenter looks for
 * content in. Null where the box shows none of the source.
 */
export const sceneWindowSourceCrop = (
  pane: ScenePane,
  layout: ScreenshotLayout,
): SourceRect | null => {
  const { picture, scale } = pictureScale(pane, layout);
  const { box } = pane;
  const left = Math.max(
    layout.crop.x + (box.x - picture.x) / scale,
    layout.sourceCrop.x,
  );
  const top = Math.max(
    layout.crop.y + (box.y - picture.y) / scale,
    layout.sourceCrop.y,
  );
  const right = Math.min(
    layout.crop.x + (box.x + box.width - picture.x) / scale,
    layout.sourceCrop.x + layout.sourceCrop.width,
  );
  const bottom = Math.min(
    layout.crop.y + (box.y + box.height - picture.y) / scale,
    layout.sourceCrop.y + layout.sourceCrop.height,
  );
  if (right <= left || bottom <= top) return null;
  const { image } = layout;
  return sourceRect({
    height: (bottom - top) / image.height,
    width: (right - left) / image.width,
    x: (left - image.x) / image.width,
    y: (top - image.y) / image.height,
  });
};

/**
 * `pane` cropped to `content`, bounds found in the source in its pixels,
 * moved so the content sits in the middle of where the box was, the picture
 * moving with it; held to the limits every crop keeps (`croppedCustomPane`).
 */
export const recenteredScenePane = (
  pane: ScenePane,
  {
    canvas,
    content,
    layout,
    source,
  }: {
    canvas: Size;
    content: ScreenshotContentBounds;
    layout: ScreenshotLayout;
    source: Size;
  },
): { box: SceneBox; framing: SceneFraming } => {
  const { picture, scale } = pictureScale(pane, layout);
  const { image } = layout;
  // The content in the recording's output pixels, then on the canvas.
  const across = image.width / Math.max(1, source.width);
  const down = image.height / Math.max(1, source.height);
  const found = {
    height: content.height * down * scale,
    width: content.width * across * scale,
    x: picture.x + (image.x + content.x * across - layout.crop.x) * scale,
    y: picture.y + (image.y + content.y * down - layout.crop.y) * scale,
  };
  const { box } = pane;
  const shift = {
    x: box.x + box.width / 2 - (found.x + found.width / 2),
    y: box.y + box.height / 2 - (found.y + found.height / 2),
  };
  return croppedCustomPane(
    { ...pane, box: { ...box, x: box.x + shift.x, y: box.y + shift.y } },
    {
      canvas,
      moving: 0,
      next: { ...found, x: found.x + shift.x, y: found.y + shift.y },
      slides: true,
    },
  );
};

/**
 * Recenter on the scene `clipId` names: its screen's window is read for
 * content on the frame `positionMs` into the recording, and the clips the
 * analysis lands on, read then from `latest`, are handed to `commit` with
 * that scene cropped to it, made custom where it was a preset's. Nothing is
 * committed where no content is found or the scene has gone.
 */
export const recenterSceneScreen = ({
  artifactId,
  camera,
  clipId,
  commit,
  latest,
  layout,
  output,
  positionMs,
  source,
}: {
  artifactId: number;
  camera: SceneCamera | null;
  clipId: string;
  commit: (clips: RecordingSceneClip[]) => void;
  latest: () => RecordingSceneClip[];
  /** The recording's own layout of its screen, whose crop a scene shows. */
  layout: ScreenshotLayout;
  output: ScreenshotOutputSettings;
  positionMs: number;
  source: Size;
}) => {
  const paneOf = (clip: RecordingSceneClip) =>
    scenePaneFraming({ camera, clip, output, pane: "screen" });
  const clip = latest().find(({ id }) => id === clipId);
  const pane = clip ? paneOf(clip) : null;
  const window = pane ? sceneWindowSourceCrop(pane, layout) : null;
  if (!window) return;
  getRecordingRecenterAnalysis(artifactId, positionMs, window)
    .then((analysis) => {
      const clips = latest();
      const still = clips.find(({ id }) => id === clipId);
      const now = still ? paneOf(still) : null;
      if (!analysis?.bounds || !still || !now) return;
      const centred = recenteredScenePane(now, {
        canvas: { height: output.height, width: output.width },
        content: analysis.bounds,
        layout,
        source,
      });
      const owned = customRecordingSceneClip({ camera, clip: still, output });
      const { boxes } = owned;
      if (!boxes) return;
      commit(
        clips.map((each) =>
          each.id === clipId
            ? {
                ...owned,
                boxes: { ...boxes, screen: centred.box },
                screen: centred.framing,
              }
            : each,
        ),
      );
    })
    .catch((error: unknown) => {
      console.error("Could not detect the scene's content bounds", error);
    });
};
