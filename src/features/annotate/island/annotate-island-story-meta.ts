// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createElement } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";

import { AnnotateIsland } from "./annotate-island";
import { AnnotateIslandStage } from "./annotate-island-stage";
import {
  PREVIEW_ANNOTATE_SETTINGS,
  PREVIEW_SAVED_COLORS,
} from "./annotate-preview";

import type { Meta } from "@storybook/react-vite";

// What every annotate island story shares: the stage, its events, and the
// notch from the toolbar, so one switch shows every story with and without it.

export const annotateIslandMeta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isShown: true,
    onChange: fn(),
    onClear: fn(),
    onDone: fn(),
    onUndo: fn(),
    savedColors: PREVIEW_SAVED_COLORS,
    settings: PREVIEW_ANNOTATE_SETTINGS,
  },
  component: AnnotateIsland,
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
  render: (args) => createElement(AnnotateIslandStage, args),
} satisfies Meta<typeof AnnotateIsland>;
