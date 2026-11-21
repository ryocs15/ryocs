# Changelog

All releases: [GitHub Releases](https://github.com/ryocs15/ryocs/releases). Your data (`%APPDATA%\RYOCS`) is kept by every update.

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
