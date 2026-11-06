// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { StrictMode } from "react";
import { I18nProvider } from "react-aria-components";
import { createRoot } from "react-dom/client";

import { LANGUAGES, layoutLocale, loadLocale } from "../i18n/i18n";
import { hostPlatform, stampPlatform } from "../lib/platform";

import { SitePage } from "./site-page";
import "./site.css";

/** The visitor's first language Screenwide is translated into, matched whole
 * and then by its language alone; English where none is. */
const siteLanguage = () => {
  for (const wanted of navigator.languages) {
    const match =
      LANGUAGES.find((language) => language === wanted) ??
      LANGUAGES.find(
        (language) => language.split("-")[0] === wanted.split("-")[0],
      );
    if (match) return match;
  }
  return LANGUAGES[0];
};

stampPlatform(hostPlatform);

// The app's colours follow `.dark` on the root, as its windows set it; a page
// takes the visitor's appearance and follows it when it changes.
const darkScheme = matchMedia("(prefers-color-scheme: dark)");
const followScheme = () => {
  document.documentElement.classList.toggle("dark", darkScheme.matches);
};
followScheme();
darkScheme.addEventListener("change", followScheme);

void loadLocale({
  formatLocale: navigator.language,
  language: siteLanguage(),
}).then(() => {
  createRoot(document.getElementById("root") as HTMLElement).render(
    <StrictMode>
      <I18nProvider locale={layoutLocale()}>
        <SitePage />
      </I18nProvider>
    </StrictMode>,
  );
});
