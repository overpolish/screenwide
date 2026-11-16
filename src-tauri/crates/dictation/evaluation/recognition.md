<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Recognising English dictation

Written by `cargo run --example evaluate_recognition`. Word errors are the words substituted, inserted or left out against what was said, case, punctuation and the way numbers are written aside. Exact counts the recordings heard without a word wrong. Finish is from the speaker stopping to the last of the text. As expected counts a person's own recordings that came out as they should once corrected.

## macOS voices

| Recogniser | Word errors | Exact | Median finish ms | Slowest finish ms | As expected |
| --- | --- | --- | --- | --- | --- |
| parakeet-en | 1.6% (8/503) | 53/58 | 44 | 56 |  |
| whisper-turbo at pauses | 1.0% (5/503) | 55/58 | 493 | 962 |  |
| parakeet-ultra at pauses | 0.8% (4/503) | 55/58 | 42 | 65 |  |
| parakeet-en, written again at pauses | 0.8% (4/503) | 55/58 | 44 | 48 |  |

## Your recordings

| Recogniser | Word errors | Exact | Median finish ms | Slowest finish ms | As expected |
| --- | --- | --- | --- | --- | --- |
| parakeet-en | 7.7% (4/52) | 3/6 | 44 | 45 | 4/6 |
| whisper-turbo at pauses | 3.8% (2/52) | 4/6 | 502 | 511 | 4/6 |
| parakeet-ultra at pauses | 3.8% (2/52) | 4/6 | 42 | 51 | 4/6 |
| parakeet-en, written again at pauses | 3.8% (2/52) | 4/6 | 83 | 97 | 4/6 |

## Each recording

### trigger-coffee-time

- Said: `Let's get coffee at 2, actually 3.`
- parakeet-en: `Let's get coffee at 2, actually 3`
- whisper-turbo at pauses: `Let's get coffee at 2, actually 3`
- parakeet-ultra at pauses: `Let's get coffee at 2, actually 3.`
- parakeet-en, written again at pauses: `Let's get coffee at 2, actually 3.`

### trigger-ship-day

- Said: `Let's ship it on Thursday, no, Friday.`
- parakeet-en: `Let's ship it on Thursday. No, Friday`
- whisper-turbo at pauses: `Let's ship it on Thursday, no, Friday`
- parakeet-ultra at pauses: `Let's ship it on Thursday. No, Friday.`
- parakeet-en, written again at pauses: `Let's ship it on Thursday. No, Friday.`

### trigger-budget

- Said: `The budget is five thousand dollars, I mean six thousand dollars.`
- parakeet-en: `The budget is $5,000, I mean $6,000`
- whisper-turbo at pauses: `The budget is $5,000, I mean $6,000.`
- parakeet-ultra at pauses: `The budget is $5,000, I mean $6,000.`
- parakeet-en, written again at pauses: `The budget is $5,000, I mean $6,000.`

### trigger-room

- Said: `We're meeting in room twelve, sorry, room fourteen.`
- parakeet-en: `We're meeting in room 12, sorry, room 14`
- whisper-turbo at pauses: `We're meeting in room 12. Sorry, room 14.`
- parakeet-ultra at pauses: `We're meeting in room 12, sorry, room 14.`
- parakeet-en, written again at pauses: `We're meeting in room 12, sorry, room 14.`

### trigger-guests

- Said: `There will be eight guests, wait, ten guests at dinner.`
- parakeet-en: `There will be eight guests, wait, ten guests at dinner`
- whisper-turbo at pauses: `There will be 8 guests, wait, 10 guests at dinner.`
- parakeet-ultra at pauses: `There will be eight guests, wait, ten guests at dinner.`
- parakeet-en, written again at pauses: `There will be eight guests, wait, ten guests at dinner.`

### trigger-make-that

