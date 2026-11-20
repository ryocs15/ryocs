# RYOCS data files

Everything RYOCS stores lives in `%APPDATA%\RYOCS`. Nothing is sent anywhere except to the services you connect yourself.

| File | What it holds |
|---|---|
| `data.json` | Your tasks, notes, settings and layout (below). Written by the app on every change. Encrypted (see below). |
| `config.json` | Weather city and the Google / Spotify keys you created. Plain JSON you can edit; only `client_secret` is stored encrypted. |
| `onbellek.json` | Cached for a fast start: calendar events, held notifications, the last weather (with the place), the last Spotify track, your name for the start screen. Encrypted. |
| `tokens.json` | Sign-in tokens for Google and Spotify, encrypted with Windows DPAPI (only your Windows account on this PC can read them; deleted by *Disconnect*). |
| `esitle.json` | OneDrive sync: on/off, folder, last synced fingerprint, the OneDrive password (`parola`, encrypted). |
| `pencere.json` | Window: maximized or not, and the normal position and size (`maks`, `x`, `y`, `w`, `h`, screen pixels). |
| `yedekler\` | Backups (encrypted): `data-YYYY-MM-DD.json` (one per day, last 14), `data-geri-alma.json` (the state before the last restore or import), `data-onedrive.json` (the OneDrive copy that lost a sync conflict). |
| `hata.log` | Error log (over 1 MB the old one becomes `hata.old`). |
| `ryocs.png` | Notification icon. |

Files are written to a temporary file, flushed to disk and then renamed, so neither a crash nor a power cut can leave half a file.

## Encryption (since 1.1)

- `data.json`, the backups, `onbellek.json` and `client_secret` in `config.json` are encrypted with Windows DPAPI: `RYOCS-DPAPI1:` followed by base64. Only your Windows account on this PC can open them; you can no longer read them with Notepad.
- A plain file from an older version is still read and is encrypted at the next start. Version 1.0.7 and older cannot open the encrypted files (they show "corrupt" and do not overwrite them).
- **After a Windows reinstall, an administrator password reset, or on another PC the encrypted files and backups cannot be opened.** *Settings → Data → Export* writes a plain JSON copy for exactly that case; keep it somewhere safe.
- OneDrive copy (`<OneDrive>\RYOCS\data.json`): plain JSON unless you set a password in *Settings → Data → Password…*. With a password it is `RYOCS-PAROLA1:` + base64(salt 16 · nonce 12 · tag 16 · ciphertext): AES-256-GCM with a key from PBKDF2-HMAC-SHA256 (310 000 rounds), all from Windows CNG. Every computer needs the same password; the sync fingerprint is taken over the plain JSON.

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
| `cam`, `anim`, `arka`, `vurgu`, `temaGunduz`, `temaGece` | Glass look, animations, background animation (`yok` none, missing = light, `kabarcik`, `serit`, `aurora`, `geometri`, `dalga`, `yildiz`, `bokeh`, `lav`, `kar`, `ates`), accent colour, day / night themes for the automatic switch. |
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
| `kurTaban`, `kurListe`, `kurTur`, `altinBirim`, `gumus`, `kurDegisim` | Currency card: home currency (ISO code; missing = from the Windows country, e.g. TRY in Turkey, EUR in Germany, else USD; TRY uses TCMB, any other ECB), codes to show (comma separated; missing = USD, EUR or the two of USD/EUR/GBP that are not home), rate type `''` selling / `al` buying / `ef` banknote (TRY only), gold unit `gram` / `ons` (missing = ounce in US, GB, CA, AU, else gram), silver row, change since the previous day (missing = on). Codes the central bank feed lacks (e.g. PLN) are cross rates from open.er-api.com. |
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

Small conveniences kept by the app's web view (plain, not synced): last settings tab, read news, cached rates / news / site icons and the last sunrise and sunset times (start screen), the free timer, the focus round, the clipboard pause, the last version the update notice was shown for. Calendar events, held notifications, the weather with its place, the last track and your name are no longer kept here (they moved to `onbellek.json`; older entries are moved once and removed). Uninstalling RYOCS deletes this storage.
