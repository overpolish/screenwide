// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fullSourceRect } from "./screenshot-geometry";
import { screenshotLayout } from "./screenshot-layout";
import {
  defaultScreenshotOutput,
  normalizedScreenshotOutput,
  ScreenshotOutputSettings,
  screenshotSourceCrop,
  withScreenshotSourceCrop,
} from "./screenshot-output-settings";

export { screenshotLayout } from "./screenshot-layout";
export type { ScreenshotLayout } from "./screenshot-layout";
export { defaultScreenshotOutput, normalizedScreenshotOutput };
export type { ScreenshotOutputSettings };

type ScreenshotWorkspaceItemOutput = {
  id: number;
  output: ScreenshotOutputSettings;
};

export type ScreenshotWorkspaceOutputSettings = ScreenshotOutputSettings & {
  items: ScreenshotWorkspaceItemOutput[];
};

export const normalizedScreenshotWorkspaceOutput = (
  settings: ScreenshotWorkspaceOutputSettings,
): ScreenshotWorkspaceOutputSettings => ({
  ...normalizedScreenshotOutput(settings),
  items: settings.items.map((item) => ({
    id: item.id,
    output: normalizedScreenshotOutput(item.output),
  })),
});

export const screenshotWorkspaceItemOutput = (
  settings: ScreenshotWorkspaceOutputSettings,
  id: number,
) => {
  const output = settings.items.find((item) => item.id === id)?.output;
  if (!output) return settings;
  return {
    ...output,
    backgroundColor: settings.backgroundColor,
    backgroundRadiusPercent: settings.backgroundRadiusPercent,
    backgroundType: settings.backgroundType,
    height: settings.height,
    meshColors: settings.meshColors,
    meshLockedColors: settings.meshLockedColors,
    meshPoints: settings.meshPoints,
    meshSeed: settings.meshSeed,
    meshWarpPercent: settings.meshWarpPercent,
    width: settings.width,
  };
};

export type RecordingOutputSettings = Record<
  "camera" | "primary",
  ScreenshotOutputSettings
> & {
  /** Front-to-back order for the two video layers. */
  cameraOnTop: boolean;
};

export const recordingVideoTrackOrder = (settings: RecordingOutputSettings) =>
  settings.cameraOnTop
    ? (["camera", "primary"] as const)
    : (["primary", "camera"] as const);

export const screenshotOutputDimensions = (
  settings: ScreenshotOutputSettings,
) => ({
  height: Math.max(1, Math.round(settings.height)),
  width: Math.max(1, Math.round(settings.width)),
});

const screenshotPlacement = (
  source: { height: number; width: number },
  output: { height: number; width: number },
) => {
  const scale = Math.min(
    output.width / source.width,
    output.height / source.height,
  );
  const width = Math.max(1, source.width * scale);
  const height = Math.max(1, source.height * scale);
  return {
    height,
    width,
    x: (output.width - width) / 2,
    y: (output.height - height) / 2,
  };
};

export type ScreenshotCanvasBounds = {
  height: number;
  originX: number;
  originY: number;
  width: number;
};

const CENTERED_RESIZE_EDGE = 1 << 16;
const MAXIMUM_CANVAS_PIXELS = 120_000_000;
const MINIMUM_CANVAS_SIZE = 64;

/**
 * Move a canvas's origin, carrying everything placed in it.
 *
 * Placement is measured from the canvas's top left, so whenever that corner
 * moves, as when the near edges are dragged or the canvas grows around its
 * items, every layer is renumbered by the same amount and stays put on
 * screen. A resize of the far edges leaves the origin, and placement, alone.
 */
const rebaseScreenshotCanvas = (
  settings: ScreenshotOutputSettings,
  bounds: ScreenshotCanvasBounds,
): ScreenshotOutputSettings => ({
  ...settings,
  cropX: settings.cropX - bounds.originX,
  cropY: settings.cropY - bounds.originY,
  height: Math.max(1, Math.round(bounds.height)),
  imageX: settings.imageX - bounds.originX,
  imageY: settings.imageY - bounds.originY,
  width: Math.max(1, Math.round(bounds.width)),
});

/** Resize the output canvas, leaving every layer exactly where it is. */
export const resizeScreenshotCanvas = ({
  height,
  settings,
  width,
}: {
  height: number;
  settings: ScreenshotOutputSettings;
  width: number;
}): ScreenshotOutputSettings => ({
  ...settings,
  height: Math.max(1, Math.round(height)),
  width: Math.max(1, Math.round(width)),
});