- Said: `Order two pizzas, make that three pizzas, for the team.`
- parakeet-en: `Order two pizzas, make that three pizzas, for the team`
- whisper-turbo at pauses: `Order 2 pizzas. Make that 3 pizzas. For the team.`
- parakeet-ultra at pauses: `Order two pizzas, make that three pizzas, for the team.`
- parakeet-en, written again at pauses: `Order two pizzas, make that three pizzas, for the team.`

### trigger-or-rather

- Said: `The report is due next week, or rather the week after.`
- parakeet-en: `The report is due next week, or rather the week after.`
- whisper-turbo at pauses: `The report is due next week, or rather the week after.`
- parakeet-ultra at pauses: `The report is due next week, or rather the week after.`
- parakeet-en, written again at pauses: `The report is due next week, or rather the week after.`

### trigger-colour

- Said: `Paint the door blue, actually green.`
- parakeet-en: `Paint the door blue, actually green`
- whisper-turbo at pauses: `Paint the door blue, actually green`
- parakeet-ultra at pauses: `Paint the door blue, actually green.`
- parakeet-en, written again at pauses: `Paint the door blue, actually green.`

### trigger-flight

- Said: `My flight lands at seven, no, at nine in the evening.`
- parakeet-en: `My flight lands at 7, no, at 9 in the evening`
- whisper-turbo at pauses: `My flight lands at 7, no, at 9 in the evening.`
- parakeet-ultra at pauses: `My flight lands at 7, no, at 9 in the evening.`
- parakeet-en, written again at pauses: `My flight lands at 7, no, at 9 in the evening.`

### trigger-verb

- Said: `Can you email the client, I mean call the client, this afternoon?`
- parakeet-en: `Can you email the client? I mean call the client, this afternoon`
- whisper-turbo at pauses: `Can you email the client, I mean call the client, this afternoon?`
- parakeet-ultra at pauses: `Can you email the client, I mean call the client, this afternoon?`
- parakeet-en, written again at pauses: `Can you email the client, I mean call the client, this afternoon?`

### trigger-city

- Said: `The conference is in Berlin, sorry, in Munich this year.`
- parakeet-en: `The conference is in Berlin, sorry, in Munich this year`
- whisper-turbo at pauses: `The conference is in Berlin, sorry, in Munich this year.`
- parakeet-ultra at pauses: `The conference is in Berlin, sorry, in Munich this year.`
- parakeet-en, written again at pauses: `The conference is in Berlin, sorry, in Munich this year.`

### trigger-percent

- Said: `Sales grew by ten percent, actually twelve percent, last quarter.`
- parakeet-en: `Sales grew by 10%, actually 12%, last quarter`
- whisper-turbo at pauses: `Sales grew by 10%, actually 12%, last quarter.`
- parakeet-ultra at pauses: `Sales grew by 10%, actually 12%, last quarter.`
- parakeet-en, written again at pauses: `Sales grew by 10%, actually 12%, last quarter.`

### trigger-long

- Said: `Please book the train for Tuesday morning and a hotel for two nights, no, three nights, near the station.`
- parakeet-en: `Please book the train for Tuesday morning and a hotel for two nights. No, three nights. Near the station`
- whisper-turbo at pauses: `Please book the train for Tuesday morning and a hotel for two nights, no, three nights, near the station.`
- parakeet-ultra at pauses: `Please book the train for Tuesday morning and a hotel for two nights, no, three nights, near the station.`
- parakeet-en, written again at pauses: `Please book the train for Tuesday morning and a hotel for two nights, no, three nights, near the station.`

### trigger-sentence-end

- Said: `The deadline is the fifteenth. Actually, it's the twentieth.`
- parakeet-en: `The deadline is the 15th, actually, it's the 20th`
- whisper-turbo at pauses: `The deadline is the 15th. Actually, it's the 20th.`
- parakeet-ultra at pauses: `The deadline is the 15th. Actually, it's the 20th.`
- parakeet-en, written again at pauses: `The deadline is the 15th. Actually, it's the 20th.`

### trigger-paused

