// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The kinds of personal information the law treats as special: health,
//! religion, sexuality, ethnicity, political views and union membership,
//! found by the English words that say them. The model cannot be trusted
//! with them: it scored a pregnancy no surer as a medical condition than a
//! memory leak. Words people mostly use another way are left out, such as
//! "headache", "sick", "disabled" (a button) and "Asian" (a market), and a
//! word the model takes for a name, such as Christian, stays a name.

use std::sync::LazyLock;

use regex::Regex;
use screenwide_speech::runner::sensitive::SensitiveKind;

const HEALTH: &str = r"pregnan(?:t|cy|cies)|maternity leave|paternity leave|miscarri(?:age|ages|ed)|ivf|fertility treatment|diagnosed with|surger(?:y|ies)|operated on|in hospital|hospitali[sz]ed|(?:hospital|doctor's|doctors|gp|dentist|clinic) appointments?|cancer|tumou?rs?|chemo(?:therapy)?|radiotherapy|diabet(?:es|ic)|depression|anxiety|panic attacks?|mental health|therap(?:y|ist|ists)|counsellors?|psychiatrists?|antidepressants?|medication|prescriptions?|insulin|disabilit(?:y|ies)|sick leave|sick note|off sick|long covid|covid(?:-19)?|adhd|autis(?:m|tic)|dyslexi(?:a|c)|bipolar|ptsd|ocd|asthma|epilep(?:sy|tic)|hiv|migraines?|physiotherapy|rehab|addiction|alcoholi(?:c|sm)|eating disorder|anorexi(?:a|c)|bulimi(?:a|c)|menopaus(?:e|al)|endometriosis|chronic (?:pain|illness|fatigue)";
const RELIGION: &str = r"christian(?:s|ity)?|muslims?|islam(?:ic)?|jewish|judaism|jews?|hindu(?:s|ism)?|sikh(?:s|ism)?|buddhis(?:m|ts?)|catholic(?:s|ism)?|protestants?|atheists?|church|mosque|synagogue|gurdwara|ramadan|shabbat|religion|religious";
const SEXUALITY: &str = r"gay|lesbians?|bisexual|homosexual|queer|lgbt(?:q|qia)?|transgender|non-binary|nonbinary|came out as";
const ETHNICITY: &str = r"ethnicity|ethnic(?: minority| background)?|racial|mixed race|bame|hispanic|latin[oax]|indigenous|aboriginal";
const POLITICS: &str = r"voted (?:for )?(?:labour|tory|tories|conservative|reform|lib dems?|liberal democrats?|green|snp|democrat|republican|trump|biden|harris|leave|remain)|labour party|conservative party|tory|tories|lib dems?|liberal democrats|reform uk|green party|snp|plaid cymru|democratic party|republican party|democrats|republicans|socialists?|communists?|brexiteers?|remainers?|maga|left-wing|right-wing|political views?";
const UNION: &str = r"trade unions?|union (?:rep|reps|representative|members?|membership|dues|meeting)|unioni[sz](?:ed|ing|ation)|strike action|on strike|picket line";

static WORDS: LazyLock<[(Regex, SensitiveKind); 6]> = LazyLock::new(|| {
  let words = |alternatives: &str| {
    Regex::new(&format!(r"(?i)\b(?:{alternatives})\b")).expect("a valid pattern")
  };
  [
    (words(HEALTH), SensitiveKind::Health),
    (words(RELIGION), SensitiveKind::Religion),
    (words(SEXUALITY), SensitiveKind::Sexuality),
    (words(ETHNICITY), SensitiveKind::Ethnicity),
    (words(POLITICS), SensitiveKind::Politics),
    (words(UNION), SensitiveKind::Union),
  ]
});

/// Where `text` says something of a special kind, as `[start, end)` in
/// bytes, in no particular order.
pub(super) fn special(text: &str) -> Vec<(usize, usize, SensitiveKind)> {
  WORDS
    .iter()
    .flat_map(|(words, kind)| {
      words
        .find_iter(text)
        .map(|found| (found.start(), found.end(), *kind))
    })
    .collect()
}
