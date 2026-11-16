<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Cleanup models on the English dictations

Written by `cargo run --example evaluate_cleanup`. Words are compared ignoring case and punctuation. Applied counts the dictations needing a correction that came out as expected. Unchanged counts the dictations needing none whose words came back as heard, and exactly, character for character. Words lost counts the words both heard and expected that the answer left out. Checked is the answer after the lost-word check, which puts back the text as heard when the answer fails it.

| Model | Applied | Unchanged | Exactly | Losing words | Words lost | Checked: applied | Checked: unchanged | Checked: losing words | Median ms | Slowest ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none (as heard) | 0/27 | 37/37 | 37/37 | 0 | 0 | 0/27 | 37/37 | 0 | 0 | 0 |
| speakoflow-mini-q8 | 27/27 | 35/37 | 30/37 | 2 | 4 | 26/27 | 37/37 | 0 | 142 | 224 |
| speakoflow-mini-q4 | 25/27 | 35/37 | 31/37 | 3 | 6 | 24/27 | 37/37 | 1 | 139 | 205 |
| s1-mini | 22/27 | 36/37 | 0/37 | 3 | 4 | 21/27 | 37/37 | 0 | 96 | 148 |
| gemma-4-e2b | 18/27 | 35/37 | 33/37 | 4 | 13 | 18/27 | 37/37 | 0 | 204 | 341 |

## Each dictation

An answer the lost-word check turns down is struck through.

### trigger-coffee-time (trigger)

- Expected: `Let's get coffee at 3.`
- none (as heard): `Let's get coffee at 2, actually 3`
- speakoflow-mini-q8: `Let's get coffee at 3`
- speakoflow-mini-q4: `Let's get coffee at 3`
- s1-mini: `Let's get coffee at 3.`
- gemma-4-e2b: `Let's get coffee at 3`

### trigger-ship-day (trigger)

- Expected: `Let's ship it on Friday.`
- none (as heard): `Let's ship it on Thursday. No, Friday`
- speakoflow-mini-q8: `Let's ship it on Friday.`
- speakoflow-mini-q4: `Let's ship it on Friday.`
- s1-mini: `Let's ship it on Friday.`
- gemma-4-e2b: `Let's ship it on Friday.`

### trigger-budget (trigger)

- Expected: `The budget is $6,000.`
- none (as heard): `The budget is $5,000, I mean $6,000`
- speakoflow-mini-q8: `The budget is $6,000.`
- speakoflow-mini-q4: `The budget is $6,000.`
- s1-mini: `The budget is $6,000.`
- gemma-4-e2b: `The budget is $6,000`

### trigger-room (trigger)

- Expected: `We're meeting in room 14.`
- none (as heard): `We're meeting in room 12, sorry, room 14`
- speakoflow-mini-q8: `We're meeting in room 14.`
- speakoflow-mini-q4: `We're meeting in room 14.`
- s1-mini: `We're meeting in room 14.`
- gemma-4-e2b: `We're meeting in room 14`

### trigger-guests (trigger)

- Expected: `There will be ten guests at dinner.`
- none (as heard): `There will be eight guests, wait, ten guests at dinner`
- speakoflow-mini-q8: `There will be ten guests at dinner.`
- speakoflow-mini-q4: `There will be ten guests at dinner.`
- s1-mini: `There will be ten guests at dinner.`
- gemma-4-e2b: ~~`There will be eight guests.`~~

### trigger-make-that (trigger)

- Expected: `Order three pizzas for the team.`
- none (as heard): `Order two pizzas, make that three pizzas, for the team`
- speakoflow-mini-q8: `Order three pizzas for the team`
- speakoflow-mini-q4: `Order three pizzas for the team`
- s1-mini: `Order three pizzas for the team.`
- gemma-4-e2b: `Order two pizzas, make that three pizzas, for the team`

### trigger-or-rather (trigger)

- Expected: `The report is due the week after.`
- none (as heard): `The report is due next week, or rather the week after.`
- speakoflow-mini-q8: `The report is due the week after.`
- speakoflow-mini-q4: `The report is due the week after.`
- s1-mini: `The report is due next week, or rather the week after.`
- gemma-4-e2b: `The report is due next week, or rather the week after.`

