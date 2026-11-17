#![windows_subsystem = "windows"]
// RYOCS personal dashboard: local HTTP server (API + OAuth callback) + the WebView2 window that shows it.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fs, path::PathBuf, thread};
use tiny_http::{Header, Method, Request, Response, Server};

const ADDR: &str = "127.0.0.1:8765";
const ORIGIN: &str = "http://127.0.0.1:8765";
const CONFIG_SABLON: &str = r#"{
  "sehir": "Istanbul",
  "google": { "client_id": "", "client_secret": "" },
  "spotify": { "client_id": "", "client_secret": "" }
}
"#;

const AUMID: &str = "RYOCS";

/// Window events: from the server and UI threads to the event loop.
enum Olay {
    Cik,
    Kapat,    // save and exit (tray menu)
    Degistir, // shortcut: show/hide
    Goster(bool), // true: bring to front; false: show only if still hidden
    Koyu(bool),   // the title bar should match the theme
    Pencere(String), // from our own title bar: surukle | kucult | buyut | kapat | boyut:<direction>
}
static PROXY: Mutex<Option<tao::event_loop::EventLoopProxy<Olay>>> = Mutex::new(None);
fn olay(o: Olay) {
    if let Some(p) = kilit(&PROXY).as_ref() {
        p.send_event(o).ok();
    }
}

/// Pending OAuth request: (state, PKCE verifier)
/// Also unlocks a poisoned lock: one request that panicked must not lock up everything else.
fn kilit<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// One refresh per provider: Spotify changes the refresh token every time, a second concurrent refresh would be rejected.
/// Separate locks: on a slow network Spotify's refresh must not wait for Google's.
static YENILEME: [Mutex<()>; 2] = [Mutex::new(()), Mutex::new(())];
/// tokens.json read-modify-write lock (so one provider's save is not lost when two save at once).
static BELIRTEC_DOSYASI: Mutex<()> = Mutex::new(());

/// Calendar list cache (so Google is not asked on every month navigation).
static TAKVIM_LISTESI: Mutex<Option<(std::time::Instant, Value)>> = Mutex::new(None);

static OAUTH_STATE: Mutex<(String, String)> = Mutex::new((String::new(), String::new()));
static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::AgentBuilder::new()
        .tls_connector(Arc::new(native_tls::TlsConnector::new().expect("TLS")))
        .timeout(Duration::from_secs(15))
        .max_idle_connections_per_host(16)
        .build()
});

struct Provider {
    name: &'static str,
    auth: &'static str,
    token: &'static str,
    scope: &'static str,
    extra: &'static [(&'static str, &'static str)],
}

fn provider(name: &str) -> Result<Provider, String> {
    match name {
        "google" => Ok(Provider {
            name: "google",
            auth: "https://accounts.google.com/o/oauth2/v2/auth",
            token: "https://oauth2.googleapis.com/token",
            // events + calendar list (birthdays, holidays, shared calendars)
            scope: "https://www.googleapis.com/auth/calendar.events https://www.googleapis.com/auth/calendar.calendarlist.readonly https://www.googleapis.com/auth/gmail.readonly",
            // to get a refresh_token
            extra: &[("access_type", "offline"), ("prompt", "consent")],
        }),
        "spotify" => Ok(Provider {
            name: "spotify",
            auth: "https://accounts.spotify.com/authorize",
            token: "https://accounts.spotify.com/api/token",
            scope: "user-read-playback-state user-modify-playback-state user-read-currently-playing",
            extra: &[],
        }),
        _ => Err("unknown service".into()),
    }
}

fn dir() -> PathBuf {
    let d = PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".into())).join("RYOCS");
    fs::create_dir_all(&d).ok();
    d
}

/// Language of user-facing text: the choice in Settings (data.json ayar.dil), otherwise the Windows UI language (Turkish → tr).
fn dil_tr() -> bool {
    match read_json("data.json")["ayar"]["dil"].as_str() {
        Some("tr") => true,
        Some("en") => false,
        _ => (unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff) == 0x1f,
    }
}

/// Picks the Turkish or English text.
fn l(tr: &'static str, en: &'static str) -> &'static str {
    if dil_tr() { tr } else { en }
}

fn read_json(name: &str) -> Value {
    fs::read_to_string(dir().join(name))
        .ok()
        .and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_else(|| json!({}))
}

/// data.json: empty data if the file does not exist; an error if it exists but cannot be read (so the UI does not save empty data over the real data).
fn veri_oku() -> Result<Value, String> {
    match fs::read_to_string(dir().join("data.json")) {
        Err(_) => Ok(json!({})),
        Ok(s) => serde_json::from_str::<Value>(s.trim_start_matches('\u{feff}')).ok().filter(Value::is_object)
            .ok_or(l("Veri dosyası (data.json) bozuk. Ayarlar → Veri'den bir yedeği geri yükleyin.", "The data file (data.json) is corrupt. Restore a backup from Settings → Data.").into()),
    }
}

/// Appends crashes and UI errors to %APPDATA%\RYOCS\hata.log (over 1 MB the old one becomes .old).
fn hata_kaydet(metin: &str) {
    use std::io::Write;
    let p = dir().join("hata.log");
    if fs::metadata(&p).is_ok_and(|m| m.len() > 1 << 20) {
        fs::rename(&p, p.with_extension("old")).ok();
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&p) {
        let (t, sn) = (now(), now() % 86_400);
        writeln!(f, "[{} {:02}:{:02}:{:02} UTC] {}", ymd((t / 86_400) as i64), sn / 3600, sn / 60 % 60, sn % 60, metin).ok();
    }
}

fn write_json(name: &str, v: &Value) -> Result<(), String> {
    write_text(name, &v.to_string())
}

/// Temporary file first, then rename: a half-finished write must not corrupt the file.
fn write_text(name: &str, s: &str) -> Result<(), String> {
    static YAZ: Mutex<()> = Mutex::new(());
    let _y = kilit(&YAZ); // two saves at once must not write the same temporary file together
    let p = dir().join(name);
    let tmp = p.with_extension("tmp");
    fs::write(&tmp, s).and_then(|_| fs::rename(&tmp, &p)).map_err(|e| e.to_string())
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn body(r: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
    match r {
        Ok(r) => Ok(r.into_json().unwrap_or(Value::Null)), // 204 / empty body -> null
        Err(ureq::Error::Status(c, r)) => Err(format!("{c}: {}", r.into_string().unwrap_or_default())),
        // connection/timeout: the raw address and TLS details are not shown to the user
        Err(ureq::Error::Transport(_)) => Err(AG_YOK.into()),
    }
}

/// Raw text (RSS / XML, parsed in the UI); errors as in body().
fn metin(u: &str) -> Result<String, String> {
    match AGENT.get(u).call() {
        Ok(r) => r.into_string().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(c, _)) => Err(format!("{c}")),
        Err(ureq::Error::Transport(_)) => Err(AG_YOK.into()),
    }
}

/// The UI recognises this text: the "retry when the connection is back" state.
const AG_YOK: &str = "No internet connection";

/// A city's coordinates do not change: do not look them up on every weather request (saves a round trip on a slow network).
static KONUM: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(Default::default);

const GEO: &str = "https://geocoding-api.open-meteo.com/v1/search";

fn konum(city: &str) -> Result<Value, String> {
    let dil = l("tr", "en"); // the city name comes back in the UI language
    let anahtar = format!("{dil}:{city}");
    if let Some(v) = kilit(&KONUM).get(&anahtar) {
        return Ok(v.clone());
    }
    let g = body(AGENT.get(GEO).query("name", city).query("count", "1").query("language", dil).call())?;
    let v = g["results"].get(0).map(yer).ok_or(format!("{} {city}", l("Şehir bulunamadı:", "City not found:")))?;
    kilit(&KONUM).insert(anahtar, v.clone());
    Ok(v)
}

/// A place in the short form the UI uses: name, region (district, province / state), country, coordinates.
fn yer(r: &Value) -> Value {
    let ad = r["name"].as_str().unwrap_or("");
    let mut b: Vec<&str> = vec![];
    for k in ["admin2", "admin1"] {
        if let Some(s) = r[k].as_str().map(|s| s.trim_end_matches(" İlçesi").trim_end_matches(" District")) {
            if !s.is_empty() && s != ad && !b.contains(&s) {
                b.push(s);
            }
        }
    }
    json!({ "ad": ad, "bolge": b.join(", "), "ulke": r["country"], "lat": r["latitude"], "lon": r["longitude"] })
}

/// A place sent by the UI (picker result, saved extra city): coordinates in range, short texts only.
fn yer_dogrula(v: &Value) -> Result<Value, String> {
    let k = |s: &str, n: usize| v[s].as_str().map(|x| x.trim().chars().take(n).collect::<String>()).unwrap_or_default();
    let ad = k("ad", 80);
    match (v["lat"].as_f64().filter(|x| x.abs() <= 90.0), v["lon"].as_f64().filter(|x| x.abs() <= 180.0)) {
        (Some(lat), Some(lon)) if !ad.is_empty() => Ok(json!({ "ad": ad, "bolge": k("bolge", 120), "lat": lat, "lon": lon })),
        _ => Err(l("Geçersiz konum", "Invalid location").into()),
    }
}

fn sehir_adi(s: &str) -> Result<&str, String> {
    Some(s.trim()).filter(|s| !s.is_empty() && s.chars().count() <= 80).ok_or(l("Şehir adı gerekli", "City name is required").into())
}

/// City search for the picker: several towns share a name (Köyceğiz, Yeşilova…), so up to 8 places with their region.
fn sehir_ara(q: &str) -> Result<Value, String> {
    let g = body(AGENT.get(GEO).query("name", sehir_adi(q)?).query("count", "8").query("language", l("tr", "en")).call())?;
    Ok(json!(g["results"].as_array().into_iter().flatten().map(yer).collect::<Vec<_>>()))
}

/// Weather of a place: ?lat=&lon=&ad=&bolge= (chosen in the picker), ?sehir= (a name, older saves), else the main city in config.json.
fn weather(q: &HashMap<String, String>) -> Result<Value, String> {
    let cfg = read_json("config.json");
    let s = |k: &str| q.get(k).cloned().unwrap_or_default();
    let loc = if q.contains_key("lat") {
        yer_dogrula(&json!({ "ad": s("ad"), "bolge": s("bolge"), "lat": s("lat").parse::<f64>().ok(), "lon": s("lon").parse::<f64>().ok() }))?
    } else if let Some(c) = q.get("sehir") {
        konum(sehir_adi(c)?)?
    } else if cfg["konum"].is_object() {
        yer_dogrula(&cfg["konum"])?
    } else {
        konum(sehir_adi(cfg["sehir"].as_str().unwrap_or("Istanbul"))?)?
    };
    let mut w = body(
        AGENT.get("https://api.open-meteo.com/v1/forecast")
            .query("latitude", &loc["lat"].to_string())
            .query("longitude", &loc["lon"].to_string())
            .query("current", "temperature_2m,apparent_temperature,weather_code,wind_speed_10m,wind_direction_10m,relative_humidity_2m,pressure_msl,precipitation")
            .query("hourly", "temperature_2m,precipitation_probability,weather_code")
            .query("daily", "weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,sunrise,sunset,uv_index_max")
            .query("timezone", "auto").query("forecast_days", "7").call(),
    )?;
    w["sehir"] = loc["ad"].clone();
    w["bolge"] = loc["bolge"].clone();
    Ok(w)
}

