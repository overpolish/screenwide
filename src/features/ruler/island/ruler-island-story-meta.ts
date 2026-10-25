// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createElement } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";

import { RulerIsland } from "./ruler-island";
import {
  PREVIEW_RULER_STATE,
  PREVIEW_STAMP_KEYS,
} from "./ruler-island-preview";
import { RulerIslandStage } from "./ruler-island-stage";

import type { Meta } from "@storybook/react-vite";

// What every Ruler island story shares: the stage, its events, and the notch
// from the toolbar, so one switch shows every story with and without it.

export const rulerIslandMeta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isShown: true,
    onAction: fn(),
    onClose: fn(),
    onTolerance: fn(),
    onTool: fn(),
    stampKeys: PREVIEW_STAMP_KEYS,
    state: PREVIEW_RULER_STATE,
  },
  component: RulerIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch === "off" ? null : PREVIEW_NOTCH,
        },
      }),
    featureStage({ height: 320, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) => createElement(RulerIslandStage, args),
} satisfies Meta<typeof RulerIsland>;