### trigger-colour (trigger)

- Expected: `Paint the door green.`
- none (as heard): `Paint the door blue, actually green`
- speakoflow-mini-q8: `Paint the door green`
- speakoflow-mini-q4: ~~`Paint the door green, actually`~~
- s1-mini: `Paint the door green.`
- gemma-4-e2b: `Paint the door green`

### trigger-flight (trigger)

- Expected: `My flight lands at 9 in the evening.`
- none (as heard): `My flight lands at 7, no, at 9 in the evening`
- speakoflow-mini-q8: `My flight lands at 9 in the evening.`
- speakoflow-mini-q4: `My flight lands at 9 in the evening.`
- s1-mini: `My flight lands at 9 in the evening.`
- gemma-4-e2b: `My flight lands at 9 in the evening`

### trigger-verb (trigger)

- Expected: `Can you call the client this afternoon?`
- none (as heard): `Can you email the client? I mean call the client, this afternoon`
- speakoflow-mini-q8: `Can you call the client this afternoon?`
- speakoflow-mini-q4: `Call the client this afternoon`
- s1-mini: `Can you call the client this afternoon?`
- gemma-4-e2b: `Can you call the client this afternoon`

### trigger-city (trigger)

- Expected: `The conference is in Munich this year.`
- none (as heard): `The conference is in Berlin, sorry, in Munich this year`
- speakoflow-mini-q8: `The conference is in Munich this year.`
- speakoflow-mini-q4: `The conference is in Munich this year.`
- s1-mini: `The conference is in Munich this year.`
- gemma-4-e2b: `The conference is in Munich this year`

### trigger-percent (trigger)

- Expected: `Sales grew by 12% last quarter.`
- none (as heard): `Sales grew by 10%, actually 12%, last quarter`
- speakoflow-mini-q8: `Sales grew by 12%, last quarter`
- speakoflow-mini-q4: `Sales grew by 12%, last quarter`
- s1-mini: `Sales grew by 12% last quarter.`
- gemma-4-e2b: `Sales grew by 12%, last quarter`

### trigger-long (trigger)

- Expected: `Please book the train for Tuesday morning and a hotel for three nights near the station.`
- none (as heard): `Please book the train for Tuesday morning and a hotel for two nights. No, three nights. Near the station`
- speakoflow-mini-q8: `Please book the train for Tuesday morning and a hotel for three nights. Near the station`
- speakoflow-mini-q4: `Please book the train for Tuesday morning and a hotel for three nights. Near the station`
- s1-mini: `Please book the train for Tuesday morning and a hotel for three nights. Near the station.`
- gemma-4-e2b: `Please book the train for Tuesday morning and a hotel for three nights. Near the station`

### trigger-sentence-end (trigger)

- Expected: `The deadline is the 20th.`
- none (as heard): `The deadline is the 15th, actually, it's the 20th`
- speakoflow-mini-q8: `The deadline is the 20th.`
- speakoflow-mini-q4: `The deadline is the 20th.`
- s1-mini: `The deadline is the 20th.`
- gemma-4-e2b: `The deadline is the 20th`

### trigger-paused (trigger)

- Expected: `Coffee at 2.`
- none (as heard): `Coffee at 3, Actually at 2`
- speakoflow-mini-q8: `Coffee at 2`
- speakoflow-mini-q4: `Coffee at 2`
- s1-mini: `Coffee at 2.`
- gemma-4-e2b: `Coffee at 2`

### trigger-paused-mean (trigger)

- Expected: `Send it to Mike.`
- none (as heard): `Send it to Mark. I mean Mike`
- speakoflow-mini-q8: `Send it to Mike.`
- speakoflow-mini-q4: `Send it to Mike.`
- s1-mini: `Send it to Mike.`
- gemma-4-e2b: `Send it to Mike.`

### no-trigger-day (no-trigger)