/** Resize a workspace canvas, leaving every layer exactly where it is. */
export const resizeScreenshotWorkspaceCanvas = ({
  height,
  settings,
  width,
}: {
  height: number;
  settings: ScreenshotWorkspaceOutputSettings;
  width: number;
}): ScreenshotWorkspaceOutputSettings => {
  const nextWidth = Math.max(1, Math.round(width));
  const nextHeight = Math.max(1, Math.round(height));
  return {
    ...settings,
    height: nextHeight,
    items: settings.items.map((item) => ({
      ...item,
      output: {
        ...item.output,
        height: nextHeight,
        width: nextWidth,
      },
    })),
    width: nextWidth,
  };
};

/**
 * Resize a workspace canvas from a native frame-handle gesture.
 *
 * The gesture only ever says how big the canvas should now be. What is in it
 * keeps the pixels it was placed at, so growing the canvas opens empty space
 * at the right and the bottom and shrinking it clips whatever falls outside.
 */
export const resizeScreenshotWorkspaceCanvasEdges = ({
  deltaX,
  deltaY,
  edges: encodedEdges,
  settings,
}: {
  deltaX: number;
  deltaY: number;
  edges: number;
  settings: ScreenshotWorkspaceOutputSettings;
}): ScreenshotWorkspaceOutputSettings => {
  const centered = (encodedEdges & CENTERED_RESIZE_EDGE) !== 0;
  const edges = encodedEdges & ~CENTERED_RESIZE_EDGE;
  const resizeAxis = (
    size: number,
    delta: number,
    edge: { far: boolean; near: boolean },
  ) => {
    if (edge.near) {
      const movement = Math.min(
        centered
          ? (size - MINIMUM_CANVAS_SIZE) / 2
          : size - MINIMUM_CANVAS_SIZE,
        delta,
      );
      return centered ? size - movement * 2 : size - movement;
    }
    if (edge.far) {
      const movement = Math.max(
        centered
          ? -(size - MINIMUM_CANVAS_SIZE) / 2
          : MINIMUM_CANVAS_SIZE - size,
        delta,
      );
      return centered ? size + movement * 2 : size + movement;
    }
    return size;
  };
  const startWidth = Math.max(1, settings.width);
  const startHeight = Math.max(1, settings.height);
  const horizontalActive = (edges & 3) !== 0;
  const verticalActive = (edges & 12) !== 0;
  let width = Math.max(
    MINIMUM_CANVAS_SIZE,
    resizeAxis(startWidth, deltaX * startWidth, {
      far: (edges & 2) !== 0,
      near: (edges & 1) !== 0,
    }),
  );
  let height = Math.max(
    MINIMUM_CANVAS_SIZE,
    resizeAxis(startHeight, deltaY * startHeight, {
      far: (edges & 8) !== 0,
      near: (edges & 4) !== 0,
    }),
  );
  if (width * height > MAXIMUM_CANVAS_PIXELS) {
    if (horizontalActive && verticalActive) {
      const factor = Math.sqrt(MAXIMUM_CANVAS_PIXELS / (width * height));
      width = Math.floor(width * factor);
      height = Math.floor(MAXIMUM_CANVAS_PIXELS / width);
    } else if (horizontalActive) {
      width = Math.floor(MAXIMUM_CANVAS_PIXELS / height);
    } else if (verticalActive) {
      height = Math.floor(MAXIMUM_CANVAS_PIXELS / width);
    }
  }
  // The drag moved the near edges by what the size did not absorb at the
  // far ones: the origin shifts by that, and the layers shift with it so
  // they hold their place on screen, as they did while the drag was live.
  const horizontalNear = (edges & 1) !== 0;
  const verticalNear = (edges & 4) !== 0;
  const originX = centered
    ? (startWidth - width) / 2
    : horizontalNear
      ? startWidth - width
      : 0;
  const originY = centered
    ? (startHeight - height) / 2
    : verticalNear
      ? startHeight - height
      : 0;
  const bounds = { height, originX, originY, width };
  return {
    ...rebaseScreenshotCanvas(settings, bounds),
    items: settings.items.map((item) => ({
      id: item.id,
      output: rebaseScreenshotCanvas(item.output, bounds),
    })),
  };
};

