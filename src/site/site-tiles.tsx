// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useNumberFormatter } from "react-aria";

import { t } from "../i18n/i18n";

import { SiteClip } from "./site-clip";

// The Tools tier's launch price, paid once after the trial.
const PRICE_GBP = 20;

type TileProps = {
  body: string;
  /** The clip in `public/recordings`. */
  clip: string;
  /** What the clip shows, for assistive technology. */
  label: string;
  title: string;
  /** Whether the tool is in the paid Tools tier. */
  isTool?: boolean;
};

function Tile({ body, clip, isTool = false, label, title }: TileProps) {
  return (
    <figure className="flex flex-col gap-4">
      <div className="overflow-hidden rounded-panel ring-1 ring-divider">
        <SiteClip label={label} name={clip} />
      </div>
      <figcaption>
        <span className="flex items-baseline justify-between gap-separate">
          <span className="text-tile-title text-fg">{title}</span>
          {isTool ? (
            <span className="text-tile text-fg-tertiary">
              {t("site-tools-title")}
            </span>
          ) : null}
        </span>
        <span className="mt-1 block text-tile text-fg-secondary">{body}</span>
      </figcaption>
    </figure>
  );
}

/** What is free and what the Tools cost, in the grid's last place. */
function Pricing() {
  const price = useNumberFormatter({
    currency: "GBP",
    maximumFractionDigits: 0,
    style: "currency",
  }).format(PRICE_GBP);
  return (
    <div className="flex flex-col justify-center gap-group rounded-panel bg-fill p-8">
      <div>
        <h2 className="text-tile-title text-fg">{t("site-free-title")}</h2>
        <p className="mt-1 text-tile text-fg-secondary">
          {t("site-free-body")}
        </p>
      </div>
      <div>
        <h2 className="text-tile-title text-fg">{t("site-tools-title")}</h2>
        <p className="mt-1 text-tile text-fg-secondary">
          {t("site-tools-body", { price })}
        </p>
      </div>
    </div>
  );
}

/** The tools beside recording, shown at work, and what they cost. */
export function SiteTiles() {
  return (
    <section className="grid gap-x-8 gap-y-12 border-t border-divider pt-12 md:grid-cols-2 lg:grid-cols-3">
      <Tile
        body={t("site-read-body")}
        clip="text-recognition"
        label={t("site-read-recording")}
        title={t("site-read-title")}
      />
      <Tile
        body={t("site-draw-body")}
        clip="annotate"
        label={t("site-draw-recording")}
        title={t("site-draw-title")}
      />
      <Tile
        body={t("site-measure-body")}
        clip="ruler"
        isTool
        label={t("site-measure-recording")}
        title={t("site-measure-title")}
      />
      <Tile
        body={t("site-keep-body")}
        clip="reference"
        isTool
        label={t("site-keep-recording")}
        title={t("site-keep-title")}
      />
      <Tile
        body={t("site-glide-body")}
        clip="glide"
        isTool
        label={t("site-glide-recording")}
        title={t("site-glide-title")}
      />
      <Pricing />
    </section>
  );
}
