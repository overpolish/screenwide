// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Storybook bundles manager code with the classic JSX runtime, which needs
// `React` in scope.
import * as React from "react";
import { Button } from "storybook/internal/components";
import { addons, types, useStorybookApi } from "storybook/manager-api";

const ADDON_ID = "screenwide/native-preview";
const TOOL_ID = `${ADDON_ID}/tool`;

const TITLES = {
  copied: "Command copied to clipboard",
  idle: "Open native preview",
  launching: "Launching…",
} as const;

type Status = keyof typeof TITLES;

// lucide's app-window-mac, inlined so the manager bundle does not pull in the
// whole icon set. Sized and stroked to sit alongside Storybook's own icons.
function AppWindowMacIcon() {
  return (
    <svg
      aria-hidden
      fill="none"
      height={14}
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
      width={14}
    >
      <rect height="16" rx="2" width="20" x="2" y="4" />
      <path d="M6 8h.01" />
      <path d="M10 8h.01" />
      <path d="M14 8h.01" />
    </svg>
  );
}

function NativePreviewTool() {
  const api = useStorybookApi();
  const story = api.getCurrentStoryData() as { id?: string } | undefined;
  const storyId = story?.id;
  const [status, setStatus] = React.useState<Status>("idle");

  const launch = React.useCallback(() => {
    if (storyId == null) return;

    setStatus("launching");
    void (async () => {
      try {
        const response = await fetch(
          `/__screenwide/native-preview?story=${encodeURIComponent(storyId)}`,
          { method: "POST" },
        );
        if (!response.ok) throw new Error(`HTTP ${String(response.status)}`);
        setStatus("idle");
      } catch {
        try {
          await navigator.clipboard.writeText(
            `pnpm dev:storybook-native -- --story ${storyId}`,
          );
          setStatus("copied");
        } catch {
          setStatus("idle");
        }
      }
    })();
  }, [storyId]);

  return (
    <Button
      ariaLabel={TITLES.idle}
      disabled={storyId == null || status === "launching"}
      key={TOOL_ID}
      onClick={launch}
      padding="small"
      tooltip={TITLES[status]}
      variant="ghost"
    >
      <AppWindowMacIcon />
    </Button>
  );
}

addons.register(ADDON_ID, () => {
  addons.add(TOOL_ID, {
    match: ({ viewMode }) => viewMode === "story" || viewMode === "docs",
    render: () => <NativePreviewTool />,
    title: TITLES.idle,
    type: types.TOOL,
  });
});
