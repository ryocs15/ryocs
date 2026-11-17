# Changelog

All releases: [GitHub Releases](https://github.com/ryocs15/ryocs/releases). Your data (`%APPDATA%\RYOCS`) is kept by every update.

## 0.9.0 (not released yet)

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
