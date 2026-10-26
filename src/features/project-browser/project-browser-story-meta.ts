// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createElement } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../storybook/feature-story-stage";

import { ProjectBrowser, type ProjectBrowserProps } from "./project-browser";
import {
  locations,
  projects,
  scrubStrips,
  thumbnails,
} from "./project-browser-fixtures";
import { LiveProjectBrowser } from "./project-browser-live-preview";
import { RECENT } from "./types";

import type { Meta } from "@storybook/react-vite";

// What every story of the browser's window shares: the window's frame, the
// sample library, and its events. Each story sets the source and state it
// shows; the window answers its sidebar and Delete, so it can be tried.

const events = {
  onAddLocation: fn(),
  onClose: fn(),
  onDelete: fn(),
  onDuplicate: fn(),
  onEmpty: fn(),
  onForget: fn(),
  onMinimize: fn(),
  onMove: fn(),
  onOpen: fn(),
  onOpenFile: fn(),
  onOpenLocation: fn(),
  onRemoveLocation: fn(),
  onRename: fn(() => Promise.resolve()),
  onRequestScrubStrip: fn(),
  onRestore: fn(),
  onReveal: fn(),
  onSelect: fn(),
  onTrash: fn(),
  onUndoDelete: fn(),
};

export const projectBrowserMeta = {
  args: {
    ...events,
    copying: null,
    deleted: null,
    error: null,
    locations,
    projects,
    scrubStrips,
    selected: RECENT,
    thumbnails,
  },
  component: ProjectBrowser,
  decorators: [featureStage({ window: "projects" })],
  parameters: {
    controls: {
      exclude: [
        ...Object.keys(events),
        "locations",
        "projects",
        "scrubStrips",
        "thumbnails",
      ],
    },
    layout: "fullscreen",
  },
  render: function Render(args) {
    const [, updateArgs] = useArgs<ProjectBrowserProps>();
    return createElement(LiveProjectBrowser, { ...args, updateArgs });
  },
} satisfies Meta<typeof ProjectBrowser>;
