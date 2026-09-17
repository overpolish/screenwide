// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "../components/base/text/text";

import {
  Colors,
  Fills,
  Focus,
  Icons,
  Page,
  Radii,
  Spacing,
  Typography,
} from "./token-sections";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  parameters: {
    controls: { disable: true },
    layout: "fullscreen",
  },
  title: "Foundations/Tokens",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const All: Story = {
  render: () => (
    <Page>
      <Text as="h1" variant="title">
        Design tokens
      </Text>
      <Typography />
      <Colors />
      <Fills />
      <Spacing />
      <Radii />
      <Focus />
      <Icons />
    </Page>
  ),
};

export const TypographyTokens: Story = {
  render: () => (
    <Page>
      <Typography />
    </Page>
  ),
};

export const ColorTokens: Story = {
  render: () => (
    <Page>
      <Colors />
    </Page>
  ),
};

export const FillTokens: Story = {
  render: () => (
    <Page>
      <Fills />
    </Page>
  ),
};

export const SpacingTokens: Story = {
  render: () => (
    <Page>
      <Spacing />
    </Page>
  ),
};

export const RadiusTokens: Story = {
  render: () => (
    <Page>
      <Radii />
    </Page>
  ),
};

export const FocusTokens: Story = {
  render: () => (
    <Page>
      <Focus />
    </Page>
  ),
};

export const IconTokens: Story = {
  render: () => (
    <Page>
      <Icons />
    </Page>
  ),
};