- Said: `Coffee at three actually at two`
- parakeet-en: `Coffee at 3, actually at 2`
- whisper-turbo at pauses: `Coffee a 3, actually a 2`
- parakeet-ultra at pauses: `Coffee at 3. Actually at 2.`
- parakeet-en, written again at pauses: `Coffee at 3. Actually at 2.`

### trigger-paused-mean

- Said: `Send it to Mark I mean Mike.`
- parakeet-en: `Send it to Mark. I mean Mike`
- whisper-turbo at pauses: `Send it to Mark, I mean Mike`
- parakeet-ultra at pauses: `Send it to Mark. I mean Mike.`
- parakeet-en, written again at pauses: `Send it to Mark. I mean Mike.`

### no-trigger-day

- Said: `The meeting is on Monday, on Tuesday at ten.`
- parakeet-en: `The meeting is on Monday, on Tuesday at 10.`
- whisper-turbo at pauses: `The meeting is on Monday, on Tuesday at 10.`
- parakeet-ultra at pauses: `The meeting is on Monday, on Tuesday at 10.`
- parakeet-en, written again at pauses: `The meeting is on Monday, on Tuesday at 10.`

### no-trigger-team

- Said: `Send it to the marketing team, the sales team.`
- parakeet-en: `Send it to the marketing team, the sales team`
- whisper-turbo at pauses: `Send it to the marketing team, the sales team.`
- parakeet-ultra at pauses: `Send it to the marketing team, the sales team.`
- parakeet-en, written again at pauses: `Send it to the marketing team, the sales team.`

### no-trigger-restart

- Said: `I think we should, we need to cancel the order.`
- parakeet-en: `I think we should, we need to cancel the order`
- whisper-turbo at pauses: `I think we should, we need to cancel the order.`
- parakeet-ultra at pauses: `I think we should, we need to cancel the order.`
- parakeet-en, written again at pauses: `I think we should, we need to cancel the order.`

### no-trigger-number

- Said: `It costs forty, forty five dollars.`
- parakeet-en: `It costs forty, forty-five dollars`
- whisper-turbo at pauses: `It costs 40, $45`
- parakeet-ultra at pauses: `It costs $40, $45.`
- parakeet-en, written again at pauses: `It costs $40, $45.`

### no-trigger-time

- Said: `Pick me up at six, at six thirty.`
- parakeet-en: `Take me up at 6, at 6.30`
- whisper-turbo at pauses: `Pick me up at 6, at 6.30`
- parakeet-ultra at pauses: `Pick me up at 6, at 6.30.`
- parakeet-en, written again at pauses: `Pick me up at 6, at 6.30.`

### no-trigger-word

- Said: `The new design is faster, much faster than before.`
- parakeet-en: `The new design is faster, much faster than before`
- whisper-turbo at pauses: `The new design is faster, much faster than before.`
- parakeet-ultra at pauses: `The new design is faster, much faster than before.`
- parakeet-en, written again at pauses: `The new design is faster, much faster than before.`

### meant-actually

- Said: `I actually enjoyed the film.`
- parakeet-en: `I actually enjoyed the film`
- whisper-turbo at pauses: `I actually enjoyed the film`
- parakeet-ultra at pauses: `I actually enjoyed the film.`
- parakeet-en, written again at pauses: `I actually enjoyed the film.`

### meant-no

- Said: `No, I don't think that will work.`
- parakeet-en: `No, I don't think that will work`
- whisper-turbo at pauses: `No, I don't think that will work.`
- parakeet-ultra at pauses: `No, I don't think that will work.`
- parakeet-en, written again at pauses: `No, I don't think that will work.`

### meant-i-mean

- Said: `I mean it when I say thank you.`
- parakeet-en: `I mean it when I say thank you`
- whisper-turbo at pauses: `I mean it when I say thank you.`
- parakeet-ultra at pauses: `I mean it when I say thank you.`
- parakeet-en, written again at pauses: `I mean it when I say thank you.`

### meant-sorry