- Expected: `The meeting is on Tuesday at 10.`
- none (as heard): `The meeting is on Monday, on Tuesday at 10.`
- speakoflow-mini-q8: `The meeting is on Tuesday at 10.`
- speakoflow-mini-q4: `The meeting is on Tuesday at 10.`
- s1-mini: `The meeting is on Tuesday at 10.`
- gemma-4-e2b: ~~`The meeting is on Monday at 10.`~~

### no-trigger-team (no-trigger)

- Expected: `Send it to the sales team.`
- none (as heard): `Send it to the marketing team, the sales team`
- speakoflow-mini-q8: `Send it to the sales team`
- speakoflow-mini-q4: `Send it to the sales team`
- s1-mini: `Send it to the sales team.`
- gemma-4-e2b: `Send it to the marketing team, the sales team`

### no-trigger-restart (no-trigger)

- Expected: `We need to cancel the order.`
- none (as heard): `I think we should, we need to cancel the order`
- speakoflow-mini-q8: ~~`We need to cancel the order`~~
- speakoflow-mini-q4: ~~`We need to cancel the order.`~~
- s1-mini: `I think we need to cancel the order.`
- gemma-4-e2b: `I think we should, we need to cancel the order`

### no-trigger-number (no-trigger)

- Expected: `It costs forty-five dollars.`
- none (as heard): `It costs forty, forty-five dollars`
- speakoflow-mini-q8: `It costs forty-five dollars`
- speakoflow-mini-q4: `It costs forty-five dollars`
- s1-mini: ~~`It costs $45.`~~
- gemma-4-e2b: `It costs forty-five dollars`

### no-trigger-time (no-trigger)

- Expected: `Take me up at 6.30.`
- none (as heard): `Take me up at 6, at 6.30`
- speakoflow-mini-q8: `Take me up at 6.30`
- speakoflow-mini-q4: `Take me up at 6.30`
- s1-mini: `Take me up at 6:30.`
- gemma-4-e2b: `Take me up at 6.30`

### no-trigger-word (no-trigger)

- Expected: `The new design is much faster than before.`
- none (as heard): `The new design is faster, much faster than before`
- speakoflow-mini-q8: `The new design is much faster than before`
- speakoflow-mini-q4: `The new design is much faster than before`
- s1-mini: `The new design is much faster than before.`
- gemma-4-e2b: `The new design is faster, much faster than before`

### meant-actually (meant)

- Expected: `I actually enjoyed the film.`
- none (as heard): `I actually enjoyed the film`
- speakoflow-mini-q8: `I actually enjoyed the film`
- speakoflow-mini-q4: `I actually enjoyed the film`
- s1-mini: `I actually enjoyed the film.`
- gemma-4-e2b: `I actually enjoyed the film`

### meant-no (meant)

- Expected: `No, I don't think that will work.`
- none (as heard): `No, I don't think that will work`
- speakoflow-mini-q8: `No, I don't think that will work`
- speakoflow-mini-q4: `No, I don't think that will work`
- s1-mini: `No, I don't think that will work.`
- gemma-4-e2b: `No, I don't think that will work`

### meant-i-mean (meant)

- Expected: `I mean it when I say thank you.`
- none (as heard): `I mean it when I say thank you`
- speakoflow-mini-q8: `I mean it when I say thank you`
- speakoflow-mini-q4: `I mean it when I say thank you`
- s1-mini: `I mean it when I say thank you.`
- gemma-4-e2b: `I mean it when I say thank you`

### meant-sorry (meant)

- Expected: `Sorry I missed your call this morning.`
- none (as heard): `Sorry, I missed your call this morning`
- speakoflow-mini-q8: `Sorry, I missed your call this morning`
- speakoflow-mini-q4: `Sorry, I missed your call this morning`
- s1-mini: `Sorry, I missed your call this morning.`
- gemma-4-e2b: `Sorry, I missed your call this morning`

### meant-wait (meant)

- Expected: `Wait for me at the station.`
- none (as heard): `Wait for me at the station`
- speakoflow-mini-q8: `Wait for me at the station`
- speakoflow-mini-q4: `Wait for me at the station`
- s1-mini: `Wait for me at the station.`
- gemma-4-e2b: `Wait for me at the station`

### meant-make-that (meant)