/// Bundle news: the (undocumented) feed the bundle.app site itself uses; no credentials needed.
/// shortcut: not an official API; if Bundle changes it the card says "could not load"; switch to an official API/RSS if one appears.
fn haber_kat(b: &str) -> &'static str {
    // the category in the request comes only from the allow list: an outside value must not reach the URL path
    let k = serde_json::from_str::<Value>(b).ok().and_then(|v| v["kat"].as_str().map(String::from)).unwrap_or_default();
    ["sondakika", "gundem", "bilim", "finans", "yasam", "spor", "en_top", "en_world", "en_business", "en_tech", "en_science", "en_sport"]
        .into_iter().find(|x| *x == k).unwrap_or("gundem")
}

fn haber(b: &str) -> Result<Value, String> {
    // Bundle has no breaking news: the TRT Haber RSS goes to the UI as raw text and is parsed there with DOMParser.
    // shortcut: not a documented API; if TRT changes the address the card says "could not load".
    // English tabs: BBC News RSS feeds, parsed in the UI like the TRT feed.
    let rss = match haber_kat(b) {
        "sondakika" => Some("https://www.trthaber.com/sondakika.rss"),
        "en_top" => Some("https://feeds.bbci.co.uk/news/rss.xml"),
        "en_world" => Some("https://feeds.bbci.co.uk/news/world/rss.xml"),
        "en_business" => Some("https://feeds.bbci.co.uk/news/business/rss.xml"),
        "en_tech" => Some("https://feeds.bbci.co.uk/news/technology/rss.xml"),
        "en_science" => Some("https://feeds.bbci.co.uk/news/science_and_environment/rss.xml"),
        "en_sport" => Some("https://feeds.bbci.co.uk/sport/rss.xml"),
        _ => None,
    };
    if let Some(u) = rss {
        return metin(u).map(Value::String);
    }
    body(
        AGENT.get("https://www.bundle.app/api/main/get-feed")
            .query("locale", "tr").query("page", &format!("/{}", haber_kat(b))).call(),
    )
}

/// "x.y.z" (optionally with a leading v) as numbers; anything else (pre-release tags etc.) is None.
fn surum_sayi(s: &str) -> Option<(u32, u32, u32)> {
    let mut p = s.trim_start_matches('v').split('.').map(|x| x.parse::<u32>().ok());
    let r = (p.next()??, p.next()??, p.next()??);
    p.next().is_none().then_some(r)
}

/// Update check: the latest GitHub release against this version. The link is built here, not taken from the response.
fn guncelleme() -> Result<Value, String> {
    let r = body(AGENT.get("https://api.github.com/repos/ryocs15/ryocs/releases/latest").set("Accept", "application/vnd.github+json").call())?;
    let son = r["tag_name"].as_str().and_then(surum_sayi).ok_or(l("Sürüm bilgisi okunamadı", "Could not read the version info"))?;
    let s = format!("{}.{}.{}", son.0, son.1, son.2);
    Ok(json!({
        "surum": env!("CARGO_PKG_VERSION"), "son": s,
        "yeni": Some(son) > surum_sayi(env!("CARGO_PKG_VERSION")),
        "url": format!("https://github.com/ryocs15/ryocs/releases/tag/v{s}"),
    }))
}

/// English currency card: ECB daily reference rates (raw XML, parsed in the UI) + gold-api.com gold ounce (USD).
fn kur_ecb() -> Result<Value, String> {
    let ecb = metin("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml")?;
    let ons = body(AGENT.get("https://api.gold-api.com/price/XAU").call()).ok().and_then(|v| v["price"].as_f64());
    Ok(json!({ "ecb": ecb, "ons": ons }))
}

/// Currency and gold: TCMB daily rates XML (raw text, parsed by the UI) + gold-api.com gold ounce (USD).
/// If gold cannot be fetched the rates still return; the UI computes gram gold as ounce × dollar ÷ 31.1035.
/// shortcut: gold-api.com is a documented but unguaranteed free service; if it closes the gold row shows "—".
fn kur() -> Result<Value, String> {
    let tcmb = metin("https://www.tcmb.gov.tr/kurlar/today.xml")?;
    let ons = body(AGENT.get("https://api.gold-api.com/price/XAU").call()).ok().and_then(|v| v["price"].as_f64());
    Ok(json!({ "tcmb": tcmb, "ons": ons }))
}

/// System card: CPU (difference between two readings), memory, system disk, battery.
static CPU_ONCEKI: Mutex<Option<(u64, u64)>> = Mutex::new(None);
/// CPU percent: from the difference of the (idle, total) counters between two readings. None on the first reading or with no difference.
fn cpu_yuzde(once: Option<(u64, u64)>, simdi: (u64, u64)) -> Option<f64> {
    let (b0, t0) = once?;
    let (db, dt) = (simdi.0.checked_sub(b0)?, simdi.1.checked_sub(t0)?);
    (dt > 0).then(|| (100.0 * (1.0 - db as f64 / dt as f64)).clamp(0.0, 100.0))
}
fn sistem() -> Value {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows_sys::Win32::System::Threading::GetSystemTimes;
    let ft = |f: FILETIME| (f.dwHighDateTime as u64) << 32 | f.dwLowDateTime as u64;
    let z = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut bos, mut cek, mut kul) = (z, z, z);
    // kernel time includes idle time: total = kernel + user
    let cpu = (unsafe { GetSystemTimes(&mut bos, &mut cek, &mut kul) } != 0).then(|| (ft(bos), ft(cek) + ft(kul)))
        .and_then(|simdi| cpu_yuzde(kilit(&CPU_ONCEKI).replace(simdi), simdi));
    let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    let bellek = (unsafe { GlobalMemoryStatusEx(&mut m) } != 0).then(|| json!({ "toplam": m.ullTotalPhys, "bos": m.ullAvailPhys }));
    let surucu: Vec<u16> = format!("{}\\", std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into())).encode_utf16().chain([0]).collect();
    let (mut db, mut dt) = (0u64, 0u64);
    let disk = (unsafe { GetDiskFreeSpaceExW(surucu.as_ptr(), &mut db, &mut dt, std::ptr::null_mut()) } != 0).then(|| json!({ "toplam": dt, "bos": db }));
    let mut g: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    // BatteryFlag 128: no battery (desktop); percent 255: unknown
    let pil = (unsafe { GetSystemPowerStatus(&mut g) } != 0 && g.BatteryFlag & 128 == 0 && g.BatteryLifePercent <= 100)
        .then(|| json!({ "yuzde": g.BatteryLifePercent, "sarj": g.ACLineStatus == 1 }));
    json!({ "cpu": cpu, "bellek": bellek, "disk": disk, "pil": pil })
}

/// Clipboard history: kept in memory only, only while the card is open (PANO_ACIK); never written to disk.
static PANO: Mutex<Vec<String>> = Mutex::new(Vec::new());
static PANO_ACIK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn pano_ekle(l: &mut Vec<String>, m: String) {
    if m.trim().is_empty() || m.chars().count() > 5000 {
        return; // empty or very long text (file dumps etc.) is not kept
    }
    l.retain(|x| *x != m);
    l.insert(0, m);
    l.truncate(20);
}
/// Reads plain text from the clipboard. Content carrying password managers' "exclude from history" markers is skipped.
fn pano_oku() -> Option<String> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW};
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    let fmt = |ad: &str| unsafe { RegisterClipboardFormatW(ad.encode_utf16().chain([0]).collect::<Vec<u16>>().as_ptr()) };
    unsafe {
        // the clipboard may be open in another app at that moment: try a few times
        if !(0..5).any(|i| { if i > 0 { thread::sleep(Duration::from_millis(40)); } OpenClipboard(std::ptr::null_mut()) != 0 }) {
            return None;
        }
        let mut metin = None;
        let gecmis = fmt("CanIncludeInClipboardHistory");
        // CanIncludeInClipboardHistory: do not keep if the DWORD is 0; also do not keep if the format exists but its value cannot be read
        let izinsiz = IsClipboardFormatAvailable(fmt("ExcludeClipboardContentFromMonitorProcessing")) != 0
            || (IsClipboardFormatAvailable(gecmis) != 0 && {
                let h = GetClipboardData(gecmis);
                let p = GlobalLock(h) as *const u32;
                let sifir = p.is_null() || GlobalSize(h) < 4 || *p == 0;
                if !p.is_null() { GlobalUnlock(h); }
                sifir
            });
        if !izinsiz {
            let h = GetClipboardData(13); // CF_UNICODETEXT
            let p = GlobalLock(h) as *const u16;
            if !p.is_null() {
                let n = GlobalSize(h) / 2;
                let d = std::slice::from_raw_parts(p, n);
                metin = Some(String::from_utf16_lossy(&d[..d.iter().position(|c| *c == 0).unwrap_or(n)]));
                GlobalUnlock(h);
            }
        }
        CloseClipboard();
        metin
    }
}
fn pano_izle() {
    use std::sync::atomic::Ordering::Relaxed;
    use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;
    let mut son = unsafe { GetClipboardSequenceNumber() };
    loop {
        thread::sleep(Duration::from_millis(700));
        let n = unsafe { GetClipboardSequenceNumber() };
        if n == son { continue; }
        son = n;
        if !PANO_ACIK.load(Relaxed) { continue; } // copies made while it was off are not picked up later
        if let Some(m) = pano_oku() {
            pano_ekle(&mut kilit(&PANO), m);
        }
    }
}
fn pano(b: &str) -> Value {
    use std::sync::atomic::Ordering::Relaxed;
    let r: Value = serde_json::from_str(b).unwrap_or_default();
    if let Some(a) = r["acik"].as_bool() { PANO_ACIK.store(a, Relaxed); }
    if r["temizle"] == true { kilit(&PANO).clear(); }
    json!({ "acik": PANO_ACIK.load(Relaxed), "liste": *kilit(&PANO) })
}

