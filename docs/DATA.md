# RYOCS data files

Everything RYOCS stores lives in `%APPDATA%\RYOCS`. Nothing is sent anywhere except to the services you connect yourself.

| File | What it holds |
|---|---|
| `data.json` | Your tasks, notes, settings and layout (below). Written by the app on every change. |
| `config.json` | Weather city and the Google / Spotify keys you created. |
| `tokens.json` | Sign-in tokens for Google and Spotify (deleted by *Disconnect*). |
| `esitle.json` | OneDrive sync: on/off, folder, last synced fingerprint. |
| `pencere.json` | Window: maximized or not, and the normal position and size (`maks`, `x`, `y`, `w`, `h`, screen pixels). |
| `yedekler\` | Backups: `data-YYYY-MM-DD.json` (one per day, last 14), `data-geri-alma.json` (the state before the last restore or import), `data-onedrive.json` (the OneDrive copy that lost a sync conflict). |
| `hata.log` | Error log (over 1 MB the old one becomes `hata.old`). |
| `ryocs.png` | Notification icon. |

Files are written to a temporary file first and then renamed, so a crash cannot leave half a file.

## data.json

Format version: `surum` (currently **1**). A save without `surum` is format 0 (versions up to 0.4). The app upgrades older saves when it opens them (`normal()` in `src/ui.html`) and never removes fields it does not know, so a save from a newer version is not damaged by an older one.

| Field | Type | Meaning |
|---|---|---|
| `surum` | number | Format version. |
| `tema` | string | Theme: `""` (light), `"dark"`, another theme name, `"auto"` (follows Windows) or `"gunes"` (follows sunrise and sunset). |
| `todos` | array | Tasks: `{t, d, due, p, tek, n}`. `t` text · `d` done · `due` local time `YYYY-MM-DDTHH:MM` · `p` priority 1 or 2 · `tek` repeat: `gun`, `isgunu`, `hafta`, `ay` or `g:1,3,5` (weekdays, 0 = Sunday) · `n` reminder already shown. |
| `aliskanlik` | array | Habits: `{ad, g}`, `g` = `{"YYYY-MM-DD": 1}` for ticked days. |
| `notlar`, `aktifNot` | array, number | Notes `{ad, metin, t}` (`t` last edit, ms) and the open note's index. Format 0 had a single `notes` string. |
| `links` | array | Quick links `{a, u}` (name, address). |
| `uygulamalar` | array | Apps and games `{ad, yol, simge, n, son}`: name, path or `steam://` link, icon (PNG data URL), times opened, last opened (ms). |
| `sayimlar` | array | Countdowns `{ad, t}` (`t` = `YYYY-MM-DD`). |
| `odak`, `biten` | object | Per day (`YYYY-MM-DD`): finished focus rounds and completed tasks. Kept for 90 days. |
| `odakLog` | array | Finished work rounds `{g, dk, ne}`: day, minutes, the "working on" note (if any). Kept for 90 days. |
| `havaUyari`, `ozet` | string | Day of the last weather alert and of the last morning summary. |
| `ayar` | object | Settings (below). |

### ayar (settings)

Missing keys mean the default.

| Key | Meaning |
|---|---|
| `ad`, `foto`, `pin` | Your name, profile picture (data URL), PIN lock `{tuz, oz}` (salted SHA-256, not the PIN itself). |
| `dil` | Language `"tr"` / `"en"`; missing = follows Windows. |
| `cam`, `anim`, `arka`, `vurgu`, `temaGunduz`, `temaGece` | Glass look, animations, background animation, accent colour, day / night themes for the automatic switch. |
| `uygBoyut`, `notYazi` | App icon size and note font size (`kucuk` / `""` / `buyuk`). |
| `yerlesim` | Card columns `{sol, orta, sag}` (lists of card names). |
| `kartBoy`, `kartMin`, `sutunEn`, `yanOran` | Card size weights, minimum heights (px), side column widths (px), side-by-side ratios. |
| `kartKapali`, `kartBilinen`, `kartGizle` | Closed cards, cards the save already knows (a newer card starts closed), hidden parts per card. |
| `yerYedek` | Saved layouts `[{ad, t, v}]`, at most 10. |
| `saatModel`, `saniye`, `saat2`, `saatBicim` | Clock dial, second hand, second clock time zone (`""` = off), time format `h24` / `h12` (missing = the system's habit). |
| `gorunum`, `etk`, `gizliTakvim`, `stil`, `haftaBasi`, `tatil` | Calendar view (`ay` / `hafta`), event notifications, hidden calendars, colour and icon per event type or event, first day of the week (`pzt` / `paz`), public holidays (country key such as `turkish`, `usa`, or `yok` = off). |
| `bitenGizle` | Hide completed tasks. |
| `havaGun`, `havaUyari`, `sehirler`, `birim` | Weather: 3-day view, morning alert, extra cities `[{ad, bolge, lat, lon}]` (older saves: plain names), units `c` / `f` (missing = °F in the US). |
| `haberKat`, `haberKelime`, `haberGizli` | News category, hidden words, hidden sources. |
| `calisma`, `mola`, `hedef`, `odakOto`, `rahatsiz`, `calismaAd`, `molaAd`, `odakNe` | Focus: work and break minutes, daily goal, auto start, do not disturb, own labels, "working on" note. |
| `ozet`, `tepsiBilgi`, `tur` | Morning summary on/off, "running in the tray" notice already shown, first-run tips already shown. |
| `kisayol` | Global show / hide shortcut: index in the list (0 Win+Ctrl+R, 1 Win+Alt+Y, 2 Ctrl+Alt+Shift+R, 3 Ctrl+Alt+Space) or `yok` = off. |

## config.json

```json
{
  "sehir": "Köyceğiz",
  "konum": { "ad": "Köyceğiz", "bolge": "Muğla", "lat": 36.97, "lon": 28.68 },
  "google": { "client_id": "…apps.googleusercontent.com", "client_secret": "…" },
  "spotify": { "client_id": "…", "client_secret": "" }
}
```

`konum` is the place chosen in the city picker (several towns can share a name). Older configs have only `sehir`; the first match for that name is used.

## Per-computer settings (browser storage, not in data.json)

Small conveniences kept by the app's web view and not synced: last settings tab, read news, cached weather / rates / news / calendar for a fast start, the free timer, the clipboard pause, the last version the update notice was shown for.