- Said: `Sorry I missed your call this morning.`
- parakeet-en: `Sorry, I missed your call this morning`
- whisper-turbo at pauses: `Sorry I missed your call this morning.`
- parakeet-ultra at pauses: `Sorry I missed your call this morning.`
- parakeet-en, written again at pauses: `Sorry I missed your call this morning.`

### meant-wait

- Said: `Wait for me at the station.`
- parakeet-en: `Wait for me at the station`
- whisper-turbo at pauses: `Wait for me at the station`
- parakeet-ultra at pauses: `Wait for me at the station.`
- parakeet-en, written again at pauses: `Wait for me at the station.`

### meant-make-that

- Said: `Please make that call before lunch.`
- parakeet-en: `Please make that call before lunch`
- whisper-turbo at pauses: `Please make that call before lunch.`
- parakeet-ultra at pauses: `Please make that call before lunch.`
- parakeet-en, written again at pauses: `Please make that call before lunch.`

### meant-actually-mid

- Said: `The fix was actually simpler than we expected.`
- parakeet-en: `The fix was actually simpler than we expected`
- whisper-turbo at pauses: `The fix was actually simpler than we expected`
- parakeet-ultra at pauses: `The fix was actually simpler than we expected.`
- parakeet-en, written again at pauses: `The fix was actually simpler than we expected.`

### meant-no-list

- Said: `There is no milk, no bread and no coffee left.`
- parakeet-en: `There is no milk, no bread and no coffee left`
- whisper-turbo at pauses: `There is no milk, no bread and no coffee left.`
- parakeet-ultra at pauses: `There is no milk, no bread and no coffee left.`
- parakeet-en, written again at pauses: `There is no milk, no bread and no coffee left.`

### meant-rather

- Said: `I would rather walk than take the bus.`
- parakeet-en: `I would rather walk than take the bus`
- whisper-turbo at pauses: `I would rather walk than take the bus.`
- parakeet-ultra at pauses: `I would rather walk than take the bus.`
- parakeet-en, written again at pauses: `I would rather walk than take the bus.`

### meant-two-days

- Said: `We need it Monday and Tuesday, not just Monday.`
- parakeet-en: `We needed Monday and Tuesday, not just Monday`
- whisper-turbo at pauses: `We need it Monday and Tuesday, not just Monday.`
- parakeet-ultra at pauses: `We need it Monday and Tuesday, not just Monday.`
- parakeet-en, written again at pauses: `We need it Monday and Tuesday, not just Monday.`

### meant-paused

- Said: `The fix was actually simpler than we expected.`
- parakeet-en: `The fix was, actually simpler than we expected`
- whisper-turbo at pauses: `The fix was, actually simpler than we expected`
- parakeet-ultra at pauses: `The fix was, actually simpler than we expected.`
- parakeet-en, written again at pauses: `The fix was, actually simpler than we expected.`

### meant-paused-wait

- Said: `Tell them to wait outside.`
- parakeet-en: `Tell them to, wait outside`
- whisper-turbo at pauses: `Tell them to, wait outside`
- parakeet-ultra at pauses: `Tell them to, wait outside.`
- parakeet-en, written again at pauses: `Tell them to, wait outside.`

### name-keep

- Said: `Ask Priya and Joaquin about the Kubernetes rollout.`
- parakeet-en: `Ask Priya and Joaquin about the Kubernetes rollout`
- whisper-turbo at pauses: `Ask Priya and Joaquin about the Kubernetes rollout.`
- parakeet-ultra at pauses: `Ask Priya and Joaquin about the Kubernetes rollout.`
- parakeet-en, written again at pauses: `Ask Priya and Joaquin about the Kubernetes rollout.`

### name-correct

- Said: `Tell Mark, I mean Mike, that the build is ready.`
- parakeet-en: `Tell Mark, I mean Mike, that the build is ready`
- whisper-turbo at pauses: `Tell Mark, I mean Mike, that the build is ready`
- parakeet-ultra at pauses: `Tell Mark, I mean Mike, that the build is ready.`
- parakeet-en, written again at pauses: `Tell Mark, I mean Mike, that the build is ready.`

