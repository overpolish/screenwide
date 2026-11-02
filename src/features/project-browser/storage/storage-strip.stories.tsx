// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { projects } from "../project-browser-fixtures";

import { StorageStrip } from "./storage-strip";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: { linked: null, onGoTo: fn(), onLink: fn(), projects },
  component: StorageStrip,
  decorators: [featureStage({ width: 192 })],
  parameters: {
    controls: { exclude: ["onGoTo", "onLink", "projects", "linked"] },
    layout: "fullscreen",
  },
  render: function Render(args) {
    const [linked, setLinked] = useState(args.linked);
    return (
      <StorageStrip
        {...args}
        linked={linked}
        onLink={(file) => {
          args.onLink(file);
          setLinked(file);
        }}
      />
    );
  },
  title: "Features/Project Browser/Storage/Strip",
} satisfies Meta<typeof StorageStrip>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Library: Story = {};

export const Pointed: Story = { args: { linked: projects[1].file } };