/// OneDrive sync: data.json is paired with <OneDrive>\RYOCS\data.json. esitle.json: {acik, klasor, oz}.
/// oz = hash of the last synced remote file; if the remote file differs from it, another computer wrote it.
fn oz(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    b64(&Sha256::digest(b))
}
fn onedrive() -> Option<PathBuf> {
    let e = read_json("esitle.json");
    e["klasor"].as_str().filter(|s| !s.is_empty()).map(PathBuf::from)
        .or_else(|| ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"].iter().find_map(|k| std::env::var(k).ok()).map(PathBuf::from))
        .or_else(|| std::env::var("USERPROFILE").ok().map(|u| PathBuf::from(u).join("OneDrive")))
        .filter(|p| p.is_dir())
}
/// Pure decision: what to do from the local and remote hashes and the last synced hash.
/// "ok": can be sent · "uzak_yeni": local unchanged, remote can be pulled · "catisma": both changed, ask the user
fn esitle_karar(yerel: Option<&str>, uzak: Option<&str>, son: Option<&str>) -> &'static str {
    match (yerel, uzak) {
        (_, None) => "ok",
        (y, u) if y == u || u == son => "ok",
        (y, _) if y.is_none() || y == son => "uzak_yeni",
        _ => "catisma",
    }
}
fn esitle_durum() -> (&'static str, Option<PathBuf>) {
    let e = read_json("esitle.json");
    if e["acik"] != true { return ("kapali", onedrive()); }
    let Some(k) = onedrive() else { return ("klasor_yok", None) };
    let (y, u) = (fs::read(dir().join("data.json")).ok().map(|b| oz(&b)), fs::read(k.join("RYOCS").join("data.json")).ok().map(|b| oz(&b)));
    (esitle_karar(y.as_deref(), u.as_deref(), e["oz"].as_str()), Some(k))
}
fn esitle_oz_yaz(o: &str) {
    let mut e = read_json("esitle.json");
    e["oz"] = json!(o);
    write_json("esitle.json", &e).ok();
}
/// Writes local to OneDrive (temp file + rename). On conflict or error it quietly gives up: it does not stop the save flow.
fn esitle_gonder() {
    let (d, Some(k)) = esitle_durum() else { return };
    if d != "ok" { return; }
    let Ok(b) = fs::read(dir().join("data.json")) else { return };
    let hedef = k.join("RYOCS");
    let tmp = hedef.join("data.json.tmp");
    let sonuc = fs::create_dir_all(&hedef).and_then(|_| fs::write(&tmp, &b)).and_then(|_| fs::rename(&tmp, hedef.join("data.json")));
    match sonuc {
        Ok(()) => esitle_oz_yaz(&oz(&b)),
        Err(e) => hata_kaydet(&format!("OneDrive sync write failed: {e}")),
    }
}
/// Replaces local with the remote data; local stays as the "State before restore" backup.
fn esitle_cek(k: &std::path::Path) -> Result<(), String> {
    let b = fs::read(k.join("RYOCS").join("data.json")).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(b.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&b)).map_err(|_| l("OneDrive'daki veri dosyası bozuk", "The data file on OneDrive is corrupt"))?;
    if !v.is_object() { return Err(l("OneDrive'daki veri dosyası bozuk", "The data file on OneDrive is corrupt").into()); }
    let (yedek, yerel) = (dir().join("yedekler"), dir().join("data.json"));
    fs::create_dir_all(&yedek).ok();
    fs::copy(&yerel, yedek.join("data-geri-alma.json")).ok();
    // written byte for byte: the local hash becomes the last synced hash, so the next remote change is not mistaken for a "conflict"
    let tmp = yerel.with_extension("tmp");
    fs::write(&tmp, &b).and_then(|_| fs::rename(&tmp, &yerel)).map_err(|e| e.to_string())?;
    esitle_oz_yaz(&oz(&b));
    Ok(())
}
fn esitle(b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).unwrap_or_default();
    if r["acik"].is_boolean() || r["klasor"].is_string() {
        let mut e = read_json("esitle.json");
        if let Some(a) = r["acik"].as_bool() { e["acik"] = json!(a); }
        if let Some(k) = r["klasor"].as_str() {
            if !k.is_empty() && !PathBuf::from(k).is_dir() { return Err(l("Klasör bulunamadı", "Folder not found").into()); }
            e["klasor"] = json!(k);
        }
        write_json("esitle.json", &e)?;
    }
    let (mut d, k) = esitle_durum();
    if let (Some(sec), Some(k)) = (r["sec"].as_str(), k.as_ref()) {
        if d == "catisma" {
            if sec == "uzak" {
                esitle_cek(k)?;
            } else if sec == "yerel" {
                // the losing side is backed up, then overwritten
                let uzak = fs::read(k.join("RYOCS").join("data.json")).map_err(|e| e.to_string())?;
                fs::create_dir_all(dir().join("yedekler")).ok();
                fs::write(dir().join("yedekler").join("data-onedrive.json"), &uzak).map_err(|e| e.to_string())?;
                esitle_oz_yaz(&oz(&uzak));
            }
            d = if sec == "uzak" { "guncellendi" } else { "ok" };
        }
    } else if d == "uzak_yeni" && r["baslangic"] == true {
        if let Some(k) = k.as_ref() { esitle_cek(k)?; d = "guncellendi"; }
    }
    if d == "ok" { esitle_gonder(); }
    Ok(json!({ "durum": d, "klasor": k, "acik": read_json("esitle.json")["acik"] == true }))
}

fn b64(b: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

/// 256-bit random string safe to carry in a URL (state and PKCE verifier).
fn rastgele() -> String {
    let mut b = [0u8; 32];
    getrandom::getrandom(&mut b).expect("random number generator");
    b64(&b)
}

/// PKCE S256: base64url(sha256(verifier))
fn pkce(dogrulayici: &str) -> String {
    use sha2::Digest;
    b64(&sha2::Sha256::digest(dogrulayici.as_bytes()))
}

/// Without a secret (Spotify PKCE) the field is not sent at all.
fn gizli_ekle<'a>(mut f: Vec<(&'a str, &'a str)>, gizli: &'a str) -> Vec<(&'a str, &'a str)> {
    if !gizli.is_empty() {
        f.push(("client_secret", gizli));
    }
    f
}

/// Format of the credentials from the setup dialog (catches pasting the wrong field early).
fn kimlik_gecerli(p: &str, id: &str, gizli: &str) -> Result<(), String> {
    let duz = |s: &str| s.len() <= 200 && s.bytes().all(|c| c.is_ascii_graphic());
    match p {
        "google" if !(id.ends_with(".apps.googleusercontent.com") && duz(id)) =>
            Err(l("Google istemci kimliği \".apps.googleusercontent.com\" ile biter", "A Google client ID ends with \".apps.googleusercontent.com\"").into()),
        "spotify" if !(id.len() == 32 && id.bytes().all(|c| c.is_ascii_hexdigit())) =>
            Err(l("Spotify Client ID 32 karakterlik bir koddur (0-9, a-f)", "A Spotify Client ID is a 32-character code (0-9, a-f)").into()),
        _ if !duz(gizli) => Err(l("Gizli anahtar boşluk ya da özel karakter içeremez", "The client secret cannot contain spaces or special characters").into()),
        _ => Ok(()),
    }
}

/// Reads config.json; if it is corrupt returns an error so it is not overwritten (the credentials the user entered are not lost).
fn config_oku() -> Result<Value, String> {
    fs::read_to_string(dir().join("config.json")).ok()
        .and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok())
        .filter(Value::is_object)
        .ok_or(l("config.json okunamadı; önce dosyayı düzeltin", "Could not read config.json; fix the file first").into())
}

fn config_yaz(c: &Value) -> Result<(), String> {
    write_text("config.json", &serde_json::to_string_pretty(c).unwrap_or_default())
}

/// Setup dialog: saves the credentials and opens the consent page in the browser.
fn creds_set(p: &Provider, b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    let id = r["client_id"].as_str().unwrap_or("").trim();
    let yeni_gizli = r["client_secret"].as_str().unwrap_or("").trim();
    kimlik_gecerli(p.name, id, yeni_gizli)?;
    let mut c = config_oku()?;
    if !c[p.name].is_object() {
        c[p.name] = json!({});
    }
    c[p.name]["client_id"] = json!(id);
    if !yeni_gizli.is_empty() {
        c[p.name]["client_secret"] = json!(yeni_gizli); // if left empty the old one is kept
    }
    if p.name == "google" && c["google"]["client_secret"].as_str().unwrap_or("").is_empty() {
        return Err(l("Google için gizli anahtar (client secret) da gerekli", "Google also needs the client secret").into());
    }
    config_yaz(&c)?;
    disconnect(p)?; // old tokens do not work with new credentials; whoever redoes the setup is reconnecting anyway
    connect(p)
}

fn creds(p: &str) -> Result<(String, String), String> {
    let c = read_json("config.json");
    let id = c[p]["client_id"].as_str().unwrap_or("").trim();
    if id.is_empty() {
        return Err("kimlik_yok".into());
    }
    Ok((id.into(), c[p]["client_secret"].as_str().unwrap_or("").trim().into()))
}

/// For the settings dialog: city and account states (credential/token values are never returned).
fn config_get() -> Value {
    let (c, t) = (read_json("config.json"), read_json("tokens.json"));
    let hesap = |p: &str| {
        if creds(p).is_err() { "kimlik_yok" } else if t[p]["refresh_token"].is_string() { "ok" } else { "bagli_degil" }
    };
    json!({
        "sehir": c["sehir"].as_str().unwrap_or("Istanbul"),
        // only the client ID to fill the setup dialog (never the secret)
        "google_id": c["google"]["client_id"].as_str().unwrap_or(""),
        "spotify_id": c["spotify"]["client_id"].as_str().unwrap_or(""),
        "google_izin": google_tam_izin(), // whether the calendar list permission was granted too
        "google": hesap("google"), "spotify": hesap("spotify"),
        "surum": env!("CARGO_PKG_VERSION"),
        "kisayol": tepsi::kisayol(),
    })
}

/// Validates the city and writes it to config.json; returns the city name found.
fn config_set(b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    // a place chosen in the picker is saved with its coordinates: the name alone can mean several towns
    let y = if r["konum"].is_object() { yer_dogrula(&r["konum"])? } else { konum(sehir_adi(r["sehir"].as_str().unwrap_or(""))?)? };
    let mut c = config_oku()?;
    c["sehir"] = y["ad"].clone();
    c["konum"] = y.clone();
    config_yaz(&c)?;
    Ok(y["ad"].clone())
}

fn disconnect(p: &Provider) -> Result<Value, String> {
    *kilit(&TAKVIM_LISTESI) = None;
    let _d = kilit(&BELIRTEC_DOSYASI);
    let mut all = read_json("tokens.json");
    if let Some(o) = all.as_object_mut() {
        o.remove(p.name);
    }
    write_json("tokens.json", &all)?;
    Ok(Value::Null)
}

fn redirect(p: &str) -> String {
    format!("{ORIGIN}/callback/{p}")
}

fn save_token(p: &str, mut t: Value) {
    if !t.is_object() {
        return; // unexpected response: the old token is kept
    }
    let _d = kilit(&BELIRTEC_DOSYASI);
    let mut all = read_json("tokens.json");
    if !all.is_object() {
        all = json!({});
    }
    if t["refresh_token"].is_null() {
        t["refresh_token"] = all[p]["refresh_token"].clone(); // yenilemede yeni refresh_token gelmeyebilir
    }
    t["expires_at"] = json!(now() + t["expires_in"].as_u64().unwrap_or(3600));
    all[p] = t;
    if let Err(e) = write_json("tokens.json", &all) {
        hata_kaydet(&format!("tokens.json write failed: {e}"));
    }
}

