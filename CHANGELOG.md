# Changelog

All releases: [GitHub Releases](https://github.com/ryocs15/ryocs/releases). Your data (`%APPDATA%\RYOCS`) is kept by every update.

## Unreleased

**New**
- Polish and Spanish. RYOCS follows a Polish or Spanish Windows, or pick the language in Settings → Language and region. Notifications, the tray menu and the installer are translated too; the news card shows Polsat News in Polish and El País in Spanish, with the same six categories as in English. The translations are machine-made and not yet reviewed by native speakers; a text that is missing shows in English.
- Settings → System → Developer options. Turn it on to get the frame-rate and layout checks as buttons there and in Ctrl+F search; off, they stay out of the way.
- Gmail card settings: how many emails to show per account, from 5 to 50 (15 as before). Outlook accounts follow the same number.

**Fixes**
- Deleting a note on the left of the open one no longer switches to another note.
- The clock card's date line reads "Week 47 · day 331 of the year" in English (it was out of order), and the day of the year is right just after midnight in summer time.
- A monthly task on the 29th, 30th or 31st no longer moves to the 28th for good after February: it returns to its own day (31 Jan → 28 Feb → 31 Mar).
- One-click update keeps the last minute of screen time.
- A long error message on the weather or news card no longer ends in a broken character code.
- A link or file dropped on the window no longer replaces the dashboard with that page (the window only shows RYOCS itself).
- A count of 1 takes the singular (1 task, 1 day, 1 round instead of "1 tasks"…), and Polish uses the right form for every number (1 zadanie, 3 zadania, 5 zadań).
- The app picker window is translated too.
- With the glass look the moving background (lava, bubbles, light…) moved in a few jerky steps per second; it moves smoothly again at the screen's refresh rate. Developer options → Frame rate shows how fast the window draws, with the background and without it.
- With Fit content on, a card in the middle column whose size was set by hand could overlap the card below once the column grew (for example the notes card over the calendar after adding notes). Developer options → Layout lists each card's size and any overlap.
- In settings windows a long control no longer squeezes its label into a narrow column (for example Other cities in the weather settings, broken almost letter by letter, or Category in the news settings, hidden completely); when there is no room the control moves under the label.
- The last row of a card's extra part (for example Other cities on the weather card) no longer has its shadow and focus ring cut off at the bottom.

## 1.1.14 (2026-11-26)

Pick the weather city from country, province and district lists, and the right town for a city saved by name.

**New**
- Pick the weather city from lists: country → province → district (Settings → Language and region, and the weather card settings, also for the other cities). The lists are in the app (GeoNames, CC BY 4.0); only the chosen place is looked up for its coordinates.

**Fixes**
- Wrong sunrise and sunset for a city saved by name only: the weather took the first town with that name (Köyceğiz in Şırnak instead of Muğla). Such a save is now asked once: choose the right place and it is kept with its coordinates.
- The sun-following theme now changes in the first minute after sunrise or sunset. Before, it waited for a check at second 0, which the clock timer often skipped (above all when the window was behind others).
- Among places with the same name, the district seat now comes first (Köyceğiz → Muğla), also for a city saved by name only. The weather card shows the region next to the city (Köyceğiz · Muğla), so a wrong town shows at a glance.

## 1.1.13 (2026-11-24)

A total daily screen time limit.

**Screen time**
- A total daily limit: right-click the screen time card → Daily total limit…, in hours and minutes. A bar under today's total shows how much is used; it turns orange at 75% and red at 90%, and the card gets a red edge once the limit is passed. One notification 10 minutes before and one at the limit, once a day. Hidden programs do not count.

## 1.1.12 (2026-11-23)

Several Gmail accounts on one card, each with its own colour.

**Mail**
- The card is called Mail now.
- Your own colour for each account: right-click its button on the mail card, or click its dot in Settings → Accounts.
- Several Gmail accounts: Settings → Accounts → Add Gmail signs in another Google account for mail only (the calendar stays with the main one). The card shows all accounts together, newest first with a colour dot per account, or one account at a time; each chip has its unread count. Opening, marking and archiving work in the right account, and a new-mail notification names it. An account that lost its connection shows a reconnect button. Each extra account must be a test user in your Google Cloud project.

## 1.1.11 (2026-11-22)

Breaking news from seven outlets, subtasks, a weekly summary, a stopwatch, a fit-content layout, more animations and a new icon.

**News**
- Breaking news (Turkish) comes from seven outlets instead of one: TRT Haber, Sabah, Sözcü, Milliyet, CNN Türk, Habertürk and En Son Haber, in one list, newest first. The same story from several outlets is one row ("+2 kaynak"); right-click opens it at the other outlets. Hiding an outlet works as before. A feed that labels Turkish time as GMT no longer shows its stories in the future.
- The bold NEW label is gone: a small dot marks only the stories that came since you last looked at the card.

**Weekly summary**
- On Sunday at 20:00 a notification says your week is ready: focus rounds and time (and what you worked on most), completed tasks, habits, screen time with the top three programs, each against last week. Any time: Ctrl+F → "özet" shows this week so far against the same days of last week. Only cards you use appear; Settings → Notifications turns it off.

**Tasks**
- Subtasks: right-click a task → Add subtask (several at once, separated by commas). They show under the task with small boxes and a "2/5" count; right-click one to edit or delete it. A repeating task starts its subtasks over at the next date. Search finds a task by its subtasks too.

**Clock and timers**
- The clock card's right-click menu starts the free timer and a new stopwatch; both show under the clock. Click the time to pause or resume (a paused timer keeps its remaining time, also after a restart), × stops it. The focus card's timer can be paused too.

**Clipboard**
- Pinned items are kept after RYOCS closes (`pano.json`, encrypted); unpinned copies are still cleared. Closing the card (or Clear) drops only the unpinned ones; a pinned item goes when you unpin or delete it.

**Animations**
- Menus and the toast now close the way they open. Switching tabs (news categories, tasks / habits, notes, a screen time day) fades the new content in. On a refresh only new news, clipboard items and mail slide in; rows already on screen stay still. Deleting a clipboard item or hiding a news source lets the rows shrink away. All of it follows Settings → Animations and Windows' "reduce animations".

**Appearance**
- Settings → Appearance → Card height: "Fit content" keeps every card at the height of its content (free space stays at the bottom of the column). News, clipboard, mail, tasks and notes grow up to half the window, then scroll inside. Sizes set by hand still count. "Fill the space" is the old layout and stays the default.
- New icon: the four stones without a plate, flat, in the cards' own colours; the title bar and the start animation use the same colours.

**Screen time**
- Renaming a program that already passed its limit today no longer notifies a second time.
- A screen time file that does not open on this PC (another Windows account, damaged) is kept as `ekran.bozuk.json` instead of being overwritten.

## 1.1.10 (2026-11-22)

Screen time grows up: past days, a daily average, an early limit warning and your own program names.

**Screen time**
- Click a bar of the last 7 days: the list and the total show that day; click it again for today.
- Under the bars: the daily average of the days before today, and how far the shown day is above or below it.
- A daily limit over 10 minutes also notifies once 10 minutes before it.
- Hidden programs come back one by one from the card's right-click menu, or all at once.
- Right-click a program → Rename: your name is used on the card, in the 7-day bars and in the limit notification. Programs given the same name share one row and their time adds up; "Back to its own name" undoes it.
- Spotify showed with stray characters after its name; Electron apps that never set their own name showed as "Electron" and now show by their file name.

## 1.1.9 (2026-11-22)

A new screen time card and a random background.

**Screen time** (new card, off until you open it)
- How long you spent in each program today (the top six) and a bar for each of the last 7 days.
- Counted every 5 seconds from the window in front; not counted after 2 minutes without mouse or keyboard, unless something runs full screen (a video, a game), nor while the screen is locked.
- Right-click a program: hide it, or set a daily limit; passing it shows one notification that day.
- Stays on this PC (`ekran.json`, encrypted, the last 30 days); the card's right-click menu deletes it.
- Programs show by their own names ("Microsoft Edge", "Steam", "Settings"), not by their file names.

**Appearance**
- Background → Random: a different pattern each time RYOCS starts or the window comes back from the tray, never the same one twice in a row.

**Project**
- The node self-check moved to `tests/sonraki.test.js`.
- Release notes now come from this changelog: a short summary, the changes by card, and the download.

## 1.1.8 (2026-11-21)

Every other card got its own round of fixes and features: currency, system, clipboard, Gmail, clock, calendar, apps, tasks, notes and news.

**News**
- Right-click a story → Save for later or Copy link; the saved list opens from the card's right-click menu.
- The card's menu can mark every story of the category as read; the saved list is also in a story's right-click menu.
- When your hidden sources or words hide every story, the card says so (before: "No news in this category").

**Notes**
- Right-click a note's tab: copy it to the clipboard, clear its checked items (with what is nested under them; can be undone), or move it left or right.
- Less work while typing: the note list is redrawn only when the first line of the note changes.

**Tasks**
- A day and time at the end of a new task set its reminder: "Dentist tomorrow 15:00", "Report friday", "Call mom 18:30" (Turkish or English day names; "18.30" only after "saat").
- Right-click a task with a reminder → Snooze 1 hour or Snooze until tomorrow.
- Fixed: "Complete all" did not count the tasks for the 7-day chart.

**Apps**
- Drag an app icon onto another to reorder.
- Alt+1 … Alt+9 open the first nine apps while RYOCS is in front.
- Right-click an .exe → Open as administrator.
- When an app cannot be opened (moved or uninstalled), the message offers to remove it.

**Calendar**
- Event reminders can come 5, 10, 15 or 30 minutes before (calendar card settings).
- An event's details have "Join the meeting" for Google Meet and other video links, and "Open in Maps" for its place.
- Arrow keys move between days in the month grid; past the edge the next or previous month opens.

**Clock**
- Alarms: right-click the clock → Alarms… (once or every day); they ring while RYOCS runs, in the tray and during a focus round too. An alarm missed while the PC was asleep rings on waking (up to 3 hours late), marked as missed. The next alarm shows under the time.
- Click the date to open today in the calendar.
- Fixed: the date follows the region's order (11/21 in the US); before, it was always day/month.
- Less work every second: the date, time and greeting are rewritten only when they change.

**Gmail**
- A notification when new mail arrives, also while RYOCS is in the tray (checked every 5 min; can be turned off in the card settings).
- Right-click a mail to archive it or mark it unread again; nothing can be deleted or sent.
- The tooltip of a mail shows the start of its text.

**Clipboard**
- Right-click an item to pin it (it stays on top and does not drop off the list) or to delete it; "Clear" keeps the pinned ones. Everything is still kept only in memory.
- Each item shows when it was copied.
- Less work while the card is open: every 2 s only a version number is checked, the list itself is sent only when it changed.

**System**
- Every fixed drive is shown (the system drive first, up to 4), not only the system drive.
- A bar at 90 % or more (a nearly full disk, memory) turns red.
- The battery shows the time left when it is not charging, and the card shows how long Windows has been running.
- A small line next to CPU shows the last minute.
- Fixed: after the window was hidden for a while, the first CPU value was the average of the whole hidden time; now it waits for a fresh reading.

**Currency and gold**
- Converter: the ⇄ button in the card title opens a small converter (an amount in a shown currency ⇄ the home currency, at the card's own rates).
- Gold and silver alerts: click the gold or silver row; the alert keeps its unit (gram or ounce).
- Gold and silver show their change since the previous day, in the home currency.
- Fixed: a rate alert could fire, and be removed, on rates cached days ago when RYOCS started; alerts now check only rates fetched in the last hour.

## 1.1.7 (2026-11-21)

Three cards got their own round of work: weather, Spotify and focus.

**Weather**
- An icon next to the current temperature (a moon at night); clear and partly cloudy nights show a moon in the hourly strip too.
- A line under the current weather says when rain starts or stops in the next 12 hours, and how much warmer or colder it is than yesterday at this hour.
- Air quality (European index, PM2.5) and, in Europe, pollen in the details when the card is large.
- Click a day in the 7-day list or the 3-day strip to see its hours; click it again to go back.
- The morning alert also covers thunderstorms, snow, strong gusts, extreme heat and poor air.
- A note with tomorrow's weather once in the evening, after 6 pm (Settings → Notifications → Tomorrow's weather).
- Fixed: for a city in another time zone the hourly strip and the 24-hour curve start at that city's current hour.
- Fixed: the rain chance next to the current temperature says "rain today", because it is the day's highest chance.
- The daylight bar, the sunrise/sunset tips and the top line name the main city too, like the other cities.

**Spotify**
- Podcasts show the episode, the show and its cover (before, the card said nothing was playing).
- Click on the progress bar (or use the arrow keys on it) to jump in the track.
- The device, shuffle and repeat show under the bar; click there or right-click the player to turn shuffle and repeat on or off, or move playback to another device.
- Click the title to open the track in Spotify.
- The next track shows right when a track ends, and the card asks Spotify only every 15 s while nothing plays (5 s while playing).

**Focus**
- A long break (15 min) after every 4th work round of the day; both numbers are in the card settings, 0 minutes turns it off.
- Right-click the timer to skip the round (a skipped work round is not counted) or add 5 minutes.
- Fixed: a round that ended while RYOCS was closed is counted when it opens again (before, it could be lost), without a late notification.

**Other fixes**
- Card settings: a number field accepts 0 where 0 is allowed (before, it went back to the default).
- Gmail card: a mail marked read in RYOCS and then marked unread again in Gmail shows as unread again (before, it stayed read until RYOCS restarted).

## 1.1.5 (2026-11-21)

**Added**
- The clock card and the top line follow the city the weather card shows: temperature, sunrise and sunset, and the daylight bar (in that city's own time). They fade in together with the card.
- Gmail card: the last 15 inbox messages, not only unread ones; unread ones are bold. Clicking a message opens it and marks it read; the right-click menu marks it read without opening. Marking needs the updated Google permission (RYOCS asks when you first try); it cannot delete or send mail.

**Changed**
- Less CPU and GPU in the tray and when minimized: the web view itself is now hidden too, so it stops drawing (before, only the window was hidden). Reminders and focus rounds stay on time while hidden.
- The moving background and the Spotify bars pause after 5 minutes without mouse or keyboard input and start again on the next move or key press.

## 1.1.4 (2026-11-21)

**Changed**
- Much less GPU use with the glass look: the background pattern behind glass cards moves in small steps instead of every frame (each frame made every card recompute its blur).
- Works on a fresh PC without installing anything else: the installer adds Microsoft's WebView2 Runtime when it is missing (needs internet for that step), and the Poppins font is now part of the app, so the interface looks the same everywhere.

## 1.1.3 (2026-11-21)

**Fixed**
- Less CPU and GPU: the moving background and the Spotify bars pause while RYOCS is in the tray or behind another window.
- Started with Windows (in the tray), RYOCS no longer polls and animates until its window is opened once.
- One-click update stops if the data cannot be saved first, and old installers are removed from the temporary folder.

## 1.1.2 (2026-11-21)

**Changed**
- Settings are grouped (Personal, Features, App) with clearer icons. Notifications have their own page with the weather alert and quiet-while-focusing switches; the PIN lock and encryption moved to Privacy and security; the app icon size moved to Cards and layout.

## 1.1.1 (2026-11-21)

**Added**
- One-click update: "Update" in the notice or in Settings downloads the new installer from the GitHub release, checks it against the release's SHA-256 file, installs it without a browser and starts RYOCS again. Updating from 1.1.0 still goes through the release page once.

## 1.1.0 (2026-11-21)

**Added**
- Currency alerts: click (or right-click) a rate to get a notification once it reaches your target; a bell marks rates with an alert.
- Right-click a clipboard entry to add it as a task or append it to the current note.

**Security**
- Your data is encrypted on disk with Windows DPAPI: data.json, its backups, the calendar / notification cache and the client secrets in config.json. Older plain files are encrypted at the first start. **You cannot go back to 1.0.7 or older after this version.** Export keeps a plain copy for moving to another PC or after a Windows reinstall.
- The OneDrive copy can be password-encrypted (Settings → Data → Password…; AES-256-GCM from Windows). Without a password it stays plain, as before, and the settings say so.
- Calendar events, held notifications, the weather (with your place), the last Spotify track and your name are no longer cached in the web view's plain storage.
- The installer refuses Windows versions older than Windows 10 1709, where RYOCS cannot start.

**Fixed**
- A damaged task or note in a loaded or imported file no longer stops the panel from drawing.
- An expired Google or Spotify sign-in now offers to reconnect instead of showing the raw error.
- Saves are flushed to disk before replacing the file: a power cut no longer leaves an empty data.json.
- Loading newer data from OneDrive could collide with a save happening at the same moment.
- The currency card fetches the bank, gold, silver and world rates at the same time (on a slow network this took up to a minute).
- A focus round restored after a restart lost its remaining time when the data loaded.
- Without the WebView2 Runtime RYOCS showed nothing; it now says what is missing and opens the download page.
- Uninstalling removes the web view's cache folder.
- Quitting waits for saves in progress (up to 5 s) instead of a fixed 0.7 s, so the last change is not lost.
- Daily backups and the export file are named by the local date (they used the UTC date).
- RYOCS starts on a PC without the Visual C++ Redistributable (the runtime is now inside the exe).

**Improved**
- If the data cannot be opened, a recovery screen offers the backups, import and the data folder (Settings could not open without data).
- Rates, news and failed connections have a "Try again" button; every card that shows "no internet" refreshes by itself.
- Keyboard resizing: Alt+Shift+↑/↓ on a card, Alt+Shift+←/→ for the side columns.
- Small text (chart labels, calculator key hints, source lines) is at least 10–11 px.

## 1.0.7 (2026-11-19)

**Fixed**
- Currency card: a currency the central bank no longer publishes (e.g. BGN in TCMB) shows as a cross rate instead of "not found".
- A calendar event that was moved to a later time is reminded again before its new start.
- The focus timer survives quitting or restarting RYOCS; a round that ended while RYOCS was closed counts on the day it ended.
- Gmail card: message headers are downloaded once instead of every minute.

## 1.0.6 (2026-11-19)

**Security**
- The local server answers only the RYOCS window: a browser tab or another Windows user on the same PC can no longer read your data or start programs through it.

**Fixed**
- If data.json cannot be read for a moment (for example locked by antivirus or OneDrive), RYOCS shows an error instead of starting empty and saving the empty data over it.
- Saves are sent one after another, so the last change before closing is never overwritten by an older save.
- Data larger than 1 MB saves again (the limit is now 16 MB).
- An event that ends at midnight no longer also shows on the next day.
- The currency card downloads silver and world rates only when it shows them.

## 1.0.5 (2026-11-19)

**New**
- Currency card settings: home currency (any ISO code), which currencies to show (up to 10, about 160 to choose from), buying / selling / banknote rate for TRY, gold and silver in grams or ounces, a silver row, change since the previous day.
- Currencies the central bank does not publish (e.g. PLN in TCMB) are cross rates from ExchangeRate-API, marked "cross".
- Quick links show the site's icon, fetched from the site itself.

**Changed**
- Defaults follow your Windows country, not the app language: home currency, °C / °F, 12 / 24 h clock, first day of the week, public holidays and the first weather city.

**Fixed**
- Weather: the 7-day forecast shows as many days as fit instead of leaving a large empty space.
- Resizing a card no longer jumps a scrolled list or column back to the top.
- News from TRT Haber and BBC are sorted newest first.
- "Next 7 days" lists only days with events.
- Minimize and maximize buttons react to the mouse like the close button.

## 1.0.0 (2026-11-17)

**New**
- Four themes: Mint, Peach (light), Ocean, Plum (dark). 16 in all.
- Three animated backgrounds: Lava lamp, Snow and Fireflies.
- Settings in clearer groups: Appearance, Cards, Language and region, Calendar, Focus, Profile, App, Accounts, Data.

Your data and settings from any 0.x version are kept.

## 0.9.0 (2026-11-17)

Everything from the 0.7 and 0.8 work, plus polish before 1.0.

**New**
- Edit calendar events: title, date and time (a repeating event: this one, or the title of the whole series).
- Focus: a weekly breakdown of finished work rounds by your "working on" note.
- Search in Settings: rows from every category that match what you type.
- Choose the global show / hide shortcut, or turn it off.
- A few tips the first time RYOCS opens.
- RYOCS remembers its window size and position (and does not open off-screen if a monitor is gone).
- Settings → Data → Error log: opens the log file, useful when reporting a problem.

**Better**
- Event reminders and the morning summary work whatever month the calendar shows.
- Busy calendars: up to 1000 events per calendar (was 250).
- Google and Spotify can be connected at the same time.
- A clear message when another program uses the network port RYOCS needs (was an empty window).
- Exports go to your real Downloads folder, also when it was moved.
- News times from Bundle are correct (they were read in the wrong time zone).
- In the tray, RYOCS no longer redraws every second or polls Spotify, news, mail, system and clipboard; reminders and timers keep running.
- Higher contrast for muted text in the Light, Sand, Ice, Sage and Rose themes, and a clearer keyboard focus ring.
- The calendar header fits in narrow windows.

## 0.6.0

- Time format (24 / 12 hour), first day of the week, weather units (°C · km/h · mm or °F · mph · in) and public holidays follow your region; each can be changed in the card settings. Public holidays can be turned off.

## 0.5.0

- Automatic tests on every change (Rust, the repeat logic and the interface in a hidden browser).
- `data.json` carries a format version; saves from older versions are upgraded step by step and tested.
- [What is in each data file](docs/DATA.md).

## 0.4.2

- Switching weather cities no longer redraws the whole card; one soft slide instead.
- The place picker opens above the card settings instead of replacing them.

## 0.4.1

- Weather: when several places share a name (e.g. Yeşilova), you choose the right one; places are saved with their coordinates.

## 0.4.0

- Update check: a notice with a download link when a new version is out.
- Up to five weather cities, switched in the card.
- Second clock: 36 cities.
- Focus: your own names for "Work" and "Break", and a "what am I working on" note.
- City names in the interface language; remaining Turkish text in the English interface translated.

## 0.3.1

- Saving works again when your data is larger than 64 KB (many apps with icons).
- A failed save is now reported instead of silently lost.
- 14 daily backups are kept (was 12); `config.json` is written safely.

## 0.3.0

- First public release: English and Turkish interface, Gmail card, layout backups, do not disturb during focus, resizing cards by dragging the gaps.
