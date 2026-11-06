// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { checkProduct, type ProductAnswer } from "./payhip";
import { keyHash, signRecord, type Claims } from "./signing";

/** Each tool's Payhip product, by the entitlement the app knows it as
 * (`Tool::entitlement` in the app). Never rename an entitlement. */
const PRODUCTS = {
  ruler: "PAYHIP_SECRET_RULER",
  references: "PAYHIP_SECRET_REFERENCES",
  glide: "PAYHIP_SECRET_GLIDE",
  annotate: "PAYHIP_SECRET_ANNOTATE",
} as const satisfies Record<string, keyof Env>;

// Bounds for what a client sends; a key and an installation ID are short.
const MAX_BODY = 1024;
const MAX_FIELD = 128;

/**
 * Checks a licence key with Payhip against every tool's product, so the
 * buyer never says which tool it is for, and signs a record of the tools it
 * grants. The app takes only `invalid_key` and `disabled` as a no; every
 * other error reads there as no answer, so a tool never closes because this
 * server or Payhip failed.
 */
export async function activate(
  request: Request,
  env: Env,
  fetcher: typeof fetch,
): Promise<Response> {
  const body = await readBody(request);
  if (!body) return error(400, "bad_request");
  const { licenseKey, installationId } = body;

  const products = Object.entries(PRODUCTS);
  if (products.some(([, secret]) => !env[secret]) || !env.LICENSE_SIGNING_KEY) {
    return error(500, "not_configured");
  }
  const answers = await Promise.all(
    products.map(async ([entitlement, secret]) => ({
      entitlement,
      answer: await checkProduct(licenseKey, env[secret], fetcher),
    })),
  );

  const granted = answers.filter(({ answer }) => answer.kind === "enabled");
  const outcome = summarize(answers.map(({ answer }) => answer));
  log(outcome, granted.length);
  if (outcome !== "granted") {
    return outcome === "failed"
      ? error(502, "upstream")
      : error(outcome === "disabled" ? 403 : 400, outcome);
  }

  const email = granted
    .map(({ answer }) => (answer.kind === "enabled" ? answer.email : undefined))
    .find((found) => found !== undefined);
  const claims: Claims = {
    entitlements: granted.map(({ entitlement }) => entitlement),
    keyHash: await keyHash(licenseKey),
    installationId,
    issuedAt: Math.floor(Date.now() / 1000),
    ...(email === undefined ? {} : { email }),
  };
  return Response.json(await signRecord(claims, env.LICENSE_SIGNING_KEY));
}

/** One answer for the key from every product's: granted by any product
 * wins; "invalid_key" only when every product clearly does not know it. */
function summarize(
  answers: ProductAnswer[],
): "granted" | "disabled" | "invalid_key" | "failed" {
  if (answers.some((answer) => answer.kind === "enabled")) return "granted";
  if (answers.some((answer) => answer.kind === "disabled")) return "disabled";
  if (answers.every((answer) => answer.kind === "unknown"))
    return "invalid_key";
  return "failed";
}

async function readBody(
  request: Request,
): Promise<{ licenseKey: string; installationId: string } | null> {
  const declared = Number(request.headers.get("content-length") ?? "0");
  if (declared > MAX_BODY) return null;
  const text = await request.text();
  if (text.length > MAX_BODY) return null;
  let body: unknown;
  try {
    body = JSON.parse(text);
  } catch {
    return null;
  }
  if (typeof body !== "object" || body === null) return null;
  const licenseKey = field(body, "licenseKey");
  const installationId = field(body, "installationId");
  return licenseKey && installationId ? { licenseKey, installationId } : null;
}

function field(body: object, name: string): string | null {
  const value: unknown = Reflect.get(body, name);
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed && trimmed.length <= MAX_FIELD ? trimmed : null;
}

function error(status: number, code: string): Response {
  return Response.json({ error: code }, { status });
}

// Never the key: only what happened, for the Workers logs.
function log(outcome: string, products: number): void {
  console.log(JSON.stringify({ event: "activate", outcome, products }));
}