/** Grow around visible items, using the gesture-start canvas as the floor. */
export const fitScreenshotWorkspaceToItems = ({
  initial,
  movedItemId,
  movedItemOutput,
}: {
  initial: ScreenshotWorkspaceOutputSettings;
  movedItemId: number;
  movedItemOutput: ScreenshotOutputSettings;
}): {
  bounds: ScreenshotCanvasBounds;
  movedItemOutput: ScreenshotOutputSettings;
  output: ScreenshotWorkspaceOutputSettings;
} => {
  const initialSize = screenshotOutputDimensions(initial);
  const movedItems = initial.items.map((item) => ({
    ...item,
    output:
      item.id === movedItemId
        ? movedItemOutput
        : screenshotWorkspaceItemOutput(initial, item.id),
  }));
  let left = 0;
  let top = 0;
  let right = initialSize.width;
  let bottom = initialSize.height;
  for (const item of movedItems) {
    const { output } = item;
    left = Math.min(left, Math.floor(output.cropX));
    top = Math.min(top, Math.floor(output.cropY));
    right = Math.max(right, Math.ceil(output.cropX + output.cropWidth));
    bottom = Math.max(bottom, Math.ceil(output.cropY + output.cropHeight));
  }
  const bounds = {
    height: bottom - top,
    originX: left,
    originY: top,
    width: right - left,
  };
  const items = movedItems.map((item) => ({
    ...item,
    output: rebaseScreenshotCanvas(item.output, bounds),
  }));
  const output = {
    ...initial,
    height: bounds.height,
    items,
    width: bounds.width,
  };
  return {
    bounds,
    movedItemOutput:
      items.find((item) => item.id === movedItemId)?.output ?? movedItemOutput,
    output,
  };
};

/** The image at its own size, centred, however that sits against the
 * canvas: a capture placed into an open workspace is not shrunk to fit. */
const naturalPlacement = (
  source: { height: number; width: number },
  output: { height: number; width: number },
) => ({
  height: Math.max(1, source.height),
  width: Math.max(1, source.width),
  x: (output.width - source.width) / 2,
  y: (output.height - source.height) / 2,
});

export const resetScreenshotLayout = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
  { fit = true }: { fit?: boolean } = {},
): ScreenshotOutputSettings => {
  const output = screenshotOutputDimensions(settings);
  const placement = fit
    ? screenshotPlacement(source, output)
    : naturalPlacement(source, output);
  return withScreenshotSourceCrop(
    {
      ...settings,
      cropHeight: placement.height,
      cropWidth: placement.width,
      cropX: placement.x,
      cropY: placement.y,
      imageWidth: placement.width,
      imageX: placement.x,
      imageY: placement.y,
      recenterInsetColor: null,
    },
    fullSourceRect(),
  );
};

/** Reset only the visible crop, retaining the image's scale and position. */
export const resetScreenshotCrop = (
  settings: ScreenshotOutputSettings,
  _source: { height: number; width: number },
): ScreenshotOutputSettings =>
  withScreenshotSourceCrop({ ...settings, radiusPercent: 0 }, fullSourceRect());

/** Reset the selected item's scale and position while retaining its crop:
 * back to its real size, centred, as it was placed. */
export const resetScreenshotTransform = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): ScreenshotOutputSettings => {
  const output = screenshotOutputDimensions(settings);
  const current = screenshotLayout(source, settings);
  // The crop's real size is its share of the source's own pixels.
  const cropSource = screenshotSourceCrop(settings);
  const target = naturalPlacement(
    {
      height: source.height * cropSource.height,
      width: source.width * cropSource.width,
    },
    output,
  );
  const scale = target.width / Math.max(1, current.crop.width);
  return {
    ...settings,
    cropHeight: target.height,
    cropWidth: target.width,
    cropX: target.x,
    cropY: target.y,
    imageWidth: current.image.width * scale,
    imageX: target.x + (current.image.x - current.crop.x) * scale,
    imageY: target.y + (current.image.y - current.crop.y) * scale,
  };
};

export const defaultRecordingOutput = ({
  camera,
  primary,
}: {
  primary: { height: number; width: number };
  camera?: { height: number; width: number } | null;
}): RecordingOutputSettings => ({
  camera: resetScreenshotLayout(
    defaultScreenshotOutput(camera?.width ?? 1, camera?.height ?? 1),
    camera ?? { height: 1, width: 1 },
  ),
  cameraOnTop: true,
  primary: resetScreenshotLayout(
    defaultScreenshotOutput(primary.width, primary.height),
    primary,
  ),
});

export const restoredRecordingOutput = ({
  camera,
  persisted,
  primary,
}: {
  primary: { height: number; width: number };
  camera?: { height: number; width: number } | null;
  persisted?: Partial<RecordingOutputSettings> | null;
}): RecordingOutputSettings => {
  const defaults = defaultRecordingOutput({ camera, primary });
  if (!persisted) return defaults;
  const restore = (
    key: "camera" | "primary",
    source: { height: number; width: number },
  ) =>
    resetScreenshotLayout(
      {
        ...defaults[key],
        ...persisted[key],
        backgroundRadiusPercent: 0,
        height: defaults[key].height,
        recenterInsetColor: null,
        width: defaults[key].width,
      },
      source,
    );
  return {
    camera: restore("camera", camera ?? { height: 1, width: 1 }),
    // Layer order belongs to the current recording, not the previous export.
    // A fresh recording always starts with its camera above the screen.
    cameraOnTop: true,
    primary: restore("primary", primary),
  };
};