/// Valid access token; refreshes if needed. Error: "kimlik_yok" | "bagli_degil" | message.
fn token(p: &Provider) -> Result<String, String> {
    let (id, secret) = creds(p.name)?;
    let _tek = kilit(&YENILEME[(p.name == "spotify") as usize]); // if another request is refreshing, wait for it and use its result
    let all = read_json("tokens.json");
    let t = &all[p.name];
    if t["expires_at"].as_u64().unwrap_or(0) > now() + 60 {
        if let Some(a) = t["access_token"].as_str() {
            return Ok(a.into());
        }
    }
    let rt = t["refresh_token"].as_str().ok_or("bagli_degil")?;
    let n = body(AGENT.post(p.token).send_form(&gizli_ekle(vec![
        ("grant_type", "refresh_token"), ("refresh_token", rt), ("client_id", &id),
    ], &secret)))?;
    save_token(p.name, n.clone());
    n["access_token"].as_str().map(String::from).ok_or("bagli_degil".into())
}

fn connect(p: &Provider) -> Result<Value, String> {
    let (id, _) = creds(p.name)?;
    let (st, dogrulayici) = (rastgele(), rastgele());
    let meydan = pkce(&dogrulayici);
    *kilit(&OAUTH_STATE) = (st.clone(), dogrulayici);
    let red = redirect(p.name);
    let mut q = vec![
        ("client_id", id.as_str()), ("response_type", "code"),
        ("redirect_uri", &red), ("scope", p.scope), ("state", &st),
        ("code_challenge", &meydan), ("code_challenge_method", "S256"),
    ];
    q.extend_from_slice(p.extra);
    let url = url::Url::parse_with_params(p.auth, &q).map_err(|e| e.to_string())?;
    open_url(url.as_str())
}

/// Opens in the default browser; http(s) only (blocks running files/programs).
/// Target allowed in the launcher: an existing .exe/.lnk/.url/.appref-ms file or a game store link
fn uygulama_gecerli(yol: &str) -> Result<(), String> {
    if let Some((sema, _)) = yol.split_once("://") {
        return match sema {
            "steam" | "com.epicgames.launcher" => Ok(()),
            _ => Err(l("Yalnızca steam:// ve Epic bağlantıları açılabilir", "Only steam:// and Epic links can be opened").into()),
        };
    }
    let p = std::path::Path::new(yol);
    let uzanti = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(uzanti.as_str(), "exe" | "lnk" | "url" | "appref-ms") {
        return Err(l("Yalnızca .exe, .lnk ve .url dosyaları eklenebilir", "Only .exe, .lnk and .url files can be added").into());
    }
    if !p.is_absolute() || !p.is_file() {
        return Err(l("Dosya bulunamadı (taşınmış ya da silinmiş olabilir)", "File not found (it may have been moved or deleted)").into());
    }
    Ok(())
}

