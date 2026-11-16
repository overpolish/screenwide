// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The commands of Japanese, Chinese and Korean. Japanese and Chinese write
//! their marks full width and their words without spaces.

use super::super::Command::{AddWord, Mark, RemoveWord, ScratchThat};
use super::{Phrases, NEW_LINE, NEW_PARAGRAPH};

// Whisper writes some Japanese commands said on their own as words that
// sound alike (開業 for 改行), so each has a phrase it writes as said: 次の
// 行, 新しい段落, クエスチョンマーク.
pub(super) const JAPANESE: Phrases = Phrases {
  said: &[
    ("取り消し", ScratchThat),
    ("取消し", ScratchThat),
    ("改行", NEW_LINE),
    ("次の行", NEW_LINE),
    ("改段落", NEW_PARAGRAPH),
    ("新しい段落", NEW_PARAGRAPH),
    ("読点", Mark('、')),
    ("てん", Mark('、')),
    ("句点", Mark('。')),
    ("まる", Mark('。')),
    ("〇", Mark('。')),
    ("疑問符", Mark('？')),
    ("クエスチョンマーク", Mark('？')),
    ("感嘆符", Mark('！')),
    ("ビックリマーク", Mark('！')),
    ("コロン", Mark('：')),
    ("セミコロン", Mark('；')),
    ("単語を追加", AddWord),
    ("単語を削除", RemoveWord),
  ],
  spaced: false,
  corrections: &[],
  numbers: "",
  hashtag: None,
  quotes: None,
};

pub(super) const CHINESE: Phrases = Phrases {
  said: &[
    ("撤销", ScratchThat),
    ("换行", NEW_LINE),
    ("新段落", NEW_PARAGRAPH),
    ("逗号", Mark('，')),
    ("句号", Mark('。')),
    ("问号", Mark('？')),
    ("感叹号", Mark('！')),
    ("冒号", Mark('：')),
    ("分号", Mark('；')),
    ("添加词语", AddWord),
    ("删除词语", RemoveWord),
  ],
  spaced: false,
  corrections: &[],
  numbers: "",
  hashtag: None,
  quotes: None,
};

pub(super) const KOREAN: Phrases = Phrases {
  said: &[
    ("방금 거 지워", ScratchThat),
    ("줄 바꿈", NEW_LINE),
    ("새 단락", NEW_PARAGRAPH),
    ("쉼표", Mark(',')),
    ("마침표", Mark('.')),
    ("물음표", Mark('?')),
    ("느낌표", Mark('!')),
    ("콜론", Mark(':')),
    ("세미콜론", Mark(';')),
    ("단어 추가", AddWord),
    ("단어 삭제", RemoveWord),
  ],
  spaced: true,
  corrections: &[],
  numbers: "",
  hashtag: None,
  quotes: None,
};
