// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { PREVIEW_WINDOWS } from "../features/recording-controls/island/island-fixtures";
import { RecordingIsland } from "../features/recording-controls/island/recording-island";
import { useIslandStage } from "../features/recording-controls/island/use-island-stage";
import { t } from "../i18n/i18n";
import { hostPlatform } from "../lib/platform";
import { cn } from "../lib/styling";
import { Button } from "../primitives/button/button";
import { Text } from "../primitives/text/text";

import { SiteClip } from "./site-clip";

import type { IslandPart } from "../primitives/island/island-outline-sync";

export const RELEASES =
  "https://github.com/overpolish/screenwide/releases/latest";

const download = () => {
  window.location.href = RELEASES;
};

function DownloadButtons() {
  // The visitor's own system's download comes first, as the primary action.
  const systems =
    hostPlatform === "windows"
      ? (["windows", "macos"] as const)
      : (["macos", "windows"] as const);
  return (
    <div className="mt-8 flex flex-wrap gap-separate">
      {systems.map((system, index) => (
        <Button
          color={index === 0 ? "primary" : "neutral"}
          key={system}
          onPress={download}
        >
          {system === "macos"
            ? t("site-download-macos")
            : t("site-download-windows")}
        </Button>
      ))}
    </div>
  );
}

/**
 * The real recording island, playing a recording's part as in its stories,
 * centred at the top of the page as it sits at the top of the screen. What
 * hangs beneath its row is drawn outside its box, so the frame keeps room
 * down to its lowest part, as the app sizes the island's window, and the
 * page below follows it as it grows and shrinks.
 */
export function SiteIsland({ className }: { className?: string }) {
  // Window mode opens on one line rather than the displays' pictures, so the
  // island stays compact; a window is chosen already, so Record works at the
  // first press.
  const island = useIslandStage({
    chosenWindow: PREVIEW_WINDOWS[0],
    initialMode: "window",
    notch: null,
  });
  const ref = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState<number>();
  const onShapeChange = (parts: IslandPart[]) => {
    const top = ref.current?.getBoundingClientRect().top;
    if (top === undefined || parts.length === 0) return;
    // Reported every frame while the island moves, so the page under it
    // moves with it rather than jumping when it settles.
    setHeight(Math.max(...parts.map((part) => part.y + part.height)) - top);
  };
  return (
    <div
      className={cn("w-fit", className)}
      ref={ref}
      style={{ minHeight: height }}
    >
      <RecordingIsland {...island} onShapeChange={onShapeChange} />
    </div>
  );
}

/** What Screenwide is, the download, and a recording opening in the editor. */
export function SiteHero() {
  return (
    <section className="grid items-start gap-x-14 gap-y-12 pt-10 pb-16 lg:grid-cols-[28rem_minmax(0,1fr)]">
      <div className="lg:pt-6">
        <h1 className="text-hero text-fg">{t("site-hero-title")}</h1>
        <p className="mt-5 text-lede text-fg-secondary">
          {t("site-hero-lede")}
        </p>
        <DownloadButtons />
        <Text className="mt-4 block" variant="caption">
          {t("site-hero-note")}
        </Text>
      </div>
      <div className="overflow-hidden rounded-window ring-1 ring-divider">
        <SiteClip label={t("site-edit-recording")} name="edit" />
      </div>
    </section>
  );
}
