// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../i18n/i18n";
import { Button } from "../primitives/button/button";

import { SiteClip } from "./site-clip";

// Payhip's checkouts, the same ones the app's Buy buttons open
// (`Tool::checkout_url` and `BUNDLE_CHECKOUT_URL` in `crates/licensing`).
// Prices are left to the checkout, so they can change without the page.
const CHECKOUT = {
  annotate: "https://store.overpolish.co/buy?s=1&link=oYtlS",
  bundle: "https://store.overpolish.co/order?link=vJeyB",
  glide: "https://store.overpolish.co/buy?s=1&link=oSXt5",
  references: "https://store.overpolish.co/buy?s=1&link=6GVnd",
  ruler: "https://store.overpolish.co/buy?s=1&link=XZ1n8",
  teleprompter: "https://store.overpolish.co/buy?s=1&link=W0pfj",
};

const open = (url: string) => () => {
  window.location.href = url;
};

type TileProps = {
  body: string;
  /** The clip in `public/recordings`. */
  clip: string;
  /** What the clip shows, for assistive technology. */
  label: string;
  title: string;
  /** A paid tool's checkout; free tools have none. */
  checkout?: string;
};

function Tile({ body, checkout, clip, label, title }: TileProps) {
  return (
    <figure className="flex flex-col gap-4">
      <div className="overflow-hidden rounded-panel ring-1 ring-divider">
        <SiteClip label={label} name={clip} />
      </div>
      <figcaption>
        <span className="flex min-h-control-height items-center justify-between gap-separate">
          <span className="text-tile-title text-fg">{title}</span>
          {checkout ? (
            <Button
              aria-label={t("site-buy-tool", { tool: title })}
              onPress={open(checkout)}
            >
              {t("site-buy")}
            </Button>
          ) : null}
        </span>
        <span className="mt-1 block text-tile text-fg-secondary">{body}</span>
      </figcaption>
    </figure>
  );
}

/** What is free and what is paid, in the grid's last place. Ten tools fill
 * whole rows of two, and leave one in the last row of three, so it takes
 * two places: a row of its own, or the rest of the last row, its two halves
 * side by side. */
function Pricing() {
  return (
    <div className="flex flex-col justify-center gap-group rounded-panel bg-fill p-8 md:col-span-2 md:grid md:grid-cols-2 md:content-center">
      <div>
        <h2 className="text-tile-title text-fg">{t("site-free-title")}</h2>
        <p className="mt-1 text-tile text-fg-secondary">
          {t("site-free-body")}
        </p>
      </div>
      <div>
        <h2 className="text-tile-title text-fg">{t("site-tools-title")}</h2>
        <p className="mt-1 text-tile text-fg-secondary">
          {t("site-tools-body")}
        </p>
        <Button className="mt-4" onPress={open(CHECKOUT.bundle)}>
          {t("site-buy-bundle")}
        </Button>
      </div>
    </div>
  );
}

/** The tools beside recording, shown at work, and what they cost. */
export function SiteTiles() {
  return (
    <section className="grid gap-x-8 gap-y-12 md:grid-cols-2 lg:grid-cols-3">
      <Tile
        body={t("site-formats-body")}
        clip="formats"
        label={t("site-formats-recording")}
        title={t("site-formats-title")}
      />
      <Tile
        body={t("site-sound-body")}
        clip="sound"
        label={t("site-sound-recording")}
        title={t("site-sound-title")}
      />
      <Tile
        body={t("site-dictate-body")}
        clip="dictation"
        label={t("site-dictate-recording")}
        title={t("site-dictate-title")}
      />
      <Tile
        body={t("site-notes-body")}
        clip="notes"
        label={t("site-notes-recording")}
        title={t("site-notes-title")}
      />
      <Tile
        body={t("site-read-body")}
        clip="text-recognition"
        label={t("site-read-recording")}
        title={t("site-read-title")}
      />
      <Tile
        body={t("site-ruler-body")}
        checkout={CHECKOUT.ruler}
        clip="ruler"
        label={t("site-ruler-recording")}
        title={t("site-ruler-title")}
      />
      <Tile
        body={t("site-annotate-body")}
        checkout={CHECKOUT.annotate}
        clip="annotate"
        label={t("site-annotate-recording")}
        title={t("site-annotate-title")}
      />
      <Tile
        body={t("site-glide-body")}
        checkout={CHECKOUT.glide}
        clip="glide"
        label={t("site-glide-recording")}
        title={t("site-glide-title")}
      />
      <Tile
        body={t("site-references-body")}
        checkout={CHECKOUT.references}
        clip="reference"
        label={t("site-references-recording")}
        title={t("site-references-title")}
      />
      <Tile
        body={t("site-teleprompter-body")}
        checkout={CHECKOUT.teleprompter}
        clip="teleprompter"
        label={t("site-teleprompter-recording")}
        title={t("site-teleprompter-title")}
      />
      <Pricing />
    </section>
  );
}
