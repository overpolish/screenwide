// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import tauriConfig from "../../src-tauri/tauri.conf.json";

import type { Args, Decorator, StoryContext } from "@storybook/react-vite";

/**
 * A window from `tauri.conf.json`, by label, or the size of anything else.
 * Omit `height` when the content sets it.
 */
export type StageSize = { window: string } | { width: number; height?: number };

type Frame = {
  width: number;
  height?: number;
  minHeight?: number;
  minWidth?: number;
};

// JSON imports type each window by its own keys; this is the shape they share.
const windows: (Partial<Frame> & { label: string })[] = tauriConfig.app.windows;

const configuredWindow = (label: string): Frame => {
  const window = windows.find((candidate) => candidate.label === label);
  if (window?.width === undefined || window.height === undefined) {
    throw new Error(`tauri.conf.json has no sized window labelled "${label}"`);
  }
  // A window with a minimum is resizable: the stage lets it fill the canvas.
  return {
    height: window.height,
    minHeight: window.minHeight,
    minWidth: window.minWidth,
    width: window.width,
  };
};

const frameOf = (size: StageSize): Frame =>
  "window" in size ? configuredWindow(size.window) : size;

/**
 * Frames a feature at its production size. The canvas scrolls instead of
 * squeezing it; a resizable window fills the canvas down to its minimum size.
 * Docs pages show every window at its default size. A function may pick the
 * size from the story's args.
 */
export function featureStage<TArgs = Args>(
  size: StageSize | ((context: StoryContext<TArgs>) => StageSize),
): Decorator<TArgs> {
  return (Story, context) => {
    const { height, minHeight, minWidth, width } = frameOf(
      typeof size === "function" ? size(context) : size,
    );

    if (context.viewMode === "docs") {
      return (
        // Centred, as on the canvas; a frame wider than the column starts at
        // its edge and scrolls instead, since an auto margin does not go
        // below zero.
        <div className="max-w-full overflow-auto p-6">
          <div className="mx-auto shrink-0" style={{ height, width }}>
            <Story />
          </div>
        </div>
      );
    }

    if (minWidth !== undefined) {
      return (
        <div className="fixed inset-0 overflow-auto">
          <div className="size-full" style={{ minHeight, minWidth }}>
            <Story />
          </div>
        </div>
      );
    }

    return (
      <div className="fixed inset-0 overflow-auto">
        <div className="box-border flex h-max min-h-full w-max min-w-full items-center justify-center p-6">
          <div className="shrink-0" style={{ height, width }}>
            <Story />
          </div>
        </div>
      </div>
    );
  };
}