- Expected: `Please make that call before lunch.`
- none (as heard): `Please make that call before lunch`
- speakoflow-mini-q8: `Please make that call before lunch`
- speakoflow-mini-q4: `Please make that call before lunch`
- s1-mini: `Please make that call before lunch.`
- gemma-4-e2b: `Please make that call before lunch`

### meant-actually-mid (meant)

- Expected: `The fix was actually simpler than we expected.`
- none (as heard): `The fix was actually simpler than we expected`
- speakoflow-mini-q8: `The fix was actually simpler than we expected`
- speakoflow-mini-q4: `The fix was actually simpler than we expected`
- s1-mini: `The fix was actually simpler than we expected.`
- gemma-4-e2b: `The fix was actually simpler than we expected`

### meant-no-list (meant)

- Expected: `There is no milk, no bread and no coffee left.`
- none (as heard): `There is no milk, no bread and no coffee left`
- speakoflow-mini-q8: `There is no milk, no bread and no coffee left`
- speakoflow-mini-q4: `There is no milk, no bread and no coffee left`
- s1-mini: `There is no milk, no bread, and no coffee left.`
- gemma-4-e2b: `There is no milk, no bread and no coffee left`

### meant-rather (meant)

- Expected: `I would rather walk than take the bus.`
- none (as heard): `I would rather walk than take the bus`
- speakoflow-mini-q8: `I would rather walk than take the bus`
- speakoflow-mini-q4: `I would rather walk than take the bus`
- s1-mini: `I would rather walk than take the bus.`
- gemma-4-e2b: `I would rather walk than take the bus`

### meant-two-days (meant)

- Expected: `We needed Monday and Tuesday, not just Monday.`
- none (as heard): `We needed Monday and Tuesday, not just Monday`
- speakoflow-mini-q8: `We needed Monday and Tuesday, not just Monday`
- speakoflow-mini-q4: `We needed Monday and Tuesday, not just Monday`
- s1-mini: `We needed Monday and Tuesday, not just Monday.`
- gemma-4-e2b: `We needed Monday and Tuesday, not just Monday`

### meant-paused (meant)

- Expected: `The fix was actually simpler than we expected.`
- none (as heard): `The fix was, actually simpler than we expected`
- speakoflow-mini-q8: `The fix was actually simpler than we expected`
- speakoflow-mini-q4: `The fix was, actually simpler than we expected`
- s1-mini: `The fix was actually simpler than we expected.`
- gemma-4-e2b: `The fix was, actually simpler than we expected`

### meant-paused-wait (meant)

- Expected: `Tell them to wait outside.`
- none (as heard): `Tell them to, wait outside`
- speakoflow-mini-q8: `Tell them to wait outside`
- speakoflow-mini-q4: `Tell them to wait outside`
- s1-mini: `Tell them to wait outside.`
- gemma-4-e2b: `Tell them to wait outside`

### name-keep (name)

- Expected: `Ask Priya and Joaquin about the Kubernetes rollout.`
- none (as heard): `Ask Priya and Joaquin about the Kubernetes rollout`
- speakoflow-mini-q8: `Ask Priya and Joaquin about the Kubernetes rollout`
- speakoflow-mini-q4: `Ask Priya and Joaquin about the Kubernetes rollout`
- s1-mini: `Ask Priya and Joaquin about the Kubernetes rollout.`
- gemma-4-e2b: `Ask Priya and Joaquin about the Kubernetes rollout`

### name-correct (name)

- Expected: `Tell Mike that the build is ready.`
- none (as heard): `Tell Mark, I mean Mike, that the build is ready`
- speakoflow-mini-q8: `Tell Mike, that the build is ready`
- speakoflow-mini-q4: `Tell Mike, that the build is ready`
- s1-mini: `Tell Mike that the build is ready.`
- gemma-4-e2b: `Tell Mike that the build is ready`

### name-company (name)

- Expected: `We recorded the demo in Screenwide yesterday.`
- none (as heard): `We recorded the demo in Screenwide yesterday`
- speakoflow-mini-q8: `We recorded the demo in Screenwide yesterday`
- speakoflow-mini-q4: `We recorded the demo in Screenwide yesterday`
- s1-mini: `We recorded the demo in Screenwide yesterday.`
- gemma-4-e2b: `We recorded the demo in Screenwide yesterday`

