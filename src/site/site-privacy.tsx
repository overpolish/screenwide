// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";
import { useDateFormatter } from "react-aria";

import { t } from "../i18n/i18n";
import { Text } from "../primitives/text/text";

import { SiteBrand, SiteFooter } from "./site-chrome";

const CONTACT = "dom@overpolish.co";

// When the page last changed; move it with every change to what it says.
const UPDATED = new Date(2026, 10, 6);

function Section({ children, title }: { children: string; title: string }) {
  return (
    <section>
      <h2 className="text-tile-title text-fg">{title}</h2>
      <p className="mt-1 text-tile text-fg-secondary">{children}</p>
    </section>
  );
}

/** What the app and the website send, and to whom, at /privacy. */
export function SitePrivacy() {
  const date = useDateFormatter({ dateStyle: "long" });
  useEffect(() => {
    document.title = t("site-privacy-document-title");
  }, []);
  return (
    <div className="min-h-full bg-surface text-fg">
      <div className="mx-auto max-w-site px-page">
        <header className="pt-6">
          <SiteBrand />
        </header>

        <main className="max-w-2xl pt-10">
          <h1 className="text-hero text-fg">{t("site-privacy-title")}</h1>
          <Text className="mt-2 block" variant="caption">
            {t("site-privacy-updated", { date: date.format(UPDATED) })}
          </Text>
          <p className="mt-6 text-lede text-fg-secondary">
            {t("site-privacy-intro")}
          </p>
          <p className="mt-4 text-tile text-fg-secondary">
            {t("site-privacy-controller")}
          </p>

          <div className="mt-10 flex flex-col gap-group">
            <section className="flex flex-col gap-separate">
              <h2 className="text-tile-title text-fg">
                {t("site-privacy-app-title")}
              </h2>
              <p className="text-tile text-fg-secondary">
                {t("site-privacy-licence")}
              </p>
              <p className="text-tile text-fg-secondary">
                {t("site-privacy-updates")}
              </p>
              <p className="text-tile text-fg-secondary">
                {t("site-privacy-models")}
              </p>
            </section>
            <Section title={t("site-privacy-buying-title")}>
              {t("site-privacy-buying")}
            </Section>
            <Section title={t("site-privacy-site-title")}>
              {t("site-privacy-site")}
            </Section>
            <Section title={t("site-privacy-retention-title")}>
              {t("site-privacy-retention")}
            </Section>
            <Section title={t("site-privacy-transfers-title")}>
              {t("site-privacy-transfers")}
            </Section>
            <section>
              <h2 className="text-tile-title text-fg">
                {t("site-privacy-rights-title")}
              </h2>
              <p className="mt-1 text-tile text-fg-secondary">
                {t("site-privacy-rights")}
              </p>
              <a
                className="mt-1 inline-block text-tile text-fg-secondary underline hover:text-fg"
                href={`mailto:${CONTACT}`}
              >
                {t("site-privacy-questions", { email: CONTACT })}
              </a>
            </section>
          </div>
        </main>

        <SiteFooter />
      </div>
    </div>
  );
}
