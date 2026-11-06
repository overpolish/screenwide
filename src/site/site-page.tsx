// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import github from "../assets/github-mark.svg";
import mark from "../assets/screenwide-mark.svg";
import { t } from "../i18n/i18n";
import { cn } from "../lib/styling";
import { Button } from "../primitives/button/button";
import { Text } from "../primitives/text/text";

import { RELEASES, SiteHero, SiteIsland } from "./site-hero";
import { SiteTiles } from "./site-tiles";
import { SiteTimeline } from "./site-timeline";

const SOURCE = "https://github.com/overpolish/screenwide";

/** An SVG drawn in the text's colour, so it follows the appearance. */
function MaskIcon({ className, src }: { className: string; src: string }) {
  return (
    <span
      aria-hidden
      className={cn("shrink-0 bg-current", className)}
      style={{
        // Vite inlines small files as data URLs, which must be quoted in CSS.
        mask: `url(${JSON.stringify(src)}) center / contain no-repeat`,
      }}
    />
  );
}

export function SitePage() {
  return (
    <div className="min-h-full bg-surface text-fg">
      <div className="mx-auto max-w-site px-page">
        {/* The island hangs from the top in the middle, as it does on the
            screen, between the name and the link. */}
        <header className="grid grid-cols-[1fr_auto_1fr] items-start gap-separate pt-6">
          <span className="flex h-control-height items-center gap-inline text-tile-title">
            <MaskIcon className="size-icon-lg" src={mark} />
            {t("site-name")}
          </span>
          <SiteIsland />
          <Button
            className="justify-self-end"
            onPress={() => {
              window.location.href = SOURCE;
            }}
          >
            <MaskIcon className="size-icon" src={github} />
            {t("site-nav-source")}
          </Button>
        </header>

        <main>
          <SiteHero />
          <SiteTimeline />
          <SiteTiles />
        </main>

        <footer className="mt-16 flex flex-wrap justify-between gap-x-group gap-y-inline py-8">
          <Text variant="caption">{t("site-footer-licence")}</Text>
          <Text variant="caption">
            {t("site-footer-requirements")}{" "}
            <a className="underline hover:text-fg" href={RELEASES}>
              {t("site-footer-releases")}
            </a>
          </Text>
        </footer>
      </div>
    </div>
  );
}