### name-place (name)

- Expected: `The office moved from Dublin to Galway.`
- none (as heard): `The office moved from Dublin to Galway`
- speakoflow-mini-q8: `The office moved from Dublin to Galway`
- speakoflow-mini-q4: `The office moved from Dublin to Galway`
- s1-mini: `The office moved from Dublin to Galway.`
- gemma-4-e2b: `The office moved from Dublin to Galway`

### name-correct-no-trigger (name)

- Expected: `Forwarded to Hannah in finance.`
- none (as heard): `Forwarded to Anna, to Hannah in finance`
- speakoflow-mini-q8: `Forwarded to Hannah in finance`
- speakoflow-mini-q4: `Forwarded to Hannah in finance`
- s1-mini: `Forwarded to Anna, to Hannah in finance.`
- gemma-4-e2b: `Forwarded to Anna, to Hannah in finance`

### name-surname (name)

- Expected: `Dr. Okefor will see you at noon.`
- none (as heard): `Dr. Okefor will see you at noon`
- speakoflow-mini-q8: `Dr. Okefor will see you at noon`
- speakoflow-mini-q4: `Dr. Okefor will see you at noon`
- s1-mini: `Dr. Okefor will see you at noon.`
- gemma-4-e2b: `Dr. Okefor will see you at noon`

### name-product (name)

- Expected: `Update the iPhone and the MacBook before the trip.`
- none (as heard): `Update the iPhone and the MacBook before the trip`
- speakoflow-mini-q8: `Update the iPhone and the MacBook before the trip`
- speakoflow-mini-q4: `Update the iPhone and the MacBook before the trip`
- s1-mini: `Update the iPhone and the MacBook before the trip.`
- gemma-4-e2b: `Update the iPhone and the MacBook before the trip`

### name-correct-sorry (name)

- Expected: `Invite Luca to the review.`
- none (as heard): `Invite Lucas, sorry, Luca to the review`
- speakoflow-mini-q8: `Invite Luca to the review`
- speakoflow-mini-q4: `Invite Luca to the review`
- s1-mini: `Invite Luca to the review.`
- gemma-4-e2b: `Invite Lucas, sorry, Luca to the review`

### command-new-line (command)

- Expected: `Dear Sam,⏎Thanks for coming to the review.`
- none (as heard): `Dear Sam,⏎Thanks for coming to the review`
- speakoflow-mini-q8: `Dear Sam,⏎⏎Thanks for coming to the review`
- speakoflow-mini-q4: `Dear Sam,⏎⏎Thanks for coming to the review`
- s1-mini: ~~`Dear Sam, thanks for coming to the review.`~~
- gemma-4-e2b: `Dear Sam,⏎Thanks for coming to the review`

### command-paragraph-correction (command)

- Expected: `The launch is on Monday.⏎⏎Let me know if that works.`
- none (as heard): `The launch is on Friday, no, Monday.⏎⏎Let me know if that works`
- speakoflow-mini-q8: `The launch is on Monday.⏎⏎Let me know if that works`
- speakoflow-mini-q4: `The launch is on Monday.⏎⏎Let me know if that works`
- s1-mini: ~~`The launch is on Monday. Let me know if that works.`~~
- gemma-4-e2b: `The launch is on Monday.⏎⏎Let me know if that works`

### command-scratch (command)

- Expected: `Thanks for the update. I'll review it tonight.`
- none (as heard): `Thanks for the update. I'll review it tonight`
- speakoflow-mini-q8: `Thanks for the update. I'll review it tonight`
- speakoflow-mini-q4: `Thanks for the update. I'll review it tonight`
- s1-mini: `Thanks for the update. I'll review it tonight.`
- gemma-4-e2b: `Thanks for the update. I'll review it tonight`

### command-comma (command)

