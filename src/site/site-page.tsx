// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../i18n/i18n";
import { Text } from "../primitives/text/text";

import { RELEASES, SiteHero } from "./site-hero";
import { SiteTiles } from "./site-tiles";

const SOURCE = "https://github.com/overpolish/screenwide";

export function SitePage() {
  return (
    <div className="min-h-full bg-surface text-fg">
      <div className="mx-auto max-w-site px-page">
        <header className="flex items-center justify-between py-6">
          <span className="flex items-center gap-inline text-tile-title">
            <img alt="" className="size-icon-lg" src="/favicon.png" />
            {t("site-name")}
          </span>
          <a
            className="text-tile text-fg-secondary hover:text-fg"
            href={SOURCE}
          >
            {t("site-nav-source")}
          </a>
        </header>

        <main>
          <SiteHero />
          <SiteTiles />
        </main>

        <footer className="mt-16 flex flex-wrap justify-between gap-x-group gap-y-inline border-t border-divider py-8">
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
