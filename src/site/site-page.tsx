// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import github from "../assets/github-mark.svg";
import { t } from "../i18n/i18n";
import { Button } from "../primitives/button/button";

import { MaskIcon, SiteBrand, SiteFooter } from "./site-chrome";
import { SiteHero, SiteIsland } from "./site-hero";
import { SiteTiles } from "./site-tiles";
import { SiteTimeline } from "./site-timeline";

const SOURCE = "https://github.com/overpolish/screenwide";

export function SitePage() {
  return (
    <div className="min-h-full bg-surface text-fg">
      <div className="mx-auto max-w-site px-page">
        {/* The island hangs from the top in the middle, as it does on the
            screen, between the name and the link. Where the three do not
            fit in a row, the island takes a row of its own under them. It
            is 440px wide, so below 32rem the page cannot hold it beside its
            margins, and phone-width windows go without it. */}
        <header className="grid grid-cols-[1fr_auto] items-start gap-x-separate gap-y-6 pt-6 lg:grid-cols-[1fr_auto_1fr]">
          <SiteBrand className="col-start-1 row-start-1" isHome />
          <SiteIsland className="col-span-2 row-start-2 hidden justify-self-center min-[32rem]:block lg:col-span-1 lg:col-start-2 lg:row-start-1" />
          <Button
            className="col-start-2 row-start-1 justify-self-end lg:col-start-3"
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

        <SiteFooter />
      </div>
    </div>
  );
}