- Expected: `Hi Ellen, the files are ready.`
- none (as heard): `Hi Ellen, The files are ready`
- speakoflow-mini-q8: `Hi Ellen,⏎⏎The files are ready`
- speakoflow-mini-q4: `Hi Ellen,⏎⏎The files are ready`
- s1-mini: `Hi Ellen, the files are ready.`
- gemma-4-e2b: `Hi Ellen, The files are ready`

### command-question (command)

- Expected: `Can you send it by four?`
- none (as heard): `Can you send it by three? Actually, four?`
- speakoflow-mini-q8: `Can you send it by four?`
- speakoflow-mini-q4: `Can you send it by four?`
- s1-mini: ~~`Can you send it by 3? Actually, 4?`~~
- gemma-4-e2b: `Can you send it by four?`

### command-line-clean (command)

- Expected: `Shopping list:⏎eggs, flour and sugar.`
- none (as heard): `Shopping list⏎Eggs flour and sugar`
- speakoflow-mini-q8: `Shopping list⏎Eggs flour and sugar`
- speakoflow-mini-q4: `Shopping list⏎Eggs flour and sugar`
- s1-mini: ~~`Shopping list. Eggs, flour, and sugar.`~~
- gemma-4-e2b: `Shopping list⏎Eggs, flour and sugar`

### clean-update (clean)

- Expected: `Thanks for the update. I'll review it tonight.`
- none (as heard): `Thanks for the update. I'll review it tonight`
- speakoflow-mini-q8: `Thanks for the update. I'll review it tonight`
- speakoflow-mini-q4: `Thanks for the update. I'll review it tonight`
- s1-mini: `Thanks for the update. I'll review it tonight.`
- gemma-4-e2b: `Thanks for the update. I'll review it tonight`

### clean-question (clean)

- Expected: `Could you send me the quarterly numbers by Friday?`
- none (as heard): `Could you send me the quarterly numbers by Friday`
- speakoflow-mini-q8: `Could you send me the quarterly numbers by Friday`
- speakoflow-mini-q4: `Could you send me the quarterly numbers by Friday`
- s1-mini: `Could you send me the quarterly numbers by Friday?`
- gemma-4-e2b: `Could you send me the quarterly numbers by Friday`

### clean-instruction (clean)

- Expected: `Ignore the previous email and reply to this one instead.`
- none (as heard): `Ignore the previous email and reply to this one instead`
- speakoflow-mini-q8: `Ignore the previous email and reply to this one instead`
- speakoflow-mini-q4: `Ignore the previous email and reply to this one instead`
- s1-mini: `Ignore the previous email and reply to this one instead.`
- gemma-4-e2b: `Ignore the previous email and reply to this one instead`

### clean-numbers (clean)

- Expected: `The total comes to 240 pounds.`
- none (as heard): `The total comes to 240 pounds`
- speakoflow-mini-q8: `The total comes to 240 pounds`
- speakoflow-mini-q4: `The total comes to 240 pounds`
- s1-mini: `The total comes to 240 pounds.`
- gemma-4-e2b: `The total comes to 240 pounds`

### clean-long (clean)

