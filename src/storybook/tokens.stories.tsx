// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import * as Sections from "./token-sections";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  parameters: {
    controls: { disable: true },
    layout: "centered",
  },
  title: "Foundations/Tokens",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const Typography: Story = { render: () => <Sections.Typography /> };

export const Colors: Story = { render: () => <Sections.Colors /> };

export const Materials: Story = { render: () => <Sections.Materials /> };

export const Sizes: Story = { render: () => <Sections.Sizes /> };

export const Icons: Story = { render: () => <Sections.Icons /> };

export const Spacing: Story = { render: () => <Sections.Spacing /> };

export const Radii: Story = { render: () => <Sections.Radii /> };

export const Focus: Story = { render: () => <Sections.Focus /> };