### name-company

- Said: `We recorded the demo in Screenwide yesterday.`
- parakeet-en: `We recorded the demo in Screenwide yesterday`
- whisper-turbo at pauses: `We recorded the demo in Screenwide yesterday.`
- parakeet-ultra at pauses: `We recorded the demo in Screenwide yesterday.`
- parakeet-en, written again at pauses: `We recorded the demo in Screenwide yesterday.`

### name-place

- Said: `The office moved from Dublin to Galway.`
- parakeet-en: `The office moved from Dublin to Galway`
- whisper-turbo at pauses: `The office moved from Dublin to Galway.`
- parakeet-ultra at pauses: `The office moved from Dublin to Galway.`
- parakeet-en, written again at pauses: `The office moved from Dublin to Galway.`

### name-correct-no-trigger

- Said: `Forward it to Anna, to Hannah in finance.`
- parakeet-en: `Forwarded to Anna, to Hannah in finance`
- whisper-turbo at pauses: `Forward it to Anna, to Hannah in Finance`
- parakeet-ultra at pauses: `Forward it to Anna, to Hannah in finance.`
- parakeet-en, written again at pauses: `Forward it to Anna, to Hannah in finance.`

### name-surname

- Said: `Dr. Okafor will see you at noon.`
- parakeet-en: `Dr. Okefor will see you at noon`
- whisper-turbo at pauses: `Dr. Okafor will see you at noon.`
- parakeet-ultra at pauses: `Dr. Okafor will see you at noon.`
- parakeet-en, written again at pauses: `Dr. Okafor will see you at noon.`

### name-product

- Said: `Update the iPhone and the MacBook before the trip.`
- parakeet-en: `Update the iPhone and the MacBook before the trip`
- whisper-turbo at pauses: `Update the iPhone and the MacBook before the trip`
- parakeet-ultra at pauses: `Update the iPhone and the MacBook before the trip.`
- parakeet-en, written again at pauses: `Update the iPhone and the MacBook before the trip.`

### name-correct-sorry

- Said: `Invite Lucas, sorry, Luca to the review.`
- parakeet-en: `Invite Lucas, sorry, Luca to the review`
- whisper-turbo at pauses: `Invite Lucas, sorry, Luca to the review`
- parakeet-ultra at pauses: `Invite Lucas, sorry, Luca to the review.`
- parakeet-en, written again at pauses: `Invite Lucas, sorry, Luca to the review.`

### clean-update

- Said: `Thanks for the update. I'll review it tonight.`
- parakeet-en: `Thanks for the update. I'll review it tonight`
- whisper-turbo at pauses: `Thanks for the update. I'll review it tonight.`
- parakeet-ultra at pauses: `Thanks for the update. I'll review it tonight.`
- parakeet-en, written again at pauses: `Thanks for the update. I'll review it tonight.`

### clean-question

- Said: `Could you send me the quarterly numbers by Friday?`
- parakeet-en: `Could you send me the quarterly numbers by Friday`
- whisper-turbo at pauses: `Could you send me the quarterly numbers by Friday?`
- parakeet-ultra at pauses: `Could you send me the quarterly numbers by Friday?`
- parakeet-en, written again at pauses: `Could you send me the quarterly numbers by Friday?`

### clean-instruction

- Said: `Ignore the previous email and reply to this one instead.`
- parakeet-en: `Ignore the previous email and reply to this one instead`
- whisper-turbo at pauses: `Ignore the previous email and reply to this one instead.`
- parakeet-ultra at pauses: `Ignore the previous email and reply to this one instead.`
- parakeet-en, written again at pauses: `Ignore the previous email and reply to this one instead.`

### clean-numbers

- Said: `The total comes to two hundred and forty pounds.`
- parakeet-en: `The total comes to 240 pounds`
- whisper-turbo at pauses: `The total comes to £240`
- parakeet-ultra at pauses: `The total comes to £240.`
- parakeet-en, written again at pauses: `The total comes to £240.`

### clean-long