fn uygulama_ac(yol: &str) -> Result<Value, String> {
    uygulama_gecerli(yol)?;
    let p = std::path::Path::new(yol);
    // .exe runs in its own folder (games look for their files there); if it needs admin rights the shell handles it
    if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        && std::process::Command::new(p).current_dir(p.parent().unwrap_or(p)).spawn().is_ok()
    {
        return Ok(Value::Null);
    }
    quiet("explorer").arg(yol).spawn().map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

/// PowerShell script; input is passed only through an environment variable (no text is spliced into the script), output is UTF-8
fn ps(betik: &str, girdi: &str) -> Result<String, String> {
    let o = quiet("powershell")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &format!("[Console]::OutputEncoding=[Text.Encoding]::UTF8;{betik}")])
        .env("RYOCS_GIRDI", girdi)
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// File icons (PNG, data: URL). For a shortcut the target's icon is used: no Windows arrow overlay
fn simgeler(b: &str) -> Result<Value, String> {
    let v: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    let yollar: Vec<&str> = v["yollar"].as_array().into_iter().flatten().filter_map(|y| y.as_str())
        .filter(|y| !y.contains("://") && uygulama_gecerli(y).is_ok()).take(80).collect();
    let cikti = ps(r#"Add-Type -AssemblyName System.Drawing;$o=@{};$k=New-Object -ComObject WScript.Shell;foreach($p in ($env:RYOCS_GIRDI|ConvertFrom-Json)){try{$h=$p;if($p -like '*.lnk'){$s=$k.CreateShortcut($p);$i=($s.IconLocation -split ',')[0];$h=@($i,$s.TargetPath,$p)|?{$_ -and (Test-Path -LiteralPath $_ -PathType Leaf)}|select -First 1}elseif($p -like '*.url'){$i=(Select-String -LiteralPath $p -Pattern '^IconFile=(.+)$'|select -First 1).Matches.Groups[1].Value;if($i -and (Test-Path -LiteralPath $i)){$h=$i}};$b=[System.Drawing.Icon]::ExtractAssociatedIcon($h).ToBitmap();$m=New-Object IO.MemoryStream;$b.Save($m,[System.Drawing.Imaging.ImageFormat]::Png);$o[[string]$p]='data:image/png;base64,'+[Convert]::ToBase64String($m.ToArray())}catch{}};$o|ConvertTo-Json -Compress"#, &serde_json::to_string(&yollar).unwrap_or_default())?;
    Ok(serde_json::from_str(&cikti).unwrap_or(json!({})))
}

/// Desktop shortcuts (user and public desktop; including a desktop moved to OneDrive)
fn kisayollar() -> Result<Value, String> {
    let mut dizinler: Vec<PathBuf> = ["USERPROFILE", "OneDrive", "PUBLIC"].iter().filter_map(std::env::var_os).map(|d| PathBuf::from(d).join("Desktop")).collect();
    dizinler.dedup();
    let mut l: Vec<Value> = dizinler.iter().filter_map(|d| fs::read_dir(d).ok()).flatten().flatten().map(|e| e.path())
        .filter(|p| uygulama_gecerli(&p.to_string_lossy()).is_ok())
        .map(|p| json!({ "ad": p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(), "yol": p.to_string_lossy() }))
        .collect();
    l.sort_by_key(|v| v["ad"].as_str().unwrap_or("").to_lowercase());
    l.dedup_by(|a, b| a["ad"] == b["ad"]);
    Ok(json!(l))
}

/// Windows file picker (opens on top); empty if cancelled
fn dosya_sec() -> Result<Value, String> {
    let y = ps(l(
        r#"Add-Type -AssemblyName System.Windows.Forms;$f=New-Object System.Windows.Forms.Form -Property @{TopMost=$true};$d=New-Object System.Windows.Forms.OpenFileDialog;$d.Title='Uygulama ya da oyun seç';$d.Filter='Uygulamalar ve kısayollar|*.exe;*.lnk;*.url;*.appref-ms';if($d.ShowDialog($f) -eq 'OK'){$d.FileName}"#,
        r#"Add-Type -AssemblyName System.Windows.Forms;$f=New-Object System.Windows.Forms.Form -Property @{TopMost=$true};$d=New-Object System.Windows.Forms.OpenFileDialog;$d.Title='Choose an app or game';$d.Filter='Apps and shortcuts|*.exe;*.lnk;*.url;*.appref-ms';if($d.ShowDialog($f) -eq 'OK'){$d.FileName}"#,
    ), "")?;
    Ok(json!(y))
}

fn open_url(u: &str) -> Result<Value, String> {
    let url = url::Url::parse(u).map_err(|_| l("Geçersiz adres", "Invalid address").to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(l("Yalnızca http/https adresleri açılabilir", "Only http/https addresses can be opened").into());
    }
    // no shell: & signs in the URL cause no trouble
    quiet("rundll32").args(["url.dll,FileProtocolHandler", url.as_str()]).spawn().map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

/// Child process that runs without opening a console window.
fn quiet(cmd: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut c = std::process::Command::new(cmd);
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    c
}

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

fn autostart(set: Option<bool>) -> Result<Value, String> {
    if let Some(on) = set {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut c = quiet("reg");
        if on {
            c.args(["add", RUN_KEY, "/v", "RYOCS", "/t", "REG_SZ", "/d", &format!("\"{}\" --gizli", exe.display()), "/f"]);
        } else {
            c.args(["delete", RUN_KEY, "/v", "RYOCS", "/f"]);
        }
        c.output().map_err(|e| e.to_string())?;
    }
    let on = quiet("reg").args(["query", RUN_KEY, "/v", "RYOCS"]).output().map(|o| o.status.success()).unwrap_or(false);
    Ok(json!(on))
}

fn notify(b: &str) -> Result<Value, String> {
    use tauri_winrt_notification::Toast;
    let r: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    let mut t = Toast::new(AUMID).title(r["baslik"].as_str().unwrap_or("RYOCS")).text1(r["metin"].as_str().unwrap_or(""));
    if let Some(m) = r["metin2"].as_str().filter(|m| !m.is_empty()) {
        t = t.text2(m); // an empty second line is not added
    }
    t.show().map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

/// Registers the app ID so notifications show the "RYOCS" name and icon as the source.
fn aumid_kaydet() {
    let ikon = dir().join("ryocs.png");
    // written at every start: after an icon update notifications must not show the old one
    fs::write(&ikon, include_bytes!("../assets/ryocs.png")).ok();
    let key = format!(r"HKCU\Software\Classes\AppUserModelId\{AUMID}");
    quiet("reg").args(["add", &key, "/v", "DisplayName", "/d", "RYOCS", "/f"]).status().ok();
    quiet("reg").args(["add", &key, "/v", "IconUri", "/d", &ikon.display().to_string(), "/f"]).status().ok();
}

/// YYYY-MM-DD from a day count (since 1970) (Howard Hinnant civil_from_days).
fn ymd(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let (d, m) = (doy - (153 * mp + 2) / 5 + 1, if mp < 10 { mp + 3 } else { mp - 9 });
    format!("{:04}-{:02}-{:02}", yoe + era * 400 + (m <= 2) as i64, m, d)
}

/// Backs up data.json before the first save of the day; keeps the last 14 days.
fn backup() {
    let b = dir().join("yedekler");
    fs::create_dir_all(&b).ok();
    let f = b.join(format!("data-{}.json", ymd((now() / 86_400) as i64)));
    if !f.exists() && fs::copy(dir().join("data.json"), &f).is_err() {
        return;
    }
    // only dated files (data-YYYY-MM-DD.json) count toward the 14; the undo / OneDrive copies stay
    let mut all: Vec<_> = fs::read_dir(&b).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| yedek_adi(n) && n.len() == 20)).collect();
    all.sort();
    for p in all.iter().rev().skip(14) {
        fs::remove_file(p).ok();
    }
}

/// Connection result page shown in the browser; says what to do for common errors.
fn sonuc_sayfasi(servis: &str, sonuc: Result<(), String>) -> String {
    let (ok, baslik, metin) = match sonuc {
        Ok(()) => (true, format!("{servis} {}", l("bağlandı", "connected")), l("Bu sekmeyi kapatıp RYOCS'a dönebilirsiniz.", "You can close this tab and return to RYOCS.").to_string()),
        Err(e) => {
            let ipucu = if e.contains("access_denied") {
                l("İzin verilmedi. Google'da \"erişim engellendi\" gördüyseniz Google hesabınızı izin ekranındaki test kullanıcılarına ekleyin.", "Permission was not granted. If Google said \"access blocked\", add your Google account to the test users on the consent screen.").to_string()
            } else if e.contains("invalid_client") || e.contains("unauthorized_client") {
                l("İstemci kimliği ya da gizli anahtar hatalı. RYOCS'ta Ayarlar → Hesaplar'dan düzeltin.", "The client ID or secret is wrong. Fix it in RYOCS under Settings → Accounts.").to_string()
            } else {
                e.replace('&', "&amp;").replace('<', "&lt;")
            };
            (false, format!("{servis} {}", l("bağlanamadı", "could not connect")), ipucu)
        }
    };
    const SAYFA: &str = r#"<!doctype html><meta charset=utf-8><title>RYOCS</title><style>
body{margin:0;min-height:100vh;display:grid;place-items:center;background:#e9eaec;color:#4a4d55;font:15px "Segoe UI",sans-serif}
main{padding:40px 48px;border-radius:28px;box-shadow:8px 8px 18px #c5c7cc,-8px -8px 18px #fff;max-width:440px;text-align:center}
i{display:grid;place-items:center;width:64px;height:64px;margin:0 auto 18px;border-radius:50%;font:normal 30px "Segoe UI";color:#fff;background:$RENK}
h1{font-weight:400;font-size:20px;margin:0 0 8px}p{color:#747780;line-height:1.5;margin:0}small{display:block;margin-top:22px;letter-spacing:4px;color:#a0a3aa}
</style><main><i>$SIMGE</i><h1>$BASLIK</h1><p>$METIN</p><small>RYOCS</small></main>"#;
    SAYFA.replace("$RENK", if ok { "#8fb996" } else { "#e9a25b" }).replace("$SIMGE", if ok { "✓" } else { "!" })
        .replace("$BASLIK", &baslik).replace("$METIN", &metin)
}

fn callback(p: &Provider, path_and_query: &str) -> Result<(), String> {
    let u = url::Url::parse(&format!("{ORIGIN}{path_and_query}")).map_err(|e| e.to_string())?;
    let q: HashMap<String, String> = u.query_pairs().into_owned().collect();
    let (st, dogrulayici) = std::mem::take(&mut *kilit(&OAUTH_STATE));
    if st.is_empty() || q.get("state") != Some(&st) {
        return Err(l("Geçersiz istek (state uyuşmuyor). Uygulamadan yeniden bağlanın.", "Invalid request (state mismatch). Connect again from the app.").into());
    }
    let code = q.get("code").ok_or_else(|| format!("{} {}", l("İzin verilmedi:", "Permission denied:"), q.get("error").map_or("", |s| s)))?;
    let (id, secret) = creds(p.name)?;
    let red = redirect(p.name);
    let t = body(AGENT.post(p.token).send_form(&gizli_ekle(vec![
        ("grant_type", "authorization_code"), ("code", code), ("redirect_uri", &red),
        ("client_id", &id), ("code_verifier", &dogrulayici),
    ], &secret)))?;
    save_token(p.name, t);
    Ok(())
}

const TR_TATIL: &str = "tr.turkish#holiday@group.v.calendar.google.com";

/// Calendar ID: e-mail, "primary" or "...#holiday@group.v.calendar.google.com" forms.
fn takvim_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"@._#-+".contains(&c))
}

/// calendars/{calendar}/... address; # and @ in the ID are encoded as a path segment.
fn takvim_url(takvim: &str, devam: &[&str]) -> Result<url::Url, String> {
    let mut u = url::Url::parse("https://www.googleapis.com/calendar/v3/calendars").map_err(|e| e.to_string())?;
    u.path_segments_mut().map_err(|_| "adres".to_string())?.push(takvim).extend(devam);
    Ok(u)
}

/// Gmail (read-only): unread count in the inbox and the headers of the last 10 unread messages.
/// If an old connection lacks the Gmail permission: "izin_yok", the UI suggests reconnecting. RYOCS never changes anything in the mailbox.
fn mail() -> Result<Value, String> {
    let auth = format!("Bearer {}", token(&provider("google")?)?);
    if !read_json("tokens.json")["google"]["scope"].as_str().is_some_and(|s| s.contains("gmail.readonly")) {
        return Err("izin_yok".into());
    }
    let g = "https://gmail.googleapis.com/gmail/v1/users/me";
    let kutu = body(AGENT.get(&format!("{g}/labels/INBOX")).query("fields", "messagesUnread").set("Authorization", &auth).call())?;
    let liste = body(AGENT.get(&format!("{g}/messages")).query("q", "is:unread in:inbox").query("maxResults", "10")
        .query("fields", "messages(id)").set("Authorization", &auth).call())?;
    // the ID goes into the URL path: letters/digits only
    let idler: Vec<String> = liste["messages"].as_array().into_iter().flatten().filter_map(|m| m["id"].as_str())
        .filter(|i| !i.is_empty() && i.len() <= 64 && i.bytes().all(|c| c.is_ascii_alphanumeric())).map(String::from).collect();
    let mailler: Vec<Value> = thread::scope(|k| {
        let isler: Vec<_> = idler.iter().map(|id| { let auth = &auth; k.spawn(move || {
            body(AGENT.get(&format!("{g}/messages/{id}")).query("format", "metadata")
                .query("metadataHeaders", "From").query("metadataHeaders", "Subject")
                .query("fields", "id,threadId,internalDate,payload/headers").set("Authorization", auth).call())
        })}).collect();
        isler.into_iter().filter_map(|h| h.join().ok()?.ok()).collect()
    });
    let baslik = |m: &Value, ad: &str| m["payload"]["headers"].as_array().into_iter().flatten()
        .find(|h| h["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(ad))).and_then(|h| h["value"].as_str()).unwrap_or("").to_string();
    let liste: Vec<Value> = mailler.iter().map(|m| json!({
        "id": m["id"], "konu": baslik(m, "Subject"), "kimden": baslik(m, "From"),         "zincir": m["threadId"], "t": m["internalDate"].as_str().and_then(|t| t.parse::<i64>().ok()),
    })).collect();
    Ok(json!({ "okunmamis": kutu["messagesUnread"], "liste": liste }))
}

fn google_tam_izin() -> bool {
    read_json("tokens.json")["google"]["scope"].as_str().is_some_and(|s| s.contains("calendar.calendarlist"))
}

/// Events of all visible calendars (and the regional holidays); each event gets its calendar as "_t".
fn calendar(req_body: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(req_body).unwrap_or_default();
    let auth = format!("Bearer {}", token(&provider("google")?)?);
    let (min, max) = (r["min"].as_str().unwrap_or(""), r["max"].as_str().unwrap_or(""));
    let onbellek = kilit(&TAKVIM_LISTESI).as_ref().filter(|(t, _)| t.elapsed() < Duration::from_secs(600)).map(|(_, v)| v.clone());
    let liste = match onbellek {
        Some(v) => Ok(v),
        None => body(AGENT.get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
            .query("fields", "items(id,summary,summaryOverride,primary,selected,backgroundColor,accessRole)")
            .set("Authorization", &auth).call()),
    };
    if let Ok(v) = &liste {
        *kilit(&TAKVIM_LISTESI) = Some((std::time::Instant::now(), v.clone()));
    }
    // 403: an old connection only has the events permission; continue with the primary calendar, the UI suggests updating the permission
    let eksik_izin = matches!(&liste, Err(e) if e.starts_with("403"));
    let mut takvimler: Vec<Value> = match liste {
        Ok(l) => l["items"].as_array().cloned().unwrap_or_default().into_iter()
            .filter(|c| c["selected"].as_bool() == Some(true) || c["primary"].as_bool() == Some(true))
            .map(|c| json!({
                "id": c["id"],
                // the primary calendar's name is the e-mail address; show the user "Main calendar"
                "ad": c["summaryOverride"].as_str()
                    .or(if c["primary"].as_bool() == Some(true) { Some(crate::l("Ana takvim", "Main calendar")) } else { c["summary"].as_str() })
                    .unwrap_or(crate::l("Takvim", "Calendar")),
                "renk": c["backgroundColor"].as_str().unwrap_or("#8fb996"),
                "yaz": matches!(c["accessRole"].as_str(), Some("owner" | "writer")),
            }))
            .collect(),
        Err(_) if eksik_izin => vec![],
        Err(e) => return Err(e),
    };
    if takvimler.is_empty() {
        takvimler.push(json!({ "id": "primary", "ad": l("Takvim", "Calendar"), "renk": "#8fb996", "yaz": true }));
    }
    let tatil = r["tatil"].as_str().filter(|t| takvim_id(t) && t.ends_with("#holiday@group.v.calendar.google.com")).unwrap_or(TR_TATIL);
    if !takvimler.iter().any(|c| c["id"] == tatil) {
        let ad = if tatil == TR_TATIL { l("Türkiye'deki tatiller", "Holidays in Turkey") } else { l("Tatiller", "Holidays") };
        takvimler.push(json!({ "id": tatil, "ad": ad, "renk": "#e9a25b", "yaz": false }));
    }
    let sonuc: Vec<Result<Value, String>> = thread::scope(|k| {
        let isler: Vec<_> = takvimler.iter().map(|c| {
            let (id, auth) = (c["id"].as_str().unwrap_or("primary").to_string(), &auth);
            k.spawn(move || {
                body(AGENT.request_url("GET", &takvim_url(&id, &["events"])?)
                    .query("timeMin", min).query("timeMax", max)
                    .query("singleEvents", "true").query("orderBy", "startTime").query("maxResults", "250")
                    // only the fields the UI uses: on a slow network the response is several times smaller
                    .query("fields", "items(id,summary,start,end,recurringEventId,location,htmlLink,extendedProperties,description,eventType)")
                    .set("Authorization", auth).call())
            })
        }).collect();
        isler.into_iter().map(|h| h.join().unwrap_or(Err("thread".into()))).collect()
    });
    let (mut items, mut hatalar) = (vec![], vec![]);
    for (c, r) in takvimler.iter().zip(sonuc) {
        match r {
            Ok(v) => items.extend(v["items"].as_array().cloned().unwrap_or_default().into_iter().map(|mut e| {
                e["_t"] = c["id"].clone();
                e
            })),
            Err(e) => hatalar.push(json!({ "ad": c["ad"], "hata": e })),
        }
    }
    if items.is_empty() && hatalar.len() == takvimler.len() {
        return Err(hatalar[0]["hata"].as_str().unwrap_or(l("Takvim alınamadı", "Could not load the calendar")).into()); // none could be fetched: a connection problem
    }
    Ok(json!({ "items": items, "takvimler": takvimler, "eksik_izin": eksik_izin, "hatalar": hatalar }))
}

/// Adds an event. Important dates (types) can repeat every year, do not show as busy, and keep their type in the event.
fn calendar_add(req_body: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(req_body).map_err(|e| e.to_string())?;
    if r["summary"].as_str().is_none_or(|s| s.trim().is_empty()) || !r["start"].is_object() || !r["end"].is_object() {
        return Err(l("Başlık ve tarih gerekli", "Title and date are required").into());
    }
    let takvim = r["takvim"].as_str().unwrap_or("primary");
    if !takvim_id(takvim) {
        return Err(l("Geçersiz takvim", "Invalid calendar").into());
    }
    let mut ev = json!({ "summary": r["summary"], "start": r["start"], "end": r["end"] });
    if r["yillik"].as_bool() == Some(true) {
        ev["recurrence"] = json!(["RRULE:FREQ=YEARLY"]);
    }
    if let Some(tur) = r["tur"].as_str().filter(|t| matches!(*t, "dogum" | "yildonumu" | "onemli")) {
        ev["transparency"] = json!("transparent");
        ev["extendedProperties"] = json!({ "private": { "ryocs": tur } });
        if let Some(y) = r["yil"].as_u64().filter(|y| (1900..=2100).contains(y)) {
            ev["extendedProperties"]["private"]["yil"] = json!(y.to_string()); // for age / anniversary year
        }
    }
    let t = token(&provider("google")?)?;
    body(AGENT.request_url("POST", &takvim_url(takvim, &["events"])?).set("Authorization", &format!("Bearer {t}")).send_json(ev))
}

/// Google event ID: letters, digits and underscore only (since it is added to the address).
fn etkinlik_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 1024 && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

fn calendar_delete(b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    let id = r["id"].as_str().filter(|s| etkinlik_id(s)).ok_or(l("Geçersiz etkinlik", "Invalid event"))?;
    let takvim = r["takvim"].as_str().unwrap_or("primary");
    if !takvim_id(takvim) {
        return Err(l("Geçersiz takvim", "Invalid calendar").into());
    }
    let t = token(&provider("google")?)?;
    body(AGENT.request_url("DELETE", &takvim_url(takvim, &["events", id])?).set("Authorization", &format!("Bearer {t}")).call())
}

/// Only names we generate ourselves are read from the backup folder (no escaping the path).
fn yedek_adi(s: &str) -> bool {
    s == "data-geri-alma.json" || s == "data-onedrive.json"
        || (s.len() == 20 && s.starts_with("data-") && s.ends_with(".json")
            && s[5..15].bytes().enumerate().all(|(i, c)| if i == 4 || i == 7 { c == b'-' } else { c.is_ascii_digit() }))
}

fn yedek_listesi() -> Value {
    let mut v: Vec<String> = fs::read_dir(dir().join("yedekler")).into_iter().flatten().flatten()
        .filter_map(|e| e.file_name().into_string().ok()).filter(|s| yedek_adi(s)).collect();
    v.sort();
    v.reverse();
    json!(v)
}

/// Replaces the data with new data; keeps the current state as the "undo" backup.
fn geri_yukle(v: Value) -> Result<Value, String> {
    if !v.is_object() {
        return Err(l("Geçersiz yedek dosyası", "Invalid backup file").into());
    }
    let b = dir().join("yedekler");
    fs::create_dir_all(&b).ok();
    fs::copy(dir().join("data.json"), b.join("data-geri-alma.json")).ok();
    write_json("data.json", &v)?;
    Ok(Value::Null)
}

fn yedekten(b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    let ad = r["ad"].as_str().filter(|s| yedek_adi(s)).ok_or(l("Geçersiz yedek adı", "Invalid backup name"))?;
    // read first: geri_yukle overwrites the "undo" file
    let s = fs::read_to_string(dir().join("yedekler").join(ad)).map_err(|e| e.to_string())?;
    geri_yukle(serde_json::from_str(s.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?)
}

fn disa_aktar() -> Result<Value, String> {
    use std::os::windows::process::CommandExt;
    // shortcut: the default location of Downloads is assumed; if it was moved, use the known folder API (SHGetKnownFolderPath)
    let hedef = PathBuf::from(std::env::var("USERPROFILE").map_err(|e| e.to_string())?)
        .join("Downloads").join(format!("RYOCS-yedek-{}.json", ymd((now() / 86_400) as i64)));
    fs::copy(dir().join("data.json"), &hedef).map_err(|e| e.to_string())?;
    quiet("explorer").raw_arg(format!("/select,\"{}\"", hedef.display())).spawn().ok();
    Ok(json!(hedef))
}

fn spotify(cmd: &str, b: &str) -> Result<Value, String> {
    let auth = format!("Bearer {}", token(&provider("spotify")?)?);
    let base = "https://api.spotify.com/v1/me/player";
    body(match cmd {
        // market: so the hundreds of country codes (available_markets) are not returned
        "" => AGENT.get(base).query("market", "from_token").set("Authorization", &auth).call(),
        "play" | "pause" => AGENT.put(&format!("{base}/{cmd}")).set("Authorization", &auth).send_string(""),
        "next" | "previous" => AGENT.post(&format!("{base}/{cmd}")).set("Authorization", &auth).send_string(""),
        "queue" => AGENT.get(&format!("{base}/queue")).set("Authorization", &auth).call(),
        "volume" => {
            let v = serde_json::from_str::<Value>(b).ok().and_then(|v| v["v"].as_u64()).filter(|v| *v <= 100).ok_or(l("Geçersiz ses düzeyi", "Invalid volume"))?;
            AGENT.put(&format!("{base}/volume")).query("volume_percent", &v.to_string()).set("Authorization", &auth).send_string("")
        }
        _ => return Err("bilinmeyen komut".into()),
    })
}

/// Only requests from our own page: blocks DNS rebinding (Host) and POSTs from other sites (Origin).
fn allowed(host: Option<&str>, origin: Option<&str>) -> bool {
    host == Some(ADDR) && origin.is_none_or(|o| o == ORIGIN)
}

fn send(req: Request, code: u16, ctype: &str, text: String) {
    let h = Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes()).unwrap();
    req.respond(Response::from_string(text).with_status_code(code).with_header(h)).ok();
}

fn handle(mut req: Request) {
    let hdr = |n: &'static str| req.headers().iter().find(|h| h.field.equiv(n)).map(|h| h.value.as_str().to_string());
    let (host, origin) = (hdr("Host"), hdr("Origin"));
    if !allowed(host.as_deref(), origin.as_deref()) {
        return send(req, 403, "text/plain", "yasak".into());
    }
    let mut b = String::new();
    req.as_reader().take(1 << 20).read_to_string(&mut b).ok();
    let url = req.url().to_string();
    let path = url.split('?').next().unwrap_or("");
    let post = *req.method() == Method::Post;
    const HTML: &str = "text/html; charset=utf-8";

    if !post && path == "/" {
        return send(req, 200, HTML, include_str!("ui.html").into());
    }
    if let (false, Some(name)) = (post, path.strip_prefix("/callback/")) {
        let sonuc = provider(name).and_then(|p| callback(&p, &url));
        if sonuc.is_ok() {
            olay(Olay::Goster(true)); // the user should return from the browser to the app
        }
        let servis = if name == "google" { l("Google Takvim", "Google Calendar") } else { "Spotify" };
        return send(req, 200, HTML, sonuc_sayfasi(servis, sonuc));
    }
    let out: Result<Value, String> = if !post && path == "/api/state" {
        veri_oku()
    } else if post && path == "/api/state" {
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string())
            .and_then(|v| { backup(); write_json("data.json", &v)?; esitle_gonder(); Ok(Value::Null) })
    } else if post && path == "/api/calendar/add" {
        calendar_add(&b)
    } else if post && path == "/api/notify" {
        notify(&b)
    } else if post && path == "/api/open" {
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()).and_then(|v| open_url(v["url"].as_str().unwrap_or("")))
    } else if post && path == "/api/uygulama" {
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()).and_then(|v| uygulama_ac(v["yol"].as_str().unwrap_or("")))
    } else if post && path == "/api/simgeler" {
        simgeler(&b)
    } else if !post && path == "/api/kisayollar" {
        kisayollar()
    } else if post && path == "/api/dosya-sec" {
        dosya_sec()
    } else if post && path == "/api/backups" {
        backup();
        quiet("explorer").arg(dir().join("yedekler")).spawn().map(|_| Value::Null).map_err(|e| e.to_string())
    } else if post && path == "/api/log" {
        let v: Value = serde_json::from_str(&b).unwrap_or_default();
        hata_kaydet(&format!("ui: {}", v["mesaj"].as_str().unwrap_or("?").chars().take(2000).collect::<String>()));
        Ok(Value::Null)
    } else if post && path == "/api/show" {
        olay(Olay::Goster(true)); // a second instance tried to start
        Ok(Value::Null)
    } else if post && path == "/api/calendar/delete" {
        calendar_delete(&b)
    } else if !post && path == "/api/backups/list" {
        Ok(yedek_listesi())
    } else if post && path == "/api/backups/restore" {
        yedekten(&b)
    } else if post && path == "/api/import" {
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()).and_then(|v| geri_yukle(v["veri"].clone()))
    } else if post && path == "/api/export" {
        disa_aktar()
    } else if !post && path == "/api/config" {
        Ok(config_get())
    } else if post && path == "/api/config" {
        config_set(&b)
    } else if post && path == "/api/config/open" {
        quiet("notepad").arg(dir().join("config.json")).spawn().map(|_| Value::Null).map_err(|e| e.to_string())
    } else if let (true, Some(n)) = (post, path.strip_prefix("/api/creds/")) {
        provider(n).and_then(|p| creds_set(&p, &b))
    } else if let (true, Some(n)) = (post, path.strip_prefix("/api/disconnect/")) {
        provider(n).and_then(|p| disconnect(&p))
    } else if path == "/api/autostart" {
        autostart(post.then(|| serde_json::from_str::<Value>(&b).ok()?["on"].as_bool()).flatten())
    } else if !post && path == "/api/weather" {
        url::Url::parse(&format!("{ORIGIN}{url}")).map_err(|e| e.to_string())
            .and_then(|u| weather(&u.query_pairs().into_owned().collect()))
    } else if !post && path == "/api/sehir_ara" {
        url::Url::parse(&format!("{ORIGIN}{url}")).map_err(|e| e.to_string())
            .and_then(|u| sehir_ara(&u.query_pairs().find(|(k, _)| k == "q").map(|(_, v)| v.into_owned()).unwrap_or_default()))
    } else if !post && path == "/api/guncelleme" {
        guncelleme()
    } else if !post && path == "/api/mail" {
        mail()
    } else if !post && path == "/api/kur_ecb" {
        kur_ecb()
    } else if !post && path == "/api/kur" {
        kur()
    } else if !post && path == "/api/sistem" {
        Ok(sistem())
    } else if post && path == "/api/pano" {
        Ok(pano(&b))
    } else if post && path == "/api/esitle" {
        esitle(&b)
    } else if post && path == "/api/haber" {
        haber(&b)
    } else if post && path == "/api/calendar" {
        calendar(&b)
    } else if !post && path == "/api/spotify" {
        spotify("", "")
    } else if let (true, Some(c)) = (post, path.strip_prefix("/api/spotify/")) {
        spotify(c, &b)
    } else if let (true, Some(n)) = (post, path.strip_prefix("/api/connect/")) {
        provider(n).and_then(|p| connect(&p))
    } else {
        return send(req, 404, "text/plain", "yok".into());
    };
    let j = match out {
        Ok(v) => json!({ "durum": "ok", "veri": v }),
        Err(e) => json!({ "durum": e, "yol": dir().join("config.json") }),
    };
    send(req, 200, "application/json; charset=utf-8", j.to_string());
}

/// Notification area icon and global shortcut: on a separate thread with its own hidden window and message loop.
mod tepsi {
    use super::{olay, Olay};
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicUsize, Ordering::Relaxed};
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    use windows_sys::Win32::UI::Shell::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const WM_TEPSI: u32 = WM_APP + 1;
    const AC: i32 = 1;
    const CIK: i32 = 2;
    static PENCERE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
    static TASKBAR: AtomicU32 = AtomicU32::new(0);
    /// Tried in order; Windows 11 reserves Win+Shift+R for screen recording, so the first free one is taken.
    const KISAYOLLAR: [(u32, char, &str); 3] = [
        (MOD_WIN | MOD_CONTROL, 'R', "Win+Ctrl+R"),
        (MOD_WIN | MOD_ALT, 'Y', "Win+Alt+Y"),
        (MOD_CONTROL | MOD_ALT | MOD_SHIFT, 'R', "Ctrl+Alt+Shift+R"),
    ];
    static ALINAN: AtomicUsize = AtomicUsize::new(usize::MAX);
    pub fn kisayol() -> Option<&'static str> {
        KISAYOLLAR.get(ALINAN.load(Relaxed)).map(|k| k.2)
    }

    pub fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    unsafe fn veri(h: HWND) -> NOTIFYICONDATAW {
        let mut d: NOTIFYICONDATAW = std::mem::zeroed();
        d.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        d.hWnd = h;
        d.uID = 1;
        d
    }

    unsafe fn ekle(h: HWND) {
        let mut d = veri(h);
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        d.uCallbackMessage = WM_TEPSI;
        d.hIcon = LoadImageW(GetModuleHandleW(null()), 1 as _, IMAGE_ICON, GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_DEFAULTCOLOR);
        let tip = w(&kisayol().map_or("RYOCS".into(), |k| format!("RYOCS ({k})")));
        d.szTip[..tip.len()].copy_from_slice(&tip);
        Shell_NotifyIconW(NIM_ADD, &d);
    }

    pub fn kaldir() {
        let h = PENCERE.load(Relaxed);
        if !h.is_null() {
            unsafe { Shell_NotifyIconW(NIM_DELETE, &veri(h)) };
        }
    }

    unsafe extern "system" fn proc(h: HWND, m: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        match m {
            WM_HOTKEY => olay(Olay::Degistir),
            WM_TEPSI => match lp as u32 {
                WM_LBUTTONUP => olay(Olay::Goster(true)),
                WM_RBUTTONUP => {
                    let menu = CreatePopupMenu();
                    AppendMenuW(menu, MF_STRING, AC as usize, w(crate::l("RYOCS'u aç", "Open RYOCS")).as_ptr());
                    AppendMenuW(menu, MF_SEPARATOR, 0, null());
                    AppendMenuW(menu, MF_STRING, CIK as usize, w(crate::l("Çıkış", "Exit")).as_ptr());
                    let mut p = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut p);
                    SetForegroundWindow(h); // otherwise it does not close on a click outside the menu
                    let secim = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, p.x, p.y, 0, h, null());
                    DestroyMenu(menu);
                    match secim {
                        AC => olay(Olay::Goster(true)),
                        CIK => olay(Olay::Kapat),
                        _ => {}
                    }
                }
                _ => {}
            },
            _ if m != 0 && m == TASKBAR.load(Relaxed) => ekle(h), // Explorer restarted: the icon must not disappear
            _ => return DefWindowProcW(h, m, wp, lp),
        }
        0
    }

    pub fn baslat() {
        std::thread::spawn(|| unsafe {
            let inst = GetModuleHandleW(null());
            let cls = w("RYOCS_tepsi");
            let mut wc: WNDCLASSW = std::mem::zeroed();
            wc.lpfnWndProc = Some(proc);
            wc.hInstance = inst;
            wc.lpszClassName = cls.as_ptr();
            RegisterClassW(&wc);
            TASKBAR.store(RegisterWindowMessageW(w("TaskbarCreated").as_ptr()), Relaxed);
            // invisible top-level window (a message-only window does not get the "TaskbarCreated" broadcast)
            let h = CreateWindowExW(0, cls.as_ptr(), cls.as_ptr(), 0, 0, 0, 0, 0, null_mut(), null_mut(), inst, null());
            PENCERE.store(h, Relaxed);
            if let Some(i) = KISAYOLLAR.iter().position(|(m, k, _)| RegisterHotKey(h, 1, m | MOD_NOREPEAT, *k as u32) != 0) {
                ALINAN.store(i, Relaxed);
            }
            ekle(h);
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });
    }
}