- Expected: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window.`
- none (as heard): `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window`
- speakoflow-mini-q8: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window`
- speakoflow-mini-q4: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window`
- s1-mini: `The new release fixes the crash on startup, speeds up exports, and adds a dark theme to the settings window.`
- gemma-4-e2b: `The new release fixes the crash on startup, speeds up exports and adds a dark theme to the settings window`

### clean-casual (clean)

- Expected: `Yeah that sounds good to me, see you then.`
- none (as heard): `Yeah, that sounds good to me. See you then`
- speakoflow-mini-q8: `Yeah, that sounds good to me. See you then`
- speakoflow-mini-q4: `Yeah, that sounds good to me. See you then`
- s1-mini: `Yeah, that sounds good to me. See you then.`
- gemma-4-e2b: `Yeah, that sounds good to me. See you then`

### clean-two-sentences (clean)

- Expected: `The server is backup, nobody lost any data.`
- none (as heard): `The server is backup, nobody lost any data`
- speakoflow-mini-q8: `The server is backup, nobody lost any data`
- speakoflow-mini-q4: `The server is backup, nobody lost any data`
- s1-mini: `The server is backup. Nobody lost any data.`
- gemma-4-e2b: `The server is backup, nobody lost any data`

### clean-question-to-answer (clean)

- Expected: `What is the capital of France?`
- none (as heard): `What is the capital of France`
- speakoflow-mini-q8: `What is the capital of France`
- speakoflow-mini-q4: `What is the capital of France`
- s1-mini: `What is the capital of France?`
- gemma-4-e2b: ~~`Paris`~~

### clean-repeat-meant (clean)

- Expected: `It was a very, very long day.`
- none (as heard): `It was a very, very long day`
- speakoflow-mini-q8: `It was a very, very long day`
- speakoflow-mini-q4: `It was a very, very long day`
- s1-mini: `It was a very, very long day.`
- gemma-4-e2b: `It was a very, very long day`

### clean-short (clean)

- Expected: `On my way.`
- none (as heard): `On my way`
- speakoflow-mini-q8: `On my way`
- speakoflow-mini-q4: `On my way`
- s1-mini: `On my way.`
- gemma-4-e2b: `On my way`

### clean-email (clean)

- Expected: `Hi Tom, the contract is signed and the invoice is attached. Best wishes, Ruth.`
- none (as heard): `Hi, Tom, the contract is signed and the invoice is attached, best wishes, Ruth`
- speakoflow-mini-q8: `Hi, Tom,⏎⏎The contract is signed and the invoice is attached.⏎⏎Best wishes,⏎Ruth`
- speakoflow-mini-q4: `Hi, Tom,⏎⏎The contract is signed and the invoice is attached.⏎⏎Best wishes,⏎Ruth`
- s1-mini: `Hi, Tom, the contract is signed and the invoice is attached. Best wishes, Ruth.`
- gemma-4-e2b: `Hi, Tom, the contract is signed and the invoice is attached, best wishes, Ruth`

### clean-not (clean)

- Expected: `Do not merge the branch until the tests pass.`
- none (as heard): `Do not merge the branch until the tests pass`
- speakoflow-mini-q8: `Do not merge the branch until the tests pass`
- speakoflow-mini-q4: `Do not merge the branch until the tests pass`
- s1-mini: `Do not merge the branch until the tests pass.`
- gemma-4-e2b: `Do not merge the branch until the tests pass`

### clean-date (clean)

- Expected: `The workshop runs from the third to the 5th of June.`
- none (as heard): `The workshop runs from the third to the 5th of June.`
- speakoflow-mini-q8: ~~`The workshop runs from the third to the fifth of June.`~~
- speakoflow-mini-q4: ~~`The workshop runs from the third to the fifth of June.`~~
- s1-mini: ~~`The workshop runs from the 3rd to the 5th of June.`~~
- gemma-4-e2b: `The workshop runs from the third to the 5th of June.`

### clean-opinion (clean)

- Expected: `Honestly, I think the first draft was better.`
- none (as heard): `Honestly, I think the first draft was better`
- speakoflow-mini-q8: `Honestly, I think the first draft was better`
- speakoflow-mini-q4: `Honestly, I think the first draft was better`
- s1-mini: `Honestly, I think the first draft was better.`
- gemma-4-e2b: `Honestly, I think the first draft was better`

### clean-negation (clean)

- Expected: `It's not Tuesday, it's Wednesday that works for me.`
- none (as heard): `It's not Tuesday, it's Wednesday that works for me`
- speakoflow-mini-q8: ~~`It's Wednesday that works for me`~~
- speakoflow-mini-q4: ~~`It's Wednesday that works for me`~~
- s1-mini: `It's not Tuesday, it's Wednesday that works for me.`
- gemma-4-e2b: ~~`It's Wednesday that works for me`~~

### clean-list (clean)

- Expected: `Bring a pen, a notebook and your laptop.`
- none (as heard): `Bring a pen, a notebook and your laptop`
- speakoflow-mini-q8: `Bring a pen, a notebook and your laptop`
- speakoflow-mini-q4: `Bring a pen, a notebook and your laptop`
- s1-mini: `Bring a pen, a notebook, and your laptop.`
- gemma-4-e2b: `Bring a pen, a notebook and your laptop`