- Said: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window.`
- parakeet-en: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window`
- whisper-turbo at pauses: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window.`
- parakeet-ultra at pauses: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window.`
- parakeet-en, written again at pauses: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window.`

### clean-casual

- Said: `Yeah that sounds good to me, see you then.`
- parakeet-en: `Yeah, that sounds good to me. See you then`
- whisper-turbo at pauses: `Yeah that sounds good to me. See you then.`
- parakeet-ultra at pauses: `Yeah that sounds good to me. See you then.`
- parakeet-en, written again at pauses: `Yeah that sounds good to me. See you then.`

### clean-two-sentences

- Said: `The server is back up. Nobody lost any data.`
- parakeet-en: `The server is backup, nobody lost any data`
- whisper-turbo at pauses: `The server is backup. Nobody lost any data.`
- parakeet-ultra at pauses: `The server is backup, nobody lost any data.`
- parakeet-en, written again at pauses: `The server is backup, nobody lost any data.`

### clean-question-to-answer

- Said: `What is the capital of France?`
- parakeet-en: `What is the capital of France`
- whisper-turbo at pauses: `What is the capital of France?`
- parakeet-ultra at pauses: `What is the capital of France?`
- parakeet-en, written again at pauses: `What is the capital of France?`

### clean-repeat-meant

- Said: `It was a very, very long day.`
- parakeet-en: `It was a very, very long day`
- whisper-turbo at pauses: `It was a very, very long day.`
- parakeet-ultra at pauses: `It was a very, very long day.`
- parakeet-en, written again at pauses: `It was a very, very long day.`

### clean-short

- Said: `On my way.`
- parakeet-en: `On my way`
- whisper-turbo at pauses: `On my way`
- parakeet-ultra at pauses: `On my way.`
- parakeet-en, written again at pauses: `On my way.`

### clean-email

- Said: `Hi Tom, the contract is signed and the invoice is attached. Best wishes, Ruth.`
- parakeet-en: `Hi, Tom, the contract is signed and the invoice is attached, best wishes, Ruth`
- whisper-turbo at pauses: `Hi Tom, the contract is signed and the invoice is attached. Best wishes, Ruth.`
- parakeet-ultra at pauses: `Hi Tom, the contract is signed and the invoice is attached. Best wishes, Ruth.`
- parakeet-en, written again at pauses: `Hi Tom, the contract is signed and the invoice is attached. Best wishes, Ruth.`

### clean-not

- Said: `Do not merge the branch until the tests pass.`
- parakeet-en: `Do not merge the branch until the tests pass`
- whisper-turbo at pauses: `Do not merge the branch until the tests pass`
- parakeet-ultra at pauses: `Do not merge the branch until the tests pass.`
- parakeet-en, written again at pauses: `Do not merge the branch until the tests pass.`

### clean-date

- Said: `The workshop runs from the third to the fifth of June.`
- parakeet-en: `The workshop runs from the third to the 5th of June.`
- whisper-turbo at pauses: `The workshop runs from the 3rd to the 5th of June.`
- parakeet-ultra at pauses: `The workshop runs from the 3rd to the 5th of June.`
- parakeet-en, written again at pauses: `The workshop runs from the 3rd to the 5th of June.`

### clean-opinion

- Said: `Honestly, I think the first draft was better.`
- parakeet-en: `Honestly, I think the first draft was better`
- whisper-turbo at pauses: `Honestly, I think the first draft was better`
- parakeet-ultra at pauses: `Honestly, I think the first draft was better.`
- parakeet-en, written again at pauses: `Honestly, I think the first draft was better.`

### clean-negation

- Said: `It's not Tuesday, it's Wednesday that works for me.`
- parakeet-en: `It's not Tuesday, it's Wednesday that works for me`
- whisper-turbo at pauses: `It's not Tuesday, it's Wednesday that works for me`
- parakeet-ultra at pauses: `It's not Tuesday, it's Wednesday that works for me.`
- parakeet-en, written again at pauses: `It's not Tuesday, it's Wednesday that works for me.`

### clean-list

- Said: `Bring a pen, a notebook and your laptop.`
- parakeet-en: `Bring a pen, a notebook and your laptop`
- whisper-turbo at pauses: `Bring a pen, a notebook and your laptop.`
- parakeet-ultra at pauses: `Bring a pen, a notebook and your laptop.`
- parakeet-en, written again at pauses: `Bring a pen, a notebook and your laptop.`

### coffee-correction

- Said: `Let's get coffee at three, actually at two.`
- Expected: `Let's get coffee at two.`
- parakeet-en: `Let's get coffee at free. Actually, at two` → `Let's get coffee at two.`
- whisper-turbo at pauses: `Let's get coffee at 3 Actually at 2` → `Let's get coffee at 2`
- parakeet-ultra at pauses: `Let's get coffee at three, actually at two.` → `Let's get coffee at two.`
- parakeet-en, written again at pauses: `Let's get coffee at three, actually at two.` → `Let's get coffee at two.`