fn main() {
    std::panic::set_hook(Box::new(|bilgi| {
        let yer = bilgi.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let is = thread::current().name().unwrap_or("?").to_string();
        hata_kaydet(&format!("PANIC [{is}] {yer}: {}\n{}", bilgi.payload_as_str().unwrap_or("?"), std::backtrace::Backtrace::force_capture()));
    }));
    let cfg = dir().join("config.json");
    if !cfg.exists() {
        fs::write(&cfg, CONFIG_SABLON).ok();
    }
    // If the port is taken the app is already running: bring the existing window to front instead of a second one.
    let server = Server::http(ADDR);
    if server.is_err() && AGENT.post(&format!("{ORIGIN}/api/show")).send_string("").is_ok() {
        return;
    }

    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::platform::windows::{IconExtWindows, WindowBuilderExtWindows};
    use wry::WebViewBuilderExtWindows;
    let event_loop = EventLoopBuilder::<Olay>::with_user_event().build();
    *kilit(&PROXY) = Some(event_loop.create_proxy());
    if let Ok(server) = server {
        thread::spawn(move || {
            for req in server.incoming_requests() {
                thread::spawn(move || {
                    // the panic hook writes details to hata.log; here it is enough that the thread dies
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handle(req))).ok();
                });
            }
        });
    }
    // Same ID as the Start menu shortcut: taskbar grouping and notifications under the RYOCS name
    unsafe { windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(tepsi::w(AUMID).as_ptr()) };
    let gizli = std::env::args().any(|a| a == "--gizli"); // at Windows startup: start in the tray only
    tepsi::baslat();
    thread::spawn(pano_izle);
    thread::spawn(|| {
        aumid_kaydet();
        thread::sleep(Duration::from_secs(3));
        olay(Olay::Goster(false)); // the window must not stay hidden even if the page could not say "ready"
    });
    let window = tao::window::WindowBuilder::new()
        .with_title("RYOCS")
        .with_visible(false) // shown once the UI is ready: no white flash and no theme jump
        .with_maximized(true) // fits the work area on every screen size
        .with_window_icon(tao::window::Icon::from_resource(1, None).ok())
        .with_taskbar_icon(tao::window::Icon::from_resource(1, None).ok())
        .with_inner_size(tao::dpi::LogicalSize::new(1180.0, 720.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(1060.0, 680.0)) // cards do not fit in anything smaller
        .with_decorations(false) // the page draws the title bar (neumorphic / glass)
        .with_undecorated_shadow(true)
        .build(&event_loop)
        .expect("pencere");
    unsafe {
        // Windows 11: rounded corners for the borderless window too (no effect on Windows 10)
        use tao::platform::windows::WindowExtWindows;
        use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
        DwmSetWindowAttribute(window.hwnd() as _, DWMWA_WINDOW_CORNER_PREFERENCE as u32, &DWMWCP_ROUND as *const i32 as _, 4);
    }
    let webview = wry::WebViewBuilder::new()
        .with_url(ORIGIN)
        .with_background_color((233, 234, 236, 255))
        .with_browser_accelerator_keys(false) // browser shortcuts such as F5, Ctrl+P, Ctrl+F do not belong to the app
        .with_hotkeys_zoom(false)
        .with_ipc_handler(|r| match r.body().as_str() {
            "hazir" => olay(Olay::Goster(false)),
            "koyu" => olay(Olay::Koyu(true)),
            "acik" => olay(Olay::Koyu(false)),
            m if m.len() < 20 => olay(Olay::Pencere(m.into())),
            _ => {}
        })
        .build(&window)
        .expect("WebView2");
    let mut kapaniyor = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let goster = || {
            window.set_visible(true);
            window.set_minimized(false);
            window.set_focus();
        };
        match event {
            // Close button: hides to the tray; reminders and the focus timer keep running.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                hata_kaydet("info: close request → hidden to the tray");
                window.set_visible(false);
                webview.evaluate_script("kapandi()").ok();
            }
            // Real exit: flush the pending save, give the save request time, then exit.
            Event::UserEvent(Olay::Kapat) if !kapaniyor => {
                kapaniyor = true;
                webview.evaluate_script("kaydetSimdi()").ok();
                window.set_visible(false);
                thread::spawn(|| {
                    thread::sleep(Duration::from_millis(700));
                    olay(Olay::Cik);
                });
            }
            Event::UserEvent(Olay::Cik) => {
                tepsi::kaldir();
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(Olay::Goster(one)) if !kapaniyor && (one || (!gizli && !window.is_visible())) => goster(),
            Event::UserEvent(Olay::Degistir) if !kapaniyor => {
                if window.is_visible() && window.is_focused() && !window.is_minimized() {
                    hata_kaydet("info: hidden by the shortcut");
                    window.set_visible(false);
                } else {
                    goster();
                }
            }
            Event::UserEvent(Olay::Koyu(k)) => {
                window.set_theme(Some(if k { tao::window::Theme::Dark } else { tao::window::Theme::Light }));
            }
            Event::UserEvent(Olay::Pencere(k)) => {
                use tao::window::ResizeDirection as Y;
                match k.as_str() {
                    "surukle" => drop(window.drag_window()),
                    "kucult" => window.set_minimized(true),
                    "buyut" => window.set_maximized(!window.is_maximized()),
                    "kapat" => {
                        window.set_visible(false);
                        webview.evaluate_script("kapandi()").ok();
                    }
                    b if !window.is_maximized() => {
                        let y = match b.strip_prefix("boyut:") {
                            Some("n") => Y::North, Some("s") => Y::South, Some("e") => Y::East, Some("w") => Y::West,
                            Some("ne") => Y::NorthEast, Some("nw") => Y::NorthWest, Some("se") => Y::SouthEast, Some("sw") => Y::SouthWest,
                            _ => return,
                        };
                        window.drag_resize_window(y).ok();
                    }
                    _ => {}
                }
            }
            // the maximize icon and edge cursors follow the window state
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                webview.evaluate_script(&format!("window.pencereDurum&&pencereDurum({})", window.is_maximized())).ok();
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sistem_gercek_degerler() {
        let v = sistem();
        assert!(v["bellek"]["toplam"].as_u64().unwrap() > v["bellek"]["bos"].as_u64().unwrap());
        assert!(v["disk"]["toplam"].as_u64().unwrap() >= v["disk"]["bos"].as_u64().unwrap());
        thread::sleep(Duration::from_millis(120));
        let c = sistem()["cpu"].as_f64().unwrap(); // second reading: a percent comes
        assert!((0.0..=100.0).contains(&c));
        let _ = pano_oku(); // must return without crashing (whatever the content)
    }

    #[test]
    fn esitleme_uctan_uca() {
        let kok = std::env::temp_dir().join(format!("ryocs-test-{}", now()));
        let (od, yerel) = (kok.join("od"), kok.join("RYOCS").join("data.json"));
        fs::create_dir_all(&od).unwrap();
        unsafe { std::env::set_var("APPDATA", &kok) }; // dir() reads from here; the other tests do not touch the file
        let uzak = od.join("RYOCS").join("data.json");
        let d = |b: &str| esitle(b).unwrap()["durum"].as_str().unwrap().to_string();
        write_json("data.json", &json!({"a": 1})).unwrap();
        assert_eq!(d("{}"), "kapali");
        assert!(esitle(&json!({"acik": true, "klasor": kok.join("yok")}).to_string()).is_err());
        assert_eq!(d(&json!({"acik": true, "klasor": od}).to_string()), "ok"); // first start: no remote, send
        assert_eq!(fs::read(&uzak).unwrap(), fs::read(&yerel).unwrap());
        fs::write(&uzak, r#"{"a":2}"#).unwrap(); // another computer wrote, local unchanged
        assert_eq!(d("{}"), "uzak_yeni"); // not pulled without asking
        assert_eq!(d(r#"{"baslangic":true}"#), "guncellendi");
        assert_eq!(veri_oku().unwrap()["a"], 2);
        fs::write(&uzak, r#"{"a": 22}"#).unwrap(); // after a pull the remote changed again with local unchanged: not a conflict
        assert_eq!(d(r#"{"baslangic":true}"#), "guncellendi");
        assert_eq!(veri_oku().unwrap()["a"], 22);
        assert!(dir().join("yedekler").join("data-geri-alma.json").exists()); // eski yerel yedekte
        write_json("data.json", &json!({"a": 3})).unwrap();
        fs::write(&uzak, r#"{"a":4}"#).unwrap(); // both sides changed
        assert_eq!(d(r#"{"baslangic":true}"#), "catisma");
        esitle_gonder(); // does not overwrite on conflict
        assert_eq!(fs::read_to_string(&uzak).unwrap(), r#"{"a":4}"#);
        assert_eq!(d(r#"{"sec":"yerel"}"#), "ok");
        assert_eq!(fs::read(&uzak).unwrap(), fs::read(&yerel).unwrap());
        assert_eq!(fs::read_to_string(dir().join("yedekler").join("data-onedrive.json")).unwrap(), r#"{"a":4}"#);
        fs::write(&uzak, "bozuk").unwrap(); // remote file corrupt: local is kept, an error returns
        assert!(esitle(r#"{"baslangic":true}"#).is_err());
        assert_eq!(veri_oku().unwrap()["a"], 3);
        fs::remove_dir_all(&kok).ok();
    }

    #[test]
    fn cpu_iki_olcum_farkindan() {
        assert_eq!(cpu_yuzde(None, (10, 100)), None); // ilk okuma
        assert_eq!(cpu_yuzde(Some((10, 100)), (10, 100)), None); // time has not passed
        assert_eq!(cpu_yuzde(Some((0, 0)), (75, 100)), Some(25.0));
        assert_eq!(cpu_yuzde(Some((50, 100)), (40, 200)), None); // the counter went backwards
    }

    #[test]
    fn pano_son_yirmi_ve_tekrarsiz() {
        let mut l = Vec::new();
        for i in 0..25 { pano_ekle(&mut l, format!("m{i}")); }
        assert_eq!((l.len(), l[0].as_str(), l[19].as_str()), (20, "m24", "m5"));
        pano_ekle(&mut l, "m10".into()); // a re-copied item moves to the top, no duplicates
        assert_eq!((l.len(), l[0].as_str()), (20, "m10"));
        pano_ekle(&mut l, "   ".into());
        pano_ekle(&mut l, "x".repeat(5001));
        assert_eq!(l[0], "m10");
    }

    #[test]
    fn esitleme_karari() {
        assert_eq!(esitle_karar(Some("a"), None, None), "ok"); // no file remotely: send
        assert_eq!(esitle_karar(Some("a"), Some("a"), None), "ok"); // same content
        assert_eq!(esitle_karar(Some("b"), Some("a"), Some("a")), "ok"); // only local changed
        assert_eq!(esitle_karar(Some("a"), Some("b"), Some("a")), "uzak_yeni"); // only remote changed
        assert_eq!(esitle_karar(None, Some("b"), None), "uzak_yeni"); // new computer, no local data
        assert_eq!(esitle_karar(Some("c"), Some("b"), Some("a")), "catisma"); // both changed
        assert_eq!(esitle_karar(Some("a"), Some("b"), None), "catisma"); // first start, two different data sets
    }

    #[test]
    fn haber_kategorisi_izin_listesinden() {
        assert_eq!(haber_kat(r#"{"kat":"spor"}"#), "spor");
        assert_eq!(haber_kat(r#"{"kat":"sondakika"}"#), "sondakika");
        assert_eq!(haber_kat(r#"{"kat":"../api/x"}"#), "gundem");
        assert_eq!(haber_kat(r#"{"kat":"en_tech"}"#), "en_tech");
        assert_eq!(haber_kat("bozuk"), "gundem");
    }

    #[test]
    fn yalnizca_kendi_kaynagimiz() {
        assert!(allowed(Some(ADDR), None));
        assert!(allowed(Some(ADDR), Some(ORIGIN)));
        assert!(!allowed(Some(ADDR), Some("https://kotu.example")));
        assert!(!allowed(Some("kotu.example:8765"), None));
        assert!(!allowed(None, None));
    }

    #[test]
    fn bos_sehir_reddedilir() {
        // validation happens before touching the file and the network
        assert!(config_set(r#"{"sehir":"  "}"#).is_err());
        assert!(config_set("{}").is_err());
    }

    #[test]
    fn yedek_adlari() {
        assert!(yedek_adi("data-2026-11-14.json"));
        assert!(yedek_adi("data-geri-alma.json"));
        assert!(!yedek_adi("../data.json"));
        assert!(!yedek_adi("data-2026-11-1x.json"));
        assert!(!yedek_adi("data-2026/11/14.json"));
        assert!(etkinlik_id("abc_123"));
        assert!(!etkinlik_id("a/../b"));
        assert!(!etkinlik_id(""));
    }

    #[test]
    fn pkce_rfc7636() {
        // RFC 7636 Appendix B example
        assert_eq!(pkce("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        let (a, b) = (rastgele(), rastgele());
        assert_eq!(a.len(), 43);
        assert_ne!(a, b);
    }

    #[test]
    fn kimlik_bicimi() {
        assert!(kimlik_gecerli("google", "123-abc.apps.googleusercontent.com", "GOCSPX-x").is_ok());
        assert!(kimlik_gecerli("google", "GOCSPX-yanlis-alan", "x").is_err());
        assert!(kimlik_gecerli("spotify", "0123456789abcdef0123456789ABCDEF", "").is_ok());
        assert!(kimlik_gecerli("spotify", "0123", "").is_err());
        assert!(kimlik_gecerli("google", "1.apps.googleusercontent.com", "bosluk var").is_err());
    }

    #[test]
    fn takvim_adresi() {
        assert_eq!(
            takvim_url(TR_TATIL, &["events"]).unwrap().as_str(),
            "https://www.googleapis.com/calendar/v3/calendars/tr.turkish%23holiday@group.v.calendar.google.com/events"
        );
        assert!(takvim_id("ad.soyad@gmail.com") && takvim_id(TR_TATIL) && takvim_id("primary"));
        assert!(!takvim_id("../users/me") && !takvim_id("a?b") && !takvim_id(""));
    }

    #[test]
    fn bozuk_belirtec_panik_yapmaz() {
        save_token("deneme_yok", json!("text only")); // not an object: silently ignored
    }

    #[test]
    fn tarih() {
        assert_eq!(ymd(0), "1970-01-01");
        assert_eq!(ymd(11017), "2000-03-01");
        assert_eq!(ymd(19782), "2024-02-29");
    }

    #[test]
    fn baslatici_yalnizca_uygulama_acar() {
        assert!(uygulama_gecerli("steam://rungameid/1091500").is_ok());
        assert!(uygulama_gecerli("https://ornek.com").is_err());
        assert!(uygulama_gecerli("C:/Windows/System32/cmd.bat").is_err());
        assert!(uygulama_gecerli("notepad.exe").is_err()); // relative path
        assert!(uygulama_gecerli("C:/Windows/notepad.exe").is_ok());
        assert!(uygulama_gecerli("C:/yok/yok.exe").is_err());
    }

    #[test]
    fn yer_bicimi_ve_dogrulama() {
        let y = yer(&json!({"name": "Yeşilova", "admin1": "Burdur", "admin2": "Yeşilova İlçesi", "country": "Türkiye", "latitude": 37.5, "longitude": 29.75}));
        assert_eq!((y["ad"].as_str(), y["bolge"].as_str()), (Some("Yeşilova"), Some("Burdur")));
        let y = yer(&json!({"name": "Yeşilova", "admin1": "Adana", "admin2": "Tufanbeyli İlçesi", "latitude": 38.2, "longitude": 36.2}));
        assert_eq!(y["bolge"], "Tufanbeyli, Adana");
        assert!(yer_dogrula(&json!({"ad": "X", "lat": 91.0, "lon": 0.0})).is_err());
        assert!(yer_dogrula(&json!({"ad": " ", "lat": 1.0, "lon": 1.0})).is_err());
        assert!(yer_dogrula(&json!({"ad": "X", "lat": "1", "lon": 1.0})).is_err());
        assert_eq!(yer_dogrula(&json!({"ad": " X ", "bolge": "B", "lat": 1.5, "lon": -2.0, "fazla": 1})).unwrap(), json!({"ad": "X", "bolge": "B", "lat": 1.5, "lon": -2.0}));
        assert!(sehir_ara("  ").is_err());
    }

    #[test]
    fn surum_karsilastirma() {
        assert_eq!(surum_sayi("v0.3.10"), Some((0, 3, 10)));
        assert!(surum_sayi("v0.3.10") > surum_sayi("0.3.9"));
        assert!(surum_sayi("v1.0.0") > surum_sayi("0.9.9"));
        assert_eq!(surum_sayi("v1.2"), None);
        assert_eq!(surum_sayi("v1.2.3.4"), None);
        assert_eq!(surum_sayi("v1.2.0-beta"), None);
    }

    #[test]
    fn yalnizca_web_adresi_acilir() {
        assert!(open_url("file:///C:/Windows/notepad.exe").is_err());
        assert!(open_url("calc").is_err());
    }
}
