// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The hesitation sounds each language fills a pause with, by the language
 * code the transcriber reports. Only sounds made while thinking are listed:
 * a word that is also said for its meaning ("like", "well", "este") would be
 * marked in every sentence. English is read for every language too, since
 * "um" and "uh" turn up in all of them. Spellings are as a transcript writes
 * them; a sound drawn out ("ummm") is matched by its shorter form.
 */
const FILLER_SOUNDS: Record<string, readonly string[]> = {
  ar: ["اه", "ام", "امم", "اممم", "ااه"],
  cs: ["ehm", "eee", "hm", "hmm", "ee", "em"],
  da: ["øh", "øhm", "æh", "hm", "hmm", "eh"],
  de: ["äh", "ähm", "öh", "öhm", "hm", "hmm", "eh", "ehm"],
  el: ["εε", "εμ", "εεμ", "χμ", "ε"],
  en: ["ah", "eh", "er", "erm", "hm", "hmm", "mhm", "mm", "uh", "uhm", "um"],
  es: ["eh", "em", "ehm", "mm", "mmm"],
  fa: ["اِم", "ام", "اه", "اوم"],
  fi: ["öö", "ää", "hmm", "mm", "tota", "tuota", "niinku"],
  fr: ["euh", "heu", "hum", "hm", "bah", "euhm"],
  he: ["אה", "אמ", "אמם", "אהה", "הממ"],
  hi: ["अं", "अम", "हम्म", "उम", "आं"],
  hu: ["öö", "ööö", "ö", "hm", "hmm", "izé"],
  id: ["eh", "em", "hmm", "anu", "eee"],
  it: ["eh", "ehm", "em", "mm", "mmm", "uhm"],
  ja: [
    "えー",
    "えーと",
    "えっと",
    "あのー",
    "うーん",
    "んー",
    "ええと",
    "あー",
  ],
  ko: ["음", "어", "아", "으음", "어어"],
  ms: ["eh", "em", "hmm", "anu"],
  nl: ["eh", "uh", "ehm", "uhm", "hm", "hmm"],
  no: ["eh", "øh", "æh", "ehm", "hm", "hmm"],
  pl: ["yyy", "yy", "eee", "ee", "hmm", "em", "ehm"],
  pt: ["é", "éé", "hã", "ãh", "ahn", "hum", "hmm", "eh"],
  ro: ["ăă", "ăăă", "ă", "hm", "hmm", "ăm"],
  ru: ["э", "ээ", "эм", "эээ", "мм", "ммм", "хм"],
  sk: ["ehm", "eee", "ee", "hm", "hmm"],
  sv: ["eh", "öh", "äh", "öhm", "hm", "hmm", "mm"],
  th: ["เอ่อ", "อืม", "เออ", "อ่า"],
  tr: ["ıı", "ııı", "ee", "eee", "hmm", "hım"],
  uk: ["е", "ее", "еее", "ем", "мм", "ммм", "хм"],
  vi: ["ờ", "ừm", "ừ", "à", "ơ", "ờm"],
  zh: ["嗯", "呃", "额", "唔", "啊"],
};

/** Languages the transcriber may report under another code than the list
 * above uses: Nynorsk and Norwegian share a list, as do Cantonese and the
 * Chinese written for it. */
const SAME_LIST: Record<string, string> = { nn: "no", yue: "zh" };

/**
 * Text as a filler is matched: no case, no punctuation or digits, and any
 * letter said more than once in a row kept once, so "Ummm," "umm" and "um"
 * fold alike. Combining marks stay, since Thai and Hindi vowels are written
 * with them.
 */
export const foldFillerText = (text: string) =>
  text
    .normalize("NFC")
    .toLowerCase()
    .replace(/[^\p{L}\p{M}]+/gu, "")
    .replace(/(\p{L}\p{M}*)\1+/gu, "$1");

/** The folded hesitation sounds to look for in a transcript in `language`,
 * English's always among them. */
export function fillerSoundsFor(language: string | null) {
  const code = language ? (SAME_LIST[language] ?? language) : null;
  const sounds = [
    ...FILLER_SOUNDS.en,
    ...(code && code !== "en" ? (FILLER_SOUNDS[code] ?? []) : []),
  ];
  return new Set(sounds.map(foldFillerText));
}