### thirty-three

- Said: `I think there are thirty three people coming.`
- Expected: `I think there are thirty three people coming.`
- parakeet-en: `I think there are 33 people coming` → `I think there are 33 people coming`
- whisper-turbo at pauses: `I think there are 33 people coming` → `I think there are 33 people coming`
- parakeet-ultra at pauses: `I think there are 33 people coming.` → `I think there are 33 people coming.`
- parakeet-en, written again at pauses: `I think there are 33 people coming.` → `I think there are 33 people coming.`

### thanks-both

- Said: `Thanks for both of the drafts, they look great.`
- Expected: `Thanks for both of the drafts, they look great.`
- parakeet-en: `Thanks for both of the drafts. They look great` → `Thanks for both of the drafts. They look great`
- whisper-turbo at pauses: `Thanks for both of the draughts, they look great` → `Thanks for both of the draughts, they look great`
- parakeet-ultra at pauses: `Thanks for both of the drafts. They look great.` → `Thanks for both of the drafts. They look great.`
- parakeet-en, written again at pauses: `Thanks for both of the drafts. They look great.` → `Thanks for both of the drafts. They look great.`

### three-free

- Said: `Three free tickets arrived this morning.`
- Expected: `Three free tickets arrived this morning.`
- parakeet-en: `Free, free tickets arrived this morning` → `Free tickets arrived this morning`
- whisper-turbo at pauses: `Free free tickets arrived this morning` → `Free tickets arrived this morning`
- parakeet-ultra at pauses: `Free free tickets arrived this morning.` → `Free tickets arrived this morning.`
- parakeet-en, written again at pauses: `Free free tickets arrived this morning.` → `Free tickets arrived this morning.`

### third-fourth

- Said: `Send the invoice on the third, no, the fourth of March.`
- Expected: `Send the invoice on the fourth of March.`
- parakeet-en: `Send the invoice from the third. Nice, the fourth of March` → `Send the invoice from the fourth of March`
- whisper-turbo at pauses: `Send the invoice on the 3rd No, the 4th of March` → `Send the invoice on the 4th of March`
- parakeet-ultra at pauses: `Send the invoice on the third no, the fourth of March.` → `Send the invoice on the fourth of March.`
- parakeet-en, written again at pauses: `Send the invoice on the third no, the fourth of March.` → `Send the invoice on the fourth of March.`

### thirty-forty

- Said: `We need thirty, I mean forty chairs for the hall.`
- Expected: `We need forty chairs for the hall.`
- parakeet-en: `We need 30. I mean 40 chairs for the hall` → `We need 40 chairs for the hall`
- whisper-turbo at pauses: `We need 30, I mean 40 chairs for the hall` → `We need 40 chairs for the hall`
- parakeet-ultra at pauses: `We need 30, I mean 40 chairs for the whole.` → `We need 40 chairs for the whole.`
- parakeet-en, written again at pauses: `We need 30, I mean 40 chairs for the whole.` → `We need 40 chairs for the whole.`
