// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Controls,
  Description,
  Primary,
  Stories,
  Subtitle,
  Title,
  useOf,
} from "@storybook/addon-docs/blocks";

// Storybook's own Docs page, less the props table for features: a feature's
// props are its wiring (callbacks, state from the backend), not an API to
// choose from, and its stories already show the states that matter.
export function DocsPage() {
  const { title } = useOf("meta", ["meta"]).preparedMeta;
  const isFeature =
    title.startsWith("Features/") || title.startsWith("Legacy/");
  return (
    <>
      <Title />
      <Subtitle />
      <Description />
      <Primary />
      {isFeature ? null : <Controls />}
      <Stories />
    </>
  );
}
