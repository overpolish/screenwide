# SPDX-FileCopyrightText: 2026 overpolish
# SPDX-License-Identifier: GPL-3.0-or-later

### The public website: what Screenwide does, shown in recordings of the app, with the real recording island to try.

site-name = Screenwide
# A link at the top of the page.
site-nav-source = Source code
site-hero-title = Everything screen.
site-hero-lede = Record your screen and edit it, dictate into any app, take notes of your calls, and read, draw on and measure what is on screen. For macOS and Windows.
site-download-macos = Download for macOS
site-download-windows = Download for Windows
# Under the download buttons.
site-hero-note = Free and open source. Tools sold separately.
# Names the recording of the editor at work, for assistive technology.
site-edit-recording = A magnifier added to a chart's peak in the editor, then the recording playing, zooming in where the clicks are and following the typing
site-free-title = Free and open source
site-free-body = Recording, editing, screenshots, text recognition, dictation and meeting notes, under the GPL.
site-read-title = Read
site-read-body = Select part of the screen and copy the text in it.
# Names the recording of Text Recognition at work, for assistive technology.
site-read-recording = Text Recognition reading a photographed page and pasting it into a note
site-dictate-title = Dictate
site-dictate-body = Speak, and the words are typed wherever your cursor is, in any app.
# Names the recording of dictation at work, for assistive technology.
site-dictate-recording = Words typed into a note as they are spoken, the dictation island showing them at the top of the screen
site-notes-title = Meeting notes
site-notes-body = Notes of your calls with each speaker told apart, made on your computer.
# Names the recording of meeting notes at work, for assistive technology.
site-notes-recording = Notes taken during a call, then opened with each speaker's lines told apart
site-sound-title = Sound
site-sound-body = Songs and sound effects from the Library, beat-matched and levelled under your voice.
# Names the recording of the sound track at work, for assistive technology.
site-sound-recording = A song from the Library dropped onto the sound track under a recording
site-annotate-title = Annotate
site-annotate-body = Draw on your screen, live, as you present or record. Annotating in the editor is free.
# Names the recording of Annotate at work, for assistive technology.
site-annotate-recording = An arrow, counters aimed at the figures, a highlight and a focus drawn over a dashboard
# The paid tools, sold one by one or all together.
site-tools-title = Paid tools
site-tools-body = Free for 14 days. Buy the ones you use, or all of them together, including any added later.
# A paid tool's button to its checkout.
site-buy = Buy
# The same button's name for assistive technology. $tool is the tool's name.
site-buy-tool = Buy { $tool }
# The button to buy every paid tool together.
site-buy-bundle = Buy all tools
site-ruler-title = Ruler
site-ruler-body = Measure sizes, spacing and alignment on screen.
# Names the recording of the Ruler at work, for assistive technology.
site-ruler-recording = The Ruler checking a dialog's padding, field height and corner radius
site-references-title = References
site-references-body = Keep a screenshot on screen while you work.
# Names the recording of a reference at work, for assistive technology.
site-references-recording = A reference of a design faded over the page as it was built
site-glide-title = Glide
site-glide-body = Snap windows to halves, quarters or thirds with a trackpad or mouse gesture.
# Names the recording of Glide at work, for assistive technology.
site-glide-recording = Two windows snapped into the halves of the screen
site-teleprompter-title = Teleprompter
site-teleprompter-body = Read your script as it follows your voice, with cues from your recordings.
# Names the recording of the teleprompter at work, for assistive technology.
site-teleprompter-recording = A script scrolling in the teleprompter, a cue dragged to its line, and each cue's picture showing as its words are read
site-footer-licence = Free software under the GNU GPL, version 3 or later.
site-footer-requirements = macOS 14.2 or later, and Windows 10 22H2 or later from the Microsoft Store.
# Links to every release and its notes.
site-footer-releases = All releases
# Links to the privacy page.
site-footer-privacy = Privacy

## The privacy page, at /privacy. It says what the app and the website send, and to whom.

site-privacy-title = Privacy
# The browser tab's title on the privacy page.
site-privacy-document-title = Screenwide privacy
# $date is when the page last changed, such as 6 November 2026.
site-privacy-updated = Updated { $date }
site-privacy-intro = Screenwide runs on your computer. It has no analytics or tracking, and your recordings, screenshots and projects stay on your computer.
site-privacy-controller = Screenwide is made by overpolish, which is responsible for the personal data described on this page.
site-privacy-app-title = What the app sends
site-privacy-licence = Licence keys. When you add a key, and every three weeks after, the app sends it to license.getscreenwide.com with a random ID made for your installation. The server checks the key with Payhip and records only whether the check worked and which tools the key unlocks. This is how the app gives you the tools you bought. The free trial is counted on your computer.
site-privacy-updates = Updates. The app asks for new versions and their release notes: the macOS version asks GitHub, and the Windows version asks the Microsoft Store, then GitHub for the notes. The Microsoft Store installs the Windows version's updates.
site-privacy-models = Models. Models for transcripts, sound and the camera are downloaded from Hugging Face and GitHub when you choose to download them.
site-privacy-online-search = Searching online. When you search online for music or sound effects in the editor's Library, the app sends the words and filters you chose to openverse.getscreenwide.com with a random ID made for your installation, separate from the one sent with licence keys. The server asks Openverse with only the words and filters. It counts each installation's searches for the day, under a scrambled form of your IP address and that ID, so one person cannot use up the searches everyone shares, and deletes the counts when the day ends. A song or sound you play or download comes straight from the site that publishes it, such as Jamendo, Wikimedia or Freesound, which sees your IP address.
site-privacy-notes-title = Meeting notes
site-privacy-notes = Meeting notes are made on your computer, from your microphone and your computer's audio. Screenwide never receives the audio, the notes or anything said in them, and no one's voice is kept as a fingerprint. A few seconds of each speaker's voice are kept with the notes so you can tell who they are, and deleted when you name them or after a week.
site-privacy-notes-consent = The notes are yours, and so is letting the people on a call know you are taking them and following the recording and consent laws where you and they are.
site-privacy-buying-title = Buying tools
site-privacy-buying = Purchases go through Payhip, which takes your payment and email address under its own privacy policy.
site-privacy-site-title = This website
site-privacy-site = The website, the licence server and the search server run on Cloudflare, which sees your IP address as any host does. The website sets no cookies and runs no analytics.
site-privacy-retention-title = How long it is kept
site-privacy-retention = The licence server keeps no record of keys or installations. The search server keeps the day's counts of searches until the day ends. Cloudflare keeps its logs, which hold the outcome of each check and details of the request such as your IP address, for up to 7 days. Payhip keeps your purchases under its own policy.
site-privacy-transfers-title = Where it is handled
site-privacy-transfers = Cloudflare, Payhip, GitHub, Microsoft, Hugging Face, Openverse and the sites that publish what you find online may handle this data outside the UK and the European Union, under the safeguards in their own terms.
site-privacy-rights-title = Your rights
site-privacy-rights = You can ask to see, correct or delete personal data about you, or object to how it is used. If you are not happy with the answer, you can complain to the Information Commissioner's Office, or to the data protection authority where you live.
# Links to an email to Screenwide. $email is the address.
site-privacy-questions = Email { $email }
