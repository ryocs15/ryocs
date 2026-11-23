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

/// Pending OAuth requests per provider: (state, PKCE verifier). Google and Spotify can be connected at the same time.
static OAUTH_STATE: LazyLock<Mutex<HashMap<&'static str, (String, String)>>> = LazyLock::new(Default::default);
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
            scope: "https://www.googleapis.com/auth/calendar.events https://www.googleapis.com/auth/calendar.calendarlist.readonly https://www.googleapis.com/auth/gmail.modify",
            // to get a refresh_token
            extra: &[("access_type", "offline"), ("prompt", "consent")],
        }),
        // an extra Gmail account: mail only (no calendar), with the main Google client; Google asks which account
        "gmail" => Ok(Provider {
            name: "gmail",
            auth: "https://accounts.google.com/o/oauth2/v2/auth",
            token: "https://oauth2.googleapis.com/token",
            scope: "https://www.googleapis.com/auth/gmail.modify",
            extra: &[("access_type", "offline"), ("prompt", "select_account consent")],
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
/// Cached (0: not read yet, 1: Turkish, 2: English); reset whenever data.json is written, because the setting may have changed.
static DIL: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
fn dil_tr() -> bool {
    use std::sync::atomic::Ordering::Relaxed;
    match DIL.load(Relaxed) {
        1 => return true,
        2 => return false,
        _ => {}
    }
    let tr = match read_json("data.json")["ayar"]["dil"].as_str() {
        Some("tr") => true,
        Some("en") => false,
        _ => (unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff) == 0x1f,
    };
    DIL.store(if tr { 1 } else { 2 }, Relaxed);
    tr
}

/// Windows "Country or region" as two letters (e.g. "DE"); empty if it is not a country code (e.g. "001" world).
/// The UI uses it for defaults that depend on the country, not the language: base currency, °C/°F, holidays, clock.
fn bolge() -> String {
    let mut b = [0u16; 16];
    let n = unsafe { windows_sys::Win32::Globalization::GetUserDefaultGeoName(b.as_mut_ptr(), b.len() as i32) };
    bolge_kodu(&String::from_utf16_lossy(&b[..(n.max(1) - 1) as usize]))
}
/// First weather city for a new install: the capital (or largest city) of the Windows country, not always Istanbul.
/// shortcut: countries not listed get London; the user picks the real city in Settings anyway.
fn varsayilan_sehir() -> &'static str {
    match bolge().as_str() {
        "TR" => "Istanbul", "US" => "New York", "GB" => "London", "IE" => "Dublin", "CA" => "Toronto", "AU" => "Sydney", "NZ" => "Auckland",
        "DE" => "Berlin", "AT" => "Vienna", "CH" => "Zurich", "FR" => "Paris", "BE" => "Brussels", "NL" => "Amsterdam", "LU" => "Luxembourg",
        "ES" => "Madrid", "PT" => "Lisbon", "IT" => "Rome", "GR" => "Athens", "CY" => "Nicosia", "MT" => "Valletta",
        "PL" => "Warsaw", "CZ" => "Prague", "SK" => "Bratislava", "HU" => "Budapest", "RO" => "Bucharest", "BG" => "Sofia",
        "RS" => "Belgrade", "HR" => "Zagreb", "SI" => "Ljubljana", "UA" => "Kyiv", "RU" => "Moscow", "AZ" => "Baku", "GE" => "Tbilisi",
        "SE" => "Stockholm", "NO" => "Oslo", "DK" => "Copenhagen", "FI" => "Helsinki", "EE" => "Tallinn", "LV" => "Riga", "LT" => "Vilnius",
        "KZ" => "Almaty", "IL" => "Tel Aviv", "SA" => "Riyadh", "AE" => "Dubai", "QA" => "Doha", "KW" => "Kuwait City", "EG" => "Cairo",
        "ZA" => "Johannesburg", "NG" => "Lagos", "IN" => "New Delhi", "PK" => "Karachi", "CN" => "Beijing", "JP" => "Tokyo", "KR" => "Seoul",
        "SG" => "Singapore", "HK" => "Hong Kong", "ID" => "Jakarta", "TH" => "Bangkok", "MY" => "Kuala Lumpur", "PH" => "Manila", "VN" => "Hanoi",
        "BR" => "Sao Paulo", "MX" => "Mexico City", "AR" => "Buenos Aires", "CL" => "Santiago", "CO" => "Bogota",
        _ => "London",
    }
}
fn bolge_kodu(s: &str) -> String {
    if s.len() == 2 && s.bytes().all(|c| c.is_ascii_uppercase()) { s.into() } else { String::new() }
}

/// Picks the Turkish or English text.
fn l(tr: &'static str, en: &'static str) -> &'static str {
    if dil_tr() { tr } else { en }
}

fn read_json(name: &str) -> Value {
    fs::read_to_string(dir().join(name))
        .ok()
        .and_then(|s| dpapi_ac(&s))
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}

/// data.json: empty data if the file does not exist; an error if it exists but cannot be read (so the UI does not save empty data over the real data).
fn veri_oku() -> Result<Value, String> {
    veri_coz(fs::read_to_string(dir().join("data.json")))
}
/// Only "file not found" gives empty data. Any other read error (locked by antivirus or OneDrive, not UTF-8) is an error:
/// the UI then does not start with empty data that it would later save over the real file.
fn veri_coz(r: std::io::Result<String>) -> Result<Value, String> {
    match r {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(format!("{} ({e})", l("Veri dosyası (data.json) okunamadı. RYOCS'u yeniden başlatın.", "Could not read the data file (data.json). Restart RYOCS."))),
        Ok(s) => serde_json::from_str::<Value>(&dpapi_coz(&s)?).ok().filter(Value::is_object)
            .ok_or(l("Veri dosyası (data.json) bozuk. Bir yedeği geri yükleyin.", "The data file (data.json) is corrupt. Restore a backup.").into()),
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

/// Windows DPAPI: only this Windows user on this PC can decrypt (other programs reading the file cannot).
fn dpapi(veri: &[u8], sifrele: bool) -> Option<Vec<u8>> {
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB};
    let girdi = CRYPT_INTEGER_BLOB { cbData: veri.len() as u32, pbData: veri.as_ptr() as *mut u8 };
    let mut cikti = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    let n = std::ptr::null();
    let ok = unsafe {
        if sifrele {
            CryptProtectData(&girdi, n as _, n as _, n, n as _, CRYPTPROTECT_UI_FORBIDDEN, &mut cikti)
        } else {
            CryptUnprotectData(&girdi, std::ptr::null_mut(), n as _, n, n as _, CRYPTPROTECT_UI_FORBIDDEN, &mut cikti)
        }
    };
    if ok == 0 {
        return None;
    }
    let v = unsafe { std::slice::from_raw_parts(cikti.pbData, cikti.cbData as usize).to_vec() };
    unsafe { windows_sys::Win32::Foundation::LocalFree(cikti.pbData as _) };
    Some(v)
}

/// tokens.json is DPAPI-encrypted (base64); a plain JSON file from an older version is still read and encrypted on the next write.
fn tokens_oku() -> Value {
    read_json("tokens.json")
}
fn tokens_yaz(v: &Value) -> Result<(), String> {
    write_json("tokens.json", v)
}

/// Files with personal data are stored DPAPI-encrypted with this prefix: data.json, its backups, the calendar/notification cache.
/// Only this Windows user on this PC can open them; config.json keeps only its secrets encrypted (it stays editable by hand).
const DPAPI_ONEK: &str = "RYOCS-DPAPI1:";
fn sifreli_dosya(name: &str) -> bool {
    matches!(name, "data.json" | "onbellek.json" | "tokens.json" | "ekran.json" | "pano.json") || name.starts_with("yedekler")
}
fn dpapi_sar(s: &str) -> Result<String, String> {
    use base64::Engine;
    let b = dpapi(s.as_bytes(), true).ok_or("DPAPI")?;
    Ok(format!("{DPAPI_ONEK}{}", base64::engine::general_purpose::STANDARD.encode(b)))
}
/// Plain text of a stored file or value: DPAPI-encrypted (prefix), or plain as an older version or the user wrote it.
/// An encrypted file that does not open here is an error, never "empty": it belongs to another Windows account or PC.
fn dpapi_coz(s: &str) -> Result<String, String> {
    dpapi_ac(s).ok_or_else(|| l("Veri dosyası bu Windows hesabında açılamıyor (başka bir hesapta ya da bilgisayarda şifrelenmiş). Dışa aktarılmış bir yedek yükleyin.",
                 "The data file cannot be opened in this Windows account (it was encrypted in another account or on another PC). Load an exported backup.").into())
}
/// Same without a message: read_json uses it, and l() itself reads data.json through read_json (a message there would recurse).
fn dpapi_ac(s: &str) -> Option<String> {
    use base64::Engine;
    let s = s.trim().trim_start_matches('\u{feff}');
    let ac = |b: &str| base64::engine::general_purpose::STANDARD.decode(b).ok().and_then(|b| dpapi(&b, false)).and_then(|b| String::from_utf8(b).ok());
    match s.strip_prefix(DPAPI_ONEK) {
        Some(b) => ac(b),
        None if s.starts_with('{') || s.is_empty() => Some(s.to_string()),
        // tokens.json up to 1.0.7: DPAPI base64 without the prefix
        None => ac(s).or_else(|| Some(s.to_string())),
    }
}

fn write_json(name: &str, v: &Value) -> Result<(), String> {
    write_text(name, &v.to_string())
}

fn write_text(name: &str, s: &str) -> Result<(), String> {
    let veri = if sifreli_dosya(name) { dpapi_sar(s)? } else { s.to_string() };
    if name == "data.json" {
        DIL.store(0, std::sync::atomic::Ordering::Relaxed);
    }
    dayanikli_yaz(&dir().join(name), veri.as_bytes())
}
/// Temporary file, flushed to disk, then rename: neither a half-finished write nor a power cut leaves a broken or empty file.
/// Every write of RYOCS files goes through here (also the OneDrive copy), under one lock: two writers never share a temporary file.
fn dayanikli_yaz(p: &std::path::Path, b: &[u8]) -> Result<(), String> {
    use std::io::Write;
    static YAZ: Mutex<()> = Mutex::new(());
    let _y = kilit(&YAZ);
    let tmp = p.with_extension("tmp");
    let yaz = || -> std::io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(b)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, p)
    };
    yaz().map_err(|e| e.to_string())
}

/// OneDrive copy with a password: AES-256-GCM, key from PBKDF2-HMAC-SHA256, both from Windows CNG (no extra crate).
/// File: prefix + base64(salt 16 | nonce 12 | tag 16 | ciphertext). Opens on any PC that knows the password.
const PAROLA_ONEK: &str = "RYOCS-PAROLA1:";
fn aes_gcm(anahtar: &[u8; 32], nonce: &mut [u8; 12], etiket: &mut [u8; 16], girdi: &[u8], sifrele: bool) -> Option<Vec<u8>> {
    use windows_sys::Win32::Security::Cryptography::*;
    let mut h = std::ptr::null_mut();
    unsafe {
        if BCryptGenerateSymmetricKey(BCRYPT_AES_GCM_ALG_HANDLE, &mut h, std::ptr::null_mut(), 0, anahtar.as_ptr(), 32, 0) != 0 {
            return None;
        }
        let bilgi = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            cbSize: size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
            dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
            pbNonce: nonce.as_mut_ptr(),
            cbNonce: 12,
            pbTag: etiket.as_mut_ptr(),
            cbTag: 16,
            ..Default::default()
        };
        let (mut cikti, mut n) = (vec![0u8; girdi.len()], 0u32);
        let bp = &bilgi as *const _ as *const core::ffi::c_void;
        let r = if sifrele {
            BCryptEncrypt(h, girdi.as_ptr(), girdi.len() as u32, bp, std::ptr::null_mut(), 0, cikti.as_mut_ptr(), cikti.len() as u32, &mut n, 0)
        } else {
            BCryptDecrypt(h, girdi.as_ptr(), girdi.len() as u32, bp, std::ptr::null_mut(), 0, cikti.as_mut_ptr(), cikti.len() as u32, &mut n, 0)
        };
        BCryptDestroyKey(h);
        (r == 0).then_some(cikti)
    }
}
fn parola_anahtar(parola: &str, tuz: &[u8]) -> Option<[u8; 32]> {
    use windows_sys::Win32::Security::Cryptography::{BCryptDeriveKeyPBKDF2, BCRYPT_HMAC_SHA256_ALG_HANDLE};
    let mut k = [0u8; 32];
    let r = unsafe { BCryptDeriveKeyPBKDF2(BCRYPT_HMAC_SHA256_ALG_HANDLE, parola.as_ptr(), parola.len() as u32, tuz.as_ptr(), tuz.len() as u32, 310_000, k.as_mut_ptr(), 32, 0) };
    (r == 0).then_some(k)
}
/// PBKDF2 takes a noticeable fraction of a second: the key is derived once per password and salt and kept for the session.
/// Encrypting reuses that salt (the nonce is new every time, which is what AES-GCM needs).
type ParolaAnahtari = (String, [u8; 16], [u8; 32]); // password, salt, key
static PAROLA_ANAHTARI: Mutex<Option<ParolaAnahtari>> = Mutex::new(None);
fn parola_anahtari(parola: &str, tuz: Option<&[u8]>) -> Option<([u8; 16], [u8; 32])> {
    let mut o = kilit(&PAROLA_ANAHTARI);
    if let Some((p, t, k)) = o.as_ref() {
        if p == parola && tuz.is_none_or(|x| x == t) { return Some((*t, *k)); }
    }
    let mut t = [0u8; 16];
    match tuz { Some(x) => t.copy_from_slice(x.get(..16)?), None => getrandom::getrandom(&mut t).ok()? }
    let k = parola_anahtar(parola, &t)?;
    *o = Some((parola.to_string(), t, k));
    Some((t, k))
}
fn parola_sifrele(parola: &str, metin: &str) -> Option<String> {
    use base64::Engine;
    let (mut nonce, mut etiket) = ([0u8; 12], [0u8; 16]);
    getrandom::getrandom(&mut nonce).ok()?;
    let (tuz, anahtar) = parola_anahtari(parola, None)?;
    let ct = aes_gcm(&anahtar, &mut nonce.clone(), &mut etiket, metin.as_bytes(), true)?;
    Some(format!("{PAROLA_ONEK}{}", base64::engine::general_purpose::STANDARD.encode([&tuz[..], &nonce, &etiket, &ct].concat())))
}
/// None: not this format, or a wrong password / damaged file (GCM checks the tag).
fn parola_coz(parola: &str, s: &str) -> Option<String> {
    use base64::Engine;
    let b = base64::engine::general_purpose::STANDARD.decode(s.trim().strip_prefix(PAROLA_ONEK)?).ok()?;
    if b.len() < 44 { return None; }
    let (mut nonce, mut etiket) = (<[u8; 12]>::try_from(&b[16..28]).ok()?, <[u8; 16]>::try_from(&b[28..44]).ok()?);
    String::from_utf8(aes_gcm(&parola_anahtari(parola, Some(&b[..16]))?.1, &mut nonce, &mut etiket, &b[44..], false)?).ok()
}

/// Encrypts files an older version left plain: data.json, backups, the secrets in config.json. Runs once per start, cheap when done.
fn sifrele_goc() {
    let eski = |name: &str| fs::read_to_string(dir().join(name)).ok().filter(|s| !s.trim_start_matches('\u{feff}').trim_start().starts_with(DPAPI_ONEK));
    let yedekler = yedek_listesi();
    let adlar = ["data.json", "tokens.json"].map(String::from).into_iter().chain(yedekler.as_array().into_iter().flatten().filter_map(|v| v.as_str().map(|s| format!("yedekler/{s}"))));
    for ad in adlar {
        // only valid JSON is rewritten: a damaged file stays as it is for the user to inspect
        if let Some(s) = eski(&ad).and_then(|s| dpapi_ac(&s)).filter(|s| serde_json::from_str::<Value>(s).is_ok()) {
            if let Err(e) = write_text(&ad, &s) { hata_kaydet(&format!("encrypt {ad}: {e}")); }
        }
    }
    if let Ok(mut c) = config_oku() {
        let mut degisti = false;
        for p in ["google", "spotify"] {
            if let Some(g) = c[p]["client_secret"].as_str().filter(|g| !g.trim().is_empty() && !g.starts_with(DPAPI_ONEK)) {
                if let Ok(s) = dpapi_sar(g.trim()) { c[p]["client_secret"] = json!(s); degisti = true; }
            }
        }
        if degisti { config_yaz(&c).ok(); }
    }
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
        konum(sehir_adi(cfg["sehir"].as_str().unwrap_or(varsayilan_sehir()))?)?
    };
    let f = q.get("birim").is_some_and(|b| b == "f"); // °F, mph, inch; anything else is metric
    let (lat, lon) = (loc["lat"].to_string(), loc["lon"].to_string());
    // air quality (and pollen in Europe) comes from a second Open-Meteo API, in parallel; the card works without it
    let kalite = thread::spawn({ let (lat, lon) = (lat.clone(), lon.clone()); move || {
        body(AGENT.get("https://air-quality-api.open-meteo.com/v1/air-quality").query("latitude", &lat).query("longitude", &lon)
            .query("current", "european_aqi,pm2_5,alder_pollen,birch_pollen,grass_pollen,mugwort_pollen,olive_pollen,ragweed_pollen")
            .query("timezone", "auto").call()).map(|v| v["current"].clone()).unwrap_or(Value::Null)
    }});
    let mut w = body(
        AGENT.get("https://api.open-meteo.com/v1/forecast")
            .query("latitude", &lat)
            .query("longitude", &lon)
            .query("current", "temperature_2m,apparent_temperature,weather_code,wind_speed_10m,wind_direction_10m,relative_humidity_2m,pressure_msl,precipitation,is_day")
            .query("hourly", "temperature_2m,precipitation_probability,weather_code,is_day")
            .query("daily", "weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,sunrise,sunset,uv_index_max,wind_gusts_10m_max")
            .query("temperature_unit", if f { "fahrenheit" } else { "celsius" })
            .query("wind_speed_unit", if f { "mph" } else { "kmh" })
            .query("precipitation_unit", if f { "inch" } else { "mm" })
            .query("timezone", "auto").query("forecast_days", "7").query("past_days", "1").call(),
    )?;
    dun_ayir(&mut w);
    w["kalite"] = kalite.join().unwrap_or(Value::Null);
    w["birim"] = json!(if f { "f" } else { "c" });
    w["sehir"] = loc["ad"].clone();
    w["bolge"] = loc["bolge"].clone();
    Ok(w)
}

/// past_days=1 gives yesterday too: its temperature at this hour goes to "dun", then yesterday is cut off,
/// so for the UI index 0 is still today (daily) and the hourly list starts at today's midnight.
fn dun_ayir(w: &mut Value) {
    let su = w["current"]["time"].as_str().and_then(|t| t.get(..13)).map(String::from);
    let i = w["hourly"]["time"].as_array().and_then(|a| a.iter().position(|t| t.as_str().and_then(|t| t.get(..13)) == su.as_deref()));
    w["dun"] = i.filter(|i| *i >= 24).map(|i| w["hourly"]["temperature_2m"][i - 24].clone()).unwrap_or(Value::Null);
    for (k, n) in [("hourly", 24), ("daily", 1)] {
        for v in w[k].as_object_mut().into_iter().flat_map(|o| o.values_mut()) {
            if let Some(a) = v.as_array_mut() {
                a.drain(..n.min(a.len()));
            }
        }
    }
}

/// Bundle news: the (undocumented) feed the bundle.app site itself uses; no credentials needed.
/// shortcut: not an official API; if Bundle changes it the card says "could not load"; switch to an official API/RSS if one appears.
fn haber_kat(b: &str) -> &'static str {
    // the category in the request comes only from the allow list: an outside value must not reach the URL path
    let k = serde_json::from_str::<Value>(b).ok().and_then(|v| v["kat"].as_str().map(String::from)).unwrap_or_default();
    ["sondakika", "gundem", "bilim", "finans", "yasam", "spor", "en_top", "en_world", "en_business", "en_tech", "en_science", "en_sport"]
        .into_iter().find(|x| *x == k).unwrap_or("gundem")
}

/// Breaking news: several outlets' RSS feeds, merged in the UI. Addresses only from here, never from the request.
/// shortcut: undocumented feeds; one that changes drops out quietly while the others still come (all failing shows the error).
const SON_DAKIKA: [(&str, &str); 7] = [
    ("TRT Haber", "https://www.trthaber.com/sondakika.rss"),
    ("Sabah", "https://www.sabah.com.tr/rss/sondakika.xml"),
    ("Sözcü", "https://www.sozcu.com.tr/feeds-son-dakika"),
    ("Milliyet", "https://www.milliyet.com.tr/rss/rssnew/sondakikarss.xml"),
    ("CNN Türk", "https://www.cnnturk.com/feed/rss/all/news"),
    ("Habertürk", "https://www.haberturk.com/rss"),
    ("En Son Haber", "https://www.ensonhaber.com/rss/ensonhaber.xml"),
];
/// [{k: outlet, x: raw RSS}] for the feeds that answered, fetched at the same time; Err only when none did.
fn son_dakika(getir: impl Fn(&str) -> Result<String, String> + Sync) -> Result<Value, String> {
    let sonuc: Vec<_> = thread::scope(|k| {
        let is: Vec<_> = SON_DAKIKA.iter().map(|(ad, u)| { let g = &getir; k.spawn(move || (*ad, g(u))) }).collect();
        is.into_iter().filter_map(|i| i.join().ok()).collect()
    });
    let hata = sonuc.iter().find_map(|(_, r)| r.as_ref().err().cloned());
    let l: Vec<Value> = sonuc.into_iter().filter_map(|(ad, r)| r.ok().map(|x| json!({ "k": ad, "x": x }))).collect();
    if l.is_empty() { Err(hata.unwrap_or_else(|| AG_YOK.into())) } else { Ok(json!(l)) }
}

fn haber(b: &str) -> Result<Value, String> {
    // Bundle has no breaking news: several outlets' RSS feeds go to the UI as raw text and are parsed there with DOMParser.
    // English tabs: BBC News RSS feeds, parsed in the UI the same way.
    let rss = match haber_kat(b) {
        "sondakika" => return son_dakika(metin),
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
    }))
}

/// SHA-256 (lowercase hex) of file `ad` in a SHA256SUMS text ("<64 hex>  <name>" per line); None if missing or malformed.
fn sha_satiri(metin: &str, ad: &str) -> Option<String> {
    metin.lines().find_map(|s| {
        let (h, a) = s.trim().split_once(char::is_whitespace)?;
        (a.trim().trim_start_matches('*') == ad && h.len() == 64 && h.bytes().all(|c| c.is_ascii_hexdigit())).then(|| h.to_ascii_lowercase())
    })
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
/// One-click update: the installer of the latest release is downloaded (GitHub over HTTPS, addresses built here, not taken from the
/// response), checked against the release's SHA256SUMS, started with /SILENT (no UAC: it installs per user) and RYOCS quits.
/// The installer starts RYOCS again when it is done. The UI saves before calling this.
fn guncelle_kur() -> Result<Value, String> {
    use std::io::Read;
    let r = body(AGENT.get("https://api.github.com/repos/ryocs15/ryocs/releases/latest").set("Accept", "application/vnd.github+json").call())?;
    let son = r["tag_name"].as_str().and_then(surum_sayi).ok_or(l("Sürüm bilgisi okunamadı", "Could not read the version info"))?;
    if Some(son) <= surum_sayi(env!("CARGO_PKG_VERSION")) {
        return Err(l("RYOCS zaten güncel", "RYOCS is already up to date").into());
    }
    let s = format!("{}.{}.{}", son.0, son.1, son.2);
    let (ad, taban) = (format!("RYOCS-Setup-{s}.exe"), format!("https://github.com/ryocs15/ryocs/releases/download/v{s}"));
    // the installer is several MB: a per-read timeout instead of the 15 s total of AGENT (slow networks)
    let indir = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(native_tls::TlsConnector::new().map_err(|e| e.to_string())?))
        .timeout_connect(Duration::from_secs(15)).timeout_read(Duration::from_secs(60)).build();
    let ag = |e: ureq::Error| match e { ureq::Error::Status(c, _) => format!("HTTP {c}"), _ => AG_YOK.to_string() };
    let toplam = indir.get(&format!("{taban}/SHA256SUMS")).call().map_err(|e| match e {
        ureq::Error::Status(404, _) => l("Bu sürüm tek tıkla kurulamıyor (doğrulama dosyası yok)", "This version cannot be installed in one click (no checksum file)").to_string(),
        e => ag(e),
    })?.into_string().map_err(|e| e.to_string())?;
    let beklenen = sha_satiri(&toplam, &ad).ok_or(l("Doğrulama dosyasında kurulum bulunamadı", "The installer is missing from the checksum file"))?;
    let mut b = Vec::new();
    indir.get(&format!("{taban}/{ad}")).call().map_err(ag)?.into_reader().take(300 << 20).read_to_end(&mut b).map_err(|_| AG_YOK.to_string())?;
    use sha2::{Digest, Sha256};
    if hex(&Sha256::digest(&b)) != beklenen {
        return Err(l("İndirilen dosya doğrulanamadı; kurulmadı", "The downloaded file did not pass the check; nothing was installed").into());
    }
    let yol = std::env::temp_dir().join(&ad);
    dayanikli_yaz(&yol, &b)?;
    std::process::Command::new(&yol).args(["/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]).spawn().map_err(|e| e.to_string())?;
    // answer the UI first, then save and quit (the installer closes a RYOCS that is still running anyway)
    thread::spawn(|| { thread::sleep(Duration::from_millis(300)); olay(Olay::Kapat); });
    Ok(json!(s))
}

/// English currency card: ECB daily reference rates (raw XML, parsed in the UI) + gold-api.com gold and silver ounce (USD).
/// ag / dunya: silver and world rates only when the card shows them (silver row on, a code the bank does not publish).
fn kur_ecb(ag: bool, dunya_da: bool) -> Result<Value, String> {
    kur_topla("ecb", "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml", ag, dunya_da)
}
/// The bank feed, gold, silver and world rates are fetched at the same time: in a row they took up to 4 × 15 s on a slow network.
fn kur_topla(ad: &str, banka: &str, ag: bool, dunya_da: bool) -> Result<Value, String> {
    thread::scope(|k| {
        let (o, a, d) = (k.spawn(|| maden("XAU")), k.spawn(move || ag.then(|| maden("XAG")).flatten()), k.spawn(move || dunya_da.then(dunya).flatten()));
        let b = metin(banka)?;
        Ok(json!({ ad: b, "ons": o.join().ok().flatten(), "ag": a.join().ok().flatten(), "dunya": d.join().ok().flatten() }))
    })
}

/// Currency and gold: TCMB daily rates XML (raw text, parsed by the UI) + gold-api.com gold and silver ounce (USD).
/// If gold cannot be fetched the rates still return; the UI computes gram gold as ounce × dollar ÷ 31.1035.
/// shortcut: gold-api.com is a documented but unguaranteed free service; if it closes the gold row shows "—".
fn kur(ag: bool, dunya_da: bool) -> Result<Value, String> {
    kur_topla("tcmb", "https://www.tcmb.gov.tr/kurlar/today.xml", ag, dunya_da)
}
/// World rates (units per 1 USD, about 160 currencies, daily) for codes the central bank feed lacks, e.g. PLN in TCMB.
/// shortcut: open.er-api.com is free without a key but asks for attribution (shown in the card) and may rate-limit; on failure those rows are skipped.
fn dunya() -> Option<Value> {
    body(AGENT.get("https://open.er-api.com/v6/latest/USD").call()).ok().map(|v| v["rates"].clone()).filter(Value::is_object)
}
/// Gold (XAU) or silver (XAG) ounce price in USD from gold-api.com; None if it cannot be fetched.
fn maden(kod: &str) -> Option<f64> {
    body(AGENT.get(&format!("https://api.gold-api.com/price/{kod}")).call()).ok().and_then(|v| v["price"].as_f64())
}

/// System card: CPU (difference between two readings), memory, system disk, battery.
/// The previous reading and when it was taken: after a long pause (window hidden) it is too old to give the current load.
static CPU_ONCEKI: Mutex<Option<((u64, u64), std::time::Instant)>> = Mutex::new(None);
/// CPU percent: from the difference of the (idle, total) counters between two readings. None on the first reading or with no difference.
fn cpu_yuzde(once: Option<(u64, u64)>, simdi: (u64, u64)) -> Option<f64> {
    let (b0, t0) = once?;
    let (db, dt) = (simdi.0.checked_sub(b0)?, simdi.1.checked_sub(t0)?);
    (dt > 0).then(|| (100.0 * (1.0 - db as f64 / dt as f64)).clamp(0.0, 100.0))
}
fn sistem() -> Value {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives};
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use windows_sys::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows_sys::Win32::System::Threading::GetSystemTimes;
    let ft = |f: FILETIME| (f.dwHighDateTime as u64) << 32 | f.dwLowDateTime as u64;
    let z = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut bos, mut cek, mut kul) = (z, z, z);
    // kernel time includes idle time: total = kernel + user
    let cpu = (unsafe { GetSystemTimes(&mut bos, &mut cek, &mut kul) } != 0).then(|| (ft(bos), ft(cek) + ft(kul)))
        .and_then(|simdi| {
            let once = kilit(&CPU_ONCEKI).replace((simdi, std::time::Instant::now()));
            // older than 10 s: the average of the whole pause, not the load now; the next reading (3 s later) shows it
            cpu_yuzde(once.filter(|(_, t)| t.elapsed() < Duration::from_secs(10)).map(|(o, _)| o), simdi)
        });
    let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    let bellek = (unsafe { GlobalMemoryStatusEx(&mut m) } != 0).then(|| json!({ "toplam": m.ullTotalPhys, "bos": m.ullAvailPhys }));
    // every fixed drive (A–Z bits of GetLogicalDrives; type 3 = DRIVE_FIXED), the system drive first; at most 4.
    // Read once a minute: many USB disks count as fixed, and asking every 3 s would keep a sleeping one spinning.
    static DISKLER: Mutex<Option<(std::time::Instant, Value)>> = Mutex::new(None);
    let onbellek = kilit(&DISKLER).as_ref().filter(|(t, _)| t.elapsed() < Duration::from_secs(60)).map(|(_, v)| v.clone());
    let diskler = onbellek.unwrap_or_else(|| {
    let sistem_surucu = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into()).to_ascii_uppercase();
    let maske = unsafe { GetLogicalDrives() };
    let mut diskler: Vec<Value> = (0..26u8).filter(|i| maske >> i & 1 == 1).filter_map(|i| {
        let ad = format!("{}:", (b'A' + i) as char);
        let kok: Vec<u16> = format!("{ad}\\").encode_utf16().chain([0]).collect();
        let (mut db, mut dt) = (0u64, 0u64);
        (unsafe { GetDriveTypeW(kok.as_ptr()) } == 3 && unsafe { GetDiskFreeSpaceExW(kok.as_ptr(), &mut db, &mut dt, std::ptr::null_mut()) } != 0 && dt > 0)
            .then(|| json!({ "ad": ad, "toplam": dt, "bos": db }))
    }).collect();
    diskler.sort_by_key(|d| d["ad"].as_str() != Some(sistem_surucu.as_str()));
    diskler.truncate(4);
    let v = json!(diskler);
    *kilit(&DISKLER) = Some((std::time::Instant::now(), v.clone()));
    v
    });
    let mut g: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    // BatteryFlag 128: no battery (desktop); percent 255: unknown
    let pil = (unsafe { GetSystemPowerStatus(&mut g) } != 0 && g.BatteryFlag & 128 == 0 && g.BatteryLifePercent <= 100)
        .then(|| json!({ "yuzde": g.BatteryLifePercent, "sarj": g.ACLineStatus == 1,
            "kalan": (g.BatteryLifeTime != u32::MAX && g.ACLineStatus != 1).then_some(g.BatteryLifeTime) })); // seconds; unknown while charging
    json!({ "cpu": cpu, "bellek": bellek, "diskler": diskler, "pil": pil, "acik": unsafe { GetTickCount64() } / 1000 })
}

/// Clipboard history: kept in memory only, only while the card is open (PANO_ACIK); never written to disk.
/// An item: the text, when it was copied (ms) and whether it is pinned. Pinned items stay on top and do not drop off the end.
struct PanoOge {
    m: String,
    t: u64,
    sabit: bool,
}
static PANO: Mutex<Vec<PanoOge>> = Mutex::new(Vec::new());
static PANO_ACIK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Raised on every change: the card polls every 2 s and gets the list only when this differs from what it has.
static PANO_SURUM: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn pano_ekle(l: &mut Vec<PanoOge>, m: String, t: u64) -> bool {
    if m.trim().is_empty() || m.chars().count() > 5000 {
        return false; // empty or very long text (file dumps etc.) is not kept
    }
    let sabit = l.iter().any(|x| x.m == m && x.sabit);
    l.retain(|x| x.m != m);
    // newest on top of its group: pinned ones first, then the others (at most 20 unpinned)
    let i = if sabit { 0 } else { l.iter().take_while(|x| x.sabit).count() };
    l.insert(i, PanoOge { m, t, sabit });
    let mut acik = 0;
    l.retain(|x| x.sabit || { acik += 1; acik <= 20 });
    true
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
            let t = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
            if pano_ekle(&mut kilit(&PANO), m, t) {
                PANO_SURUM.fetch_add(1, Relaxed);
            }
        }
    }
}
/// Pinned items survive a restart: pano.json (DPAPI) holds them, [{m, t}]. Unpinned copies stay in memory only.
fn pano_sabit_kaydet(l: &[PanoOge]) {
    let s: Vec<Value> = l.iter().filter(|x| x.sabit).map(|x| json!({ "m": x.m, "t": x.t })).collect();
    if let Err(e) = write_json("pano.json", &json!(s)) { hata_kaydet(&format!("pano.json: {e}")); }
}
fn pano_sabit_oku() -> Vec<PanoOge> {
    read_json("pano.json").as_array().into_iter().flatten()
        .filter_map(|x| Some(PanoOge { m: x["m"].as_str().filter(|m| !m.trim().is_empty())?.to_string(), t: x["t"].as_u64().unwrap_or(0), sabit: true }))
        .take(200).collect()
}
/// {acik, temizle: true (the unpinned ones; also when the card closes), sil: text, sabitle: {m, on}, surum}
fn pano(b: &str) -> Value {
    use std::sync::atomic::Ordering::Relaxed;
    let r: Value = serde_json::from_str(b).unwrap_or_default();
    if let Some(a) = r["acik"].as_bool() { PANO_ACIK.store(a, Relaxed); }
    let mut l = kilit(&PANO);
    let n = l.len();
    let sabit_say = |l: &Vec<PanoOge>| l.iter().filter(|x| x.sabit).count();
    let s0 = sabit_say(&l);
    if r["temizle"] == true { l.retain(|x| x.sabit); }
    if let Some(m) = r["sil"].as_str() { l.retain(|x| x.m != m); }
    let mut degisti = l.len() != n;
    let mut sabit_degisti = sabit_say(&l) != s0;
    if let (Some(m), Some(on)) = (r["sabitle"]["m"].as_str(), r["sabitle"]["on"].as_bool()) {
        if let Some(i) = l.iter().position(|x| x.m == m && x.sabit != on) {
            let mut o = l.remove(i);
            o.sabit = on;
            let j = if on { 0 } else { l.iter().take_while(|x| x.sabit).count() };
            l.insert(j, o);
            degisti = true;
            sabit_degisti = true;
        }
    }
    if sabit_degisti { pano_sabit_kaydet(&l); }
    if degisti { PANO_SURUM.fetch_add(1, Relaxed); }
    let surum = PANO_SURUM.load(Relaxed);
    let mut v = json!({ "acik": PANO_ACIK.load(Relaxed), "surum": surum });
    if r["surum"].as_u64() != Some(surum) {
        v["liste"] = l.iter().map(|x| json!({ "m": x.m, "t": x.t, "s": x.sabit })).collect();
    }
    v
}

/// Screen time: seconds per program per local day ({"g": {"2026-11-22": {"msedge": 3600}}}), the 30 newest days, in ekran.json (DPAPI).
/// Counted only while the card is open (EKRAN_ACIK, set by the UI like the clipboard); stays on this PC, not synced.
static EKRAN: Mutex<Option<Value>> = Mutex::new(None);
static EKRAN_ACIK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static EKRAN_KIRLI: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn ekran_ekle(v: &mut Value, gun: &str, exe: &str, sn: u64) {
    if !v.is_object() { *v = json!({}); }
    if !v["g"].is_object() { v["g"] = json!({}); }
    let g = v["g"].as_object_mut().expect("object");
    let d = g.entry(gun).or_insert_with(|| json!({}));
    if !d.is_object() { *d = json!({}); }
    d[exe] = json!(d[exe].as_u64().unwrap_or(0) + sn);
    while g.len() > 30 {
        let eski = g.keys().min().cloned().expect("not empty");
        g.remove(&eski);
    }
}
fn ekran_veri() -> std::sync::MutexGuard<'static, Option<Value>> {
    let mut e = kilit(&EKRAN);
    if e.is_none() { *e = Some(ekran_yukle()); }
    e
}
/// ekran.json, or empty when there is none. A file that does not open here (another Windows account, damaged) is kept aside
/// as ekran.bozuk.json, not overwritten by the next save.
fn ekran_yukle() -> Value {
    let p = dir().join("ekran.json");
    // only "not found" is empty; any other read error (locked by antivirus or OneDrive) also keeps the file aside
    let v = match fs::read_to_string(&p) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return json!({}),
        r => r.ok().and_then(|s| dpapi_ac(&s)).and_then(|s| serde_json::from_str::<Value>(&s).ok()).filter(Value::is_object),
    };
    if v.is_none() { fs::rename(&p, dir().join("ekran.bozuk.json")).ok(); }
    v.unwrap_or_else(|| json!({}))
}
fn ekran_kaydet() {
    use std::sync::atomic::Ordering::Relaxed;
    if !EKRAN_KIRLI.swap(false, Relaxed) { return; }
    let v = ekran_veri().clone().unwrap_or_default();
    if write_json("ekran.json", &v).is_err() { EKRAN_KIRLI.store(true, Relaxed); }
}
/// The program of the window in front, by its readable name (program_adi); None when there is none or the screen is locked.
fn on_program() -> Option<String> {
    use windows_sys::{core::BOOL, Win32::Foundation::{HWND, LPARAM}};
    use windows_sys::Win32::UI::WindowsAndMessaging::{EnumChildWindows, GetForegroundWindow, GetWindowThreadProcessId};
    // a Store app (Settings, Calculator) runs inside ApplicationFrameHost: its own process owns a child window
    unsafe extern "system" fn cocuk(h: HWND, l: LPARAM) -> BOOL {
        let v = unsafe { &mut *(l as *mut (u32, u32)) };
        let mut p = 0;
        unsafe { GetWindowThreadProcessId(h, &mut p) };
        if p != 0 && p != v.0 { v.1 = p; 0 } else { 1 }
    }
    let w = unsafe { GetForegroundWindow() };
    if w.is_null() { return None; }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(w, &mut pid) };
    if pid == std::process::id() { return Some("RYOCS".into()); }
    let mut yol = surec_yolu(pid)?;
    let kok = |y: &str| std::path::Path::new(y).file_stem().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    if kok(&yol) == "applicationframehost" {
        let mut v = (pid, 0u32);
        unsafe { EnumChildWindows(w, Some(cocuk), &mut v as *mut _ as LPARAM) };
        yol = surec_yolu(v.1).unwrap_or(yol);
    }
    (kok(&yol) != "lockapp").then(|| program_adi(&yol))
}
fn surec_yolu(pid: u32) -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
    if pid == 0 { return None; }
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() { return None; }
        let (mut b, mut n) = ([0u16; 1024], 1024u32);
        let ok = QueryFullProcessImageNameW(h, 0, b.as_mut_ptr(), &mut n) != 0;
        CloseHandle(h);
        ok.then(|| String::from_utf16_lossy(&b[..n as usize]))
    }
}
/// Readable name of an exe: its file description ("Microsoft Edge", "Windows Explorer"), else its product name, else the
/// file name. Cached per path. A helper is named after its program: "Steam Client WebHelper" → "Steam".
fn program_adi(yol: &str) -> String {
    static AD: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Default::default);
    if let Some(a) = kilit(&AD).get(yol) { return a.clone(); }
    let ad = ["FileDescription", "ProductName"].iter().filter_map(|a| surum_alani(yol, a)).map(|s| ad_temizle(&s)).find(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let k = std::path::Path::new(yol).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let mut c = k.chars();
            c.next().map(|i| i.to_uppercase().chain(c).collect()).unwrap_or(k)
        });
    kilit(&AD).insert(yol.into(), ad.clone());
    ad
}
fn ad_temizle(s: &str) -> String {
    let s = s.replace(['®', '™'], "").replace("(R)", "").replace("(TM)", "");
    let mut k: Vec<&str> = s.split_whitespace().collect();
    if k.len() > 1 && k.last().is_some_and(|x| x.to_ascii_lowercase().ends_with("helper")) {
        k.pop();
        if k.len() > 1 && k.last().is_some_and(|x| x.eq_ignore_ascii_case("client")) { k.pop(); }
    }
    let s = k.join(" ");
    // Windows files' product name and the runtime that Electron apps ship unchanged: not a program name
    if s.contains("Operating System") || s.eq_ignore_ascii_case("electron") { String::new() } else { s }
}
/// A string from an exe's version resource, in the file's first language.
fn surum_alani(yol: &str, alan: &str) -> Option<String> {
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    let w: Vec<u16> = yol.encode_utf16().chain([0]).collect();
    unsafe {
        let n = GetFileVersionInfoSizeW(w.as_ptr(), std::ptr::null_mut());
        if n == 0 { return None; }
        let mut b = vec![0u8; n as usize];
        if GetFileVersionInfoW(w.as_ptr(), 0, n, b.as_mut_ptr().cast()) == 0 { return None; }
        let sorgu = |s: &str| {
            let q: Vec<u16> = s.encode_utf16().chain([0]).collect();
            let (mut p, mut l) = (std::ptr::null_mut(), 0u32);
            (VerQueryValueW(b.as_ptr().cast(), q.as_ptr(), &mut p, &mut l) != 0 && l > 0 && !p.is_null()).then_some((p as *const u16, l))
        };
        let (dil, kod) = sorgu("\\VarFileInfo\\Translation").map(|(p, _)| (*p, *p.add(1))).unwrap_or((0x0409, 0x04B0));
        let (p, l) = sorgu(&format!("\\StringFileInfo\\{dil:04x}{kod:04x}\\{alan}"))?;
        // some exes (Spotify) give the length in bytes, not characters: stay inside the buffer and stop at the first NUL
        let kalan = (b.as_ptr().add(b.len()) as usize).saturating_sub(p as usize) / 2;
        let s = std::slice::from_raw_parts(p, (l as usize).min(kalan));
        let s = &s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())];
        Some(String::from_utf16_lossy(s).trim().to_string())
    }
}
/// Away from the PC: no input for 2 minutes, unless something runs full screen (a video, a game, a presentation);
/// also away while the screen is locked or the screen saver runs.
fn ekran_bosta() -> bool {
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows_sys::Win32::UI::Shell::SHQueryUserNotificationState;
    let mut q = 0;
    let durum = if unsafe { SHQueryUserNotificationState(&mut q) } == 0 { q } else { 0 };
    if durum == 1 { return true; } // QUNS_NOT_PRESENT: locked, screen saver, fast user switching
    if matches!(durum, 2..=4) { return false; } // busy / full-screen D3D / presentation mode
    let mut l = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe { GetLastInputInfo(&mut l) != 0 && GetTickCount().wrapping_sub(l.dwTime) > 120_000 }
}
fn ekran_izle() {
    use std::sync::atomic::Ordering::Relaxed;
    let mut son_yaz = std::time::Instant::now();
    loop {
        thread::sleep(Duration::from_secs(5));
        if EKRAN_ACIK.load(Relaxed) && !ekran_bosta() {
            if let Some(p) = on_program() {
                ekran_ekle(ekran_veri().get_or_insert_with(|| json!({})), &bugun(), &p, 5);
                EKRAN_KIRLI.store(true, Relaxed);
            }
        }
        if son_yaz.elapsed() >= Duration::from_secs(60) {
            ekran_kaydet();
            son_yaz = std::time::Instant::now();
        }
    }
}
/// {acik, sil: true} → {acik, bugun, gunler: {day: {program: seconds}}} (the 14 newest days)
fn ekran(b: &str) -> Value {
    use std::sync::atomic::Ordering::Relaxed;
    let r: Value = serde_json::from_str(b).unwrap_or_default();
    if let Some(a) = r["acik"].as_bool() { EKRAN_ACIK.store(a, Relaxed); }
    let mut e = ekran_veri();
    let v = e.get_or_insert_with(|| json!({}));
    if r["sil"] == true {
        *v = json!({});
        EKRAN_KIRLI.store(true, Relaxed);
    }
    let mut gunler = serde_json::Map::new();
    if let Some(g) = v["g"].as_object() {
        let mut k: Vec<&String> = g.keys().collect();
        k.sort();
        for d in k.into_iter().rev().take(14) { gunler.insert(d.clone(), g[d].clone()); } // 14: the weekly summary compares with the week before
    }
    drop(e);
    if r["sil"] == true { ekran_kaydet(); } // deleted data leaves the disk now, not a minute later
    json!({ "acik": EKRAN_ACIK.load(Relaxed), "bugun": bugun(), "gunler": gunler })
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
/// Plain JSON text of local data.json (decrypted); None if missing or not openable.
fn yerel_metin() -> Option<String> {
    fs::read_to_string(dir().join("data.json")).ok().and_then(|s| dpapi_coz(&s).ok())
}
/// Password of the OneDrive copy on this PC (kept DPAPI-encrypted in esitle.json).
fn onedrive_parola() -> Option<String> {
    read_json("esitle.json")["parola"].as_str().and_then(dpapi_ac).filter(|p| !p.is_empty())
}
/// Plain text of the OneDrive copy: None if there is none; Err if it is password-encrypted and the password here is missing or wrong.
fn uzak_metin(k: &std::path::Path) -> Option<Result<String, ()>> {
    let b = fs::read(k.join("RYOCS").join("data.json")).ok()?;
    let s = String::from_utf8_lossy(b.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&b)).into_owned();
    Some(if s.trim_start().starts_with(PAROLA_ONEK) { onedrive_parola().and_then(|p| parola_coz(&p, &s)).ok_or(()) } else { Ok(s) })
}
/// Hashes are over the plain JSON: encrypted bytes differ at every write. (Older versions hashed the plain file, so old "oz" values still match.)
fn esitle_durum() -> (&'static str, Option<PathBuf>) {
    let e = read_json("esitle.json");
    if e["acik"] != true { return ("kapali", onedrive()); }
    let Some(k) = onedrive() else { return ("klasor_yok", None) };
    let u = match uzak_metin(&k) {
        None => None,
        Some(Ok(s)) => Some(oz(s.as_bytes())),
        Some(Err(())) => return ("parola", Some(k)), // encrypted with a password this PC does not have
    };
    let y = yerel_metin().map(|s| oz(s.as_bytes()));
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
    let Some(duz) = yerel_metin() else { return };
    let hedef = k.join("RYOCS");
    // with a password the copy is encrypted; without one it stays plain JSON as before (the UI warns about that)
    let Some(veri) = (match onedrive_parola() { Some(p) => parola_sifrele(&p, &duz), None => Some(duz.clone()) }) else { return };
    let sonuc = fs::create_dir_all(&hedef).map_err(|e| e.to_string()).and_then(|_| dayanikli_yaz(&hedef.join("data.json"), veri.as_bytes()));
    match sonuc {
        Ok(()) => esitle_oz_yaz(&oz(duz.as_bytes())),
        Err(e) => hata_kaydet(&format!("OneDrive sync write failed: {e}")),
    }
}
/// Replaces local with the remote data; local stays as the "State before restore" backup.
fn esitle_cek(k: &std::path::Path) -> Result<(), String> {
    let bozuk = || l("OneDrive'daki veri dosyası bozuk", "The data file on OneDrive is corrupt").to_string();
    let s = uzak_metin(k).ok_or_else(bozuk)?.map_err(|_| l("OneDrive kopyasının parolası gerekli", "The OneDrive copy needs its password").to_string())?;
    if !serde_json::from_str::<Value>(&s).is_ok_and(|v| v.is_object()) { return Err(bozuk()); }
    let yedek = dir().join("yedekler");
    fs::create_dir_all(&yedek).ok();
    fs::copy(dir().join("data.json"), yedek.join("data-geri-alma.json")).ok();
    // the same plain text is written (encrypted): its hash becomes the last synced hash, so the next remote change is not a "conflict"
    write_text("data.json", &s)?;
    esitle_oz_yaz(&oz(s.as_bytes()));
    Ok(())
}
fn esitle(b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).unwrap_or_default();
    if r["acik"].is_boolean() || r["klasor"].is_string() || r["parola"].is_string() {
        let mut e = read_json("esitle.json");
        if let Some(a) = r["acik"].as_bool() { e["acik"] = json!(a); }
        if let Some(p) = r["parola"].as_str() {
            if !p.is_empty() && p.chars().count() < 8 { return Err(l("Parola en az 8 karakter olmalı", "The password needs at least 8 characters").into()); }
            // A copy already password-encrypted on OneDrive is opened with the password this PC has (or the one typed: the first time
            // on a second PC) and written again under the new setting. Otherwise removing or changing the password would leave a copy
            // that this PC can no longer open, and sync would stop. A password that opens nothing is refused (a typo would split the PCs).
            if let Some(k) = onedrive() {
                let yol = k.join("RYOCS").join("data.json");
                let uzak = fs::read_to_string(&yol).unwrap_or_default();
                if uzak.trim_start().starts_with(PAROLA_ONEK) {
                    let duz = onedrive_parola().and_then(|o| parola_coz(&o, &uzak)).or_else(|| parola_coz(p, &uzak)).ok_or(if p.is_empty() {
                        l("OneDrive kopyası bu bilgisayarın bilmediği bir parolayla şifreli; önce o parolayı gir", "The OneDrive copy uses a password this computer does not know; enter that password first")
                    } else {
                        l("Bu parola OneDrive'daki kopyayı açmıyor", "This password does not open the copy on OneDrive")
                    })?;
                    let yeni = if p.is_empty() { Some(duz) } else { parola_sifrele(p, &duz) }.ok_or("AES-GCM")?;
                    dayanikli_yaz(&yol, yeni.as_bytes())?; // same plain text: the sync fingerprints do not change
                }
            }
            if p.is_empty() {
                e.as_object_mut().map(|o| o.remove("parola"));
            } else {
                e["parola"] = json!(dpapi_sar(p)?);
            }
        }
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
                let uzak = uzak_metin(k).and_then(Result::ok).ok_or(l("OneDrive kopyası okunamadı", "Could not read the OneDrive copy"))?;
                fs::create_dir_all(dir().join("yedekler")).ok();
                write_text("yedekler/data-onedrive.json", &uzak)?;
                esitle_oz_yaz(&oz(uzak.as_bytes()));
            }
            d = if sec == "uzak" { "guncellendi" } else { "ok" };
        }
    } else if d == "uzak_yeni" && r["baslangic"] == true {
        if let Some(k) = k.as_ref() { esitle_cek(k)?; d = "guncellendi"; }
    }
    if d == "ok" { esitle_gonder(); }
    Ok(json!({ "durum": d, "klasor": k, "acik": read_json("esitle.json")["acik"] == true, "parola": onedrive_parola().is_some() }))
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
    // a secret encrypted on another PC (config.json copied over) cannot be kept: without a new one Google asks for it below
    if c[p.name]["client_secret"].as_str().is_some_and(|g| dpapi_ac(g).is_none()) {
        c[p.name]["client_secret"] = json!("");
    }
    c[p.name]["client_id"] = json!(id);
    if !yeni_gizli.is_empty() {
        c[p.name]["client_secret"] = json!(dpapi_sar(yeni_gizli)?); // if left empty the old one is kept
    }
    if p.name == "google" && c["google"]["client_secret"].as_str().unwrap_or("").is_empty() {
        return Err(l("Google için gizli anahtar (client secret) da gerekli", "Google also needs the client secret").into());
    }
    config_yaz(&c)?;
    disconnect(p)?; // old tokens do not work with new credentials; whoever redoes the setup is reconnecting anyway
    if p.name == "google" { // the extra Gmail accounts were signed in with the old client too: they stay listed, to reconnect
        let e: Vec<String> = tokens_oku()["posta"].as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
        for x in e { posta_kaydet(&x, None); }
    }
    connect(p)
}

fn creds(p: &str) -> Result<(String, String), String> {
    let p = if p == "gmail" { "google" } else { p }; // extra Gmail accounts sign in with the main Google client
    let c = read_json("config.json");
    let id = c[p]["client_id"].as_str().unwrap_or("").trim();
    if id.is_empty() {
        return Err("kimlik_yok".into());
    }
    // DPAPI-encrypted by RYOCS, or plain as the user typed it into the file (encrypted at the next start)
    Ok((id.into(), dpapi_coz(c[p]["client_secret"].as_str().unwrap_or(""))?))
}

/// For the settings dialog: city and account states (credential/token values are never returned).
fn config_get() -> Value {
    let (c, t) = (read_json("config.json"), tokens_oku());
    let hesap = |p: &str| {
        if creds(p).is_err() { "kimlik_yok" } else if t[p]["refresh_token"].is_string() { "ok" } else { "bagli_degil" }
    };
    json!({
        "sehir": c["sehir"].as_str().unwrap_or(varsayilan_sehir()),
        // only the client ID to fill the setup dialog (never the secret)
        "google_id": c["google"]["client_id"].as_str().unwrap_or(""),
        "spotify_id": c["spotify"]["client_id"].as_str().unwrap_or(""),
        "google_izin": google_tam_izin(), // whether the calendar list permission was granted too
        "google": hesap("google"), "spotify": hesap("spotify"),
        // extra Gmail accounts: address and whether they still work
        "posta": t["posta"].as_object().into_iter().flatten()
            .map(|(e, v)| json!({ "eposta": e, "durum": if v["refresh_token"].is_string() { "ok" } else { "bagli_degil" } })).collect::<Vec<_>>(),
        "surum": env!("CARGO_PKG_VERSION"),
        "kisayol": tepsi::kisayol(),
        "kisayollar": tepsi::adlar(),
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
    let mut all = tokens_oku();
    if let Some(o) = all.as_object_mut() {
        o.remove(p.name);
    }
    tokens_yaz(&all)?;
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
    let mut all = tokens_oku();
    if !all.is_object() {
        all = json!({});
    }
    if t["refresh_token"].is_null() {
        t["refresh_token"] = all[p]["refresh_token"].clone(); // yenilemede yeni refresh_token gelmeyebilir
        t["eposta"] = all[p]["eposta"].clone(); // a refresh is the same account: its address stays (a new sign-in reads it again)
    }
    t["expires_at"] = json!(now() + t["expires_in"].as_u64().unwrap_or(3600));
    all[p] = t;
    if let Err(e) = tokens_yaz(&all) {
        hata_kaydet(&format!("tokens.json write failed: {e}"));
    }
}

/// Valid access token; refreshes if needed. Error: "kimlik_yok" | "bagli_degil" | message.
fn token(p: &Provider) -> Result<String, String> {
    let (id, secret) = creds(p.name)?;
    let _tek = kilit(&YENILEME[(p.name == "spotify") as usize]); // if another request is refreshing, wait for it and use its result
    match tazele(p, &tokens_oku()[p.name], &id, &secret)? {
        Tazele::Gecerli(a) => Ok(a),
        Tazele::Yeni(n) => { save_token(p.name, n.clone()); n["access_token"].as_str().map(String::from).ok_or("bagli_degil".into()) }
        Tazele::Iptal => { disconnect(p).ok(); Err("bagli_degil".into()) }
    }
}
enum Tazele { Gecerli(String), Yeni(Value), Iptal }
/// The stored access token if it is still good, otherwise a refreshed one; Iptal when the refresh token was revoked or expired
/// (a Google project in "Testing" mode: after 7 days): retrying never helps, so the card offers to reconnect.
fn tazele(p: &Provider, t: &Value, id: &str, secret: &str) -> Result<Tazele, String> {
    if t["expires_at"].as_u64().unwrap_or(0) > now() + 60 {
        if let Some(a) = t["access_token"].as_str() { return Ok(Tazele::Gecerli(a.into())); }
    }
    let rt = t["refresh_token"].as_str().ok_or("bagli_degil")?;
    match body(AGENT.post(p.token).send_form(&gizli_ekle(vec![("grant_type", "refresh_token"), ("refresh_token", rt), ("client_id", id)], secret))) {
        Err(e) if iptal_edildi(&e) => Ok(Tazele::Iptal),
        r => Ok(Tazele::Yeni(r?)),
    }
}

/// Extra Gmail accounts (mail only): tokens.json "posta": {address: {access_token, refresh_token, expires_at, scope}}.
/// A revoked one stays listed without tokens, so the card can offer to reconnect it.
fn posta_token(eposta: &str) -> Result<String, String> {
    let p = provider("gmail")?;
    let (id, secret) = creds("gmail")?;
    let _tek = kilit(&YENILEME[0]);
    match tazele(&p, &tokens_oku()["posta"][eposta], &id, &secret)? {
        Tazele::Gecerli(a) => Ok(a),
        Tazele::Yeni(n) => { posta_kaydet(eposta, Some(n.clone())); n["access_token"].as_str().map(String::from).ok_or("bagli_degil".into()) }
        Tazele::Iptal => { posta_kaydet(eposta, None); Err("bagli_degil".into()) }
    }
}
/// Writes an extra account's token (None: revoked, the address stays without tokens).
fn posta_kaydet(eposta: &str, t: Option<Value>) {
    let _d = kilit(&BELIRTEC_DOSYASI);
    let mut all = tokens_oku();
    if !all.is_object() { all = json!({}); }
    if !all["posta"].is_object() { all["posta"] = json!({}); }
    let v = match t {
        Some(mut t) if t.is_object() => {
            if t["refresh_token"].is_null() { t["refresh_token"] = all["posta"][eposta]["refresh_token"].clone(); }
            t["expires_at"] = json!(now() + t["expires_in"].as_u64().unwrap_or(3600));
            t
        }
        _ => json!({}),
    };
    all["posta"][eposta] = v;
    if let Err(e) = tokens_yaz(&all) { hata_kaydet(&format!("tokens.json write failed: {e}")); }
}
/// The Gmail address behind an access token.
fn gmail_adresi(auth: &str) -> Result<String, String> {
    let r = body(AGENT.get("https://gmail.googleapis.com/gmail/v1/users/me/profile").query("fields", "emailAddress").set("Authorization", auth).call())?;
    r["emailAddress"].as_str().filter(|e| e.contains('@')).map(|e| e.to_lowercase()).ok_or("profil okunamadı".into())
}
/// The main Google account's address, read once and kept with its token.
fn ana_eposta(auth: &str) -> Option<String> {
    if let Some(e) = tokens_oku()["google"]["eposta"].as_str() { return Some(e.into()); }
    let e = gmail_adresi(auth).ok()?;
    let _d = kilit(&BELIRTEC_DOSYASI);
    let mut all = tokens_oku();
    if all["google"].is_object() { all["google"]["eposta"] = json!(e); tokens_yaz(&all).ok(); }
    Some(e)
}
/// After signing in an extra account: its address is read and the token stored under it. The main account is not added twice.
fn posta_ekle(t: Value) -> Result<(), String> {
    let a = t["access_token"].as_str().ok_or("bagli_degil")?;
    let auth = format!("Bearer {a}");
    let e = gmail_adresi(&auth)?;
    let ana = token(&provider("google")?).ok().and_then(|x| ana_eposta(&format!("Bearer {x}")));
    if ana.as_deref() == Some(e.as_str()) {
        return Err(l("Bu hesap zaten ana Google hesabın; başka bir hesap seç.", "This is already your main Google account; choose another one.").into());
    }
    posta_kaydet(&e, Some(t));
    Ok(())
}
fn posta_sil(b: &str) -> Result<Value, String> {
    let e = serde_json::from_str::<Value>(b).ok().and_then(|v| v["eposta"].as_str().map(String::from)).ok_or("eposta yok")?;
    let _d = kilit(&BELIRTEC_DOSYASI);
    let mut all = tokens_oku();
    if let Some(o) = all["posta"].as_object_mut() { o.remove(&e); }
    tokens_yaz(&all)?;
    Ok(Value::Null)
}

/// Token endpoint answer for a refresh token that no longer works (Google and Spotify: 400 invalid_grant).
fn iptal_edildi(hata: &str) -> bool {
    hata.starts_with("400:") && hata.contains("invalid_grant")
}

fn connect(p: &Provider) -> Result<Value, String> {
    let (id, _) = creds(p.name)?;
    let (st, dogrulayici) = (rastgele(), rastgele());
    let meydan = pkce(&dogrulayici);
    kilit(&OAUTH_STATE).insert(p.name, (st.clone(), dogrulayici));
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

fn uygulama_ac(yol: &str, yonetici: bool) -> Result<Value, String> {
    uygulama_gecerli(yol)?;
    let p = std::path::Path::new(yol);
    if yonetici {
        // "runas": Windows asks for permission (UAC); a value above 32 means it started
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        let g = |s: &std::ffi::OsStr| -> Vec<u16> { use std::os::windows::ffi::OsStrExt; s.encode_wide().chain([0]).collect() };
        let (fiil, dosya, klasor) = (g("runas".as_ref()), g(p.as_os_str()), g(p.parent().unwrap_or(p).as_os_str()));
        let r = unsafe { ShellExecuteW(std::ptr::null_mut(), fiil.as_ptr(), dosya.as_ptr(), std::ptr::null(), klasor.as_ptr(), 1) } as isize;
        return if r > 32 { Ok(Value::Null) } else { Err(l("Yönetici olarak açılamadı (izin verilmedi ya da dosya yok)", "Could not open as administrator (not allowed or the file is missing)").into()) };
    }
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

/// Quick link icon: the site's own favicon, fetched from the site itself (no third-party icon service learns the user's links).
/// /favicon.ico first; if that is not an image, the first <link rel="…icon…"> of the home page. Returned as a data: URL.
fn favicon(adres: &str) -> Result<Value, String> {
    let u = url::Url::parse(adres).map_err(|e| e.to_string())?;
    if !matches!(u.scheme(), "http" | "https") || u.host_str().is_none() {
        return Err("http(s) değil".into());
    }
    let oku = |a: &str| -> Option<(String, Vec<u8>)> {
        let r = AGENT.get(a).call().ok()?;
        let tur = r.content_type().to_string();
        let mut b = Vec::new();
        r.into_reader().take(300_000).read_to_end(&mut b).ok()?;
        Some((tur, b))
    };
    let resim = |(tur, b): (String, Vec<u8>)| simge_turu(&tur, &b).map(|t| (t, b));
    let kok = u.join("/").map_err(|e| e.to_string())?;
    let bulunan = kok.join("/favicon.ico").ok().and_then(|a| oku(a.as_str())).and_then(resim).or_else(|| {
        let (_, sayfa) = oku(kok.as_str())?;
        let href = ikon_href(&String::from_utf8_lossy(&sayfa))?;
        oku(kok.join(&href).ok()?.as_str()).and_then(resim)
    });
    use base64::Engine;
    let (tur, b) = bulunan.ok_or("simge yok")?;
    Ok(Value::String(format!("data:{tur};base64,{}", base64::engine::general_purpose::STANDARD.encode(b))))
}
/// Image type of a downloaded icon from its header or its first bytes (servers often send .ico as octet-stream); None if not an image.
fn simge_turu(tur: &str, b: &[u8]) -> Option<String> {
    if b.is_empty() {
        return None;
    }
    if b.starts_with(&[0, 0, 1, 0]) {
        return Some("image/x-icon".into());
    }
    if b.starts_with(b"\x89PNG") {
        return Some("image/png".into());
    }
    let t = tur.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    t.starts_with("image/").then_some(t) // an SVG shown in <img> cannot run scripts
}
/// href of the first <link> whose rel contains "icon" (icon, shortcut icon, apple-touch-icon).
fn ikon_href(html: &str) -> Option<String> {
    let k = html.to_ascii_lowercase();
    let mut i = 0;
    while let Some(j) = k[i..].find("<link") {
        let bas = i + j;
        let son = bas + k[bas..].find('>')?;
        let etiket = &k[bas..son];
        let deger = |ad: &str| {
            let p = etiket.find(&format!("{ad}="))? + ad.len() + 1;
            let q = etiket[p..].chars().next()?;
            let (a, b) = if q == '"' || q == '\'' { (p + 1, etiket[p + 1..].find(q)? + p + 1) } else { (p, etiket[p..].find(char::is_whitespace).map_or(etiket.len(), |x| x + p)) };
            Some((a, b))
        };
        if deger("rel").is_some_and(|(a, b)| etiket[a..b].contains("icon")) {
            let (a, b) = deger("href")?;
            return Some(html[bas + a..bas + b].trim().to_string()); // original case: paths are case-sensitive
        }
        i = son;
    }
    None
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
/// Today's local date for backup and export names: a save at 01:00 in Istanbul belongs to that day, not to yesterday in UTC.
fn bugun() -> String {
    let mut t: windows_sys::Win32::Foundation::SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t) };
    format!("{:04}-{:02}-{:02}", t.wYear, t.wMonth, t.wDay)
}
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
    let f = b.join(format!("data-{}.json", bugun()));
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
    // removed only when the state matches: a stray /callback from a web page must not cancel a sign-in in progress
    let (st, dogrulayici) = {
        let mut s = kilit(&OAUTH_STATE);
        if s.get(p.name).is_some_and(|(st, _)| q.get("state") == Some(st)) { s.remove(p.name).unwrap_or_default() } else { Default::default() }
    };
    if st.is_empty() {
        return Err(l("Geçersiz istek (state uyuşmuyor). Uygulamadan yeniden bağlanın.", "Invalid request (state mismatch). Connect again from the app.").into());
    }
    let code = q.get("code").ok_or_else(|| format!("{} {}", l("İzin verilmedi:", "Permission denied:"), q.get("error").map_or("", |s| s)))?;
    let (id, secret) = creds(p.name)?;
    let red = redirect(p.name);
    let t = body(AGENT.post(p.token).send_form(&gizli_ekle(vec![
        ("grant_type", "authorization_code"), ("code", code), ("redirect_uri", &red),
        ("client_id", &id), ("code_verifier", &dogrulayici),
    ], &secret)))?;
    if p.name == "gmail" { return posta_ekle(t); }
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

/// Gmail message IDs go into URL paths: letters/digits only.
fn mail_id(i: &str) -> bool {
    !i.is_empty() && i.len() <= 64 && i.bytes().all(|c| c.is_ascii_alphanumeric())
}

/// Gmail: unread count in the inbox and the headers of the last 15 inbox messages, each with its unread state.
/// Reading works with the old gmail.readonly permission too; marking as read needs gmail.modify ("izin_yok" otherwise).
/// All accounts: the main Google one (its failure is the card's error, as before) and the extra Gmail accounts (each fetched at
/// the same time; a failing one is reported in "hesaplar" and the others still come). Mail of all accounts, newest first.
fn mail() -> Result<Value, String> {
    let auth = format!("Bearer {}", token(&provider("google")?)?);
    if !tokens_oku()["google"]["scope"].as_str().is_some_and(|s| s.contains("gmail.readonly") || s.contains("gmail.modify")) {
        return Err("izin_yok".into());
    }
    let ana = ana_eposta(&auth).unwrap_or_default();
    let ekler: Vec<String> = tokens_oku()["posta"].as_object().map(|o| o.keys().filter(|e| **e != ana).cloned().collect()).unwrap_or_default();
    let (a, ek) = thread::scope(|k| {
        let is: Vec<_> = ekler.iter().map(|e| k.spawn(move || (e.clone(), posta_token(e).and_then(|t| mail_kutusu(e, &format!("Bearer {t}")))))).collect();
        let a = mail_kutusu(&ana, &auth);
        (a, is.into_iter().filter_map(|i| i.join().ok()).collect::<Vec<_>>())
    });
    let a = a?;
    let mut hesaplar = vec![json!({ "eposta": ana, "ana": true, "okunmamis": a["okunmamis"] })];
    let mut liste: Vec<Value> = a["liste"].as_array().cloned().unwrap_or_default();
    let mut top = a["okunmamis"].as_u64().unwrap_or(0);
    for (e, r) in ek {
        match r {
            Ok(v) => {
                top += v["okunmamis"].as_u64().unwrap_or(0);
                hesaplar.push(json!({ "eposta": e, "okunmamis": v["okunmamis"] }));
                liste.extend(v["liste"].as_array().cloned().unwrap_or_default());
            }
            Err(h) => hesaplar.push(json!({ "eposta": e, "hata": h })),
        }
    }
    liste.sort_by_key(|m| std::cmp::Reverse(m["t"].as_i64().unwrap_or(0)));
    Ok(json!({ "okunmamis": top, "liste": liste, "hesaplar": hesaplar }))
}
/// One mailbox: unread count in the inbox and the last 15 inbox messages; each message carries its account ("hesap").
fn mail_kutusu(hesap: &str, auth: &str) -> Result<Value, String> {
    let g = "https://gmail.googleapis.com/gmail/v1/users/me";
    let kutu = body(AGENT.get(&format!("{g}/labels/INBOX")).query("fields", "messagesUnread").set("Authorization", auth).call())?;
    let idler_al = |q: &str| -> Result<Vec<String>, String> {
        let l = body(AGENT.get(&format!("{g}/messages")).query("q", q).query("maxResults", "15")
            .query("fields", "messages(id)").set("Authorization", auth).call())?;
        Ok(l["messages"].as_array().into_iter().flatten().filter_map(|m| m["id"].as_str()).filter(|i| mail_id(i)).map(String::from).collect())
    };
    let idler = idler_al("in:inbox")?;
    // the unread state changes, the headers do not: a second id-only list says which of the 15 are unread
    let okunmamis = idler_al("is:unread in:inbox")?;
    // headers of a message do not change: only new ids are fetched; the memory keeps just the current list, per account
    static ONBELLEK: LazyLock<Mutex<HashMap<String, HashMap<String, Value>>>> = LazyLock::new(Default::default);
    let yeni: Vec<&String> = { let o = kilit(&ONBELLEK); idler.iter().filter(|i| !o.get(hesap).is_some_and(|h| h.contains_key(*i))).collect() };
    let gelen: Vec<Value> = thread::scope(|k| {
        let isler: Vec<_> = yeni.iter().map(|id| { k.spawn(move || {
            body(AGENT.get(&format!("{g}/messages/{id}")).query("format", "metadata")
                .query("metadataHeaders", "From").query("metadataHeaders", "Subject")
                .query("fields", "id,threadId,internalDate,snippet,payload/headers").set("Authorization", auth).call())
        })}).collect();
        isler.into_iter().filter_map(|h| h.join().ok()?.ok()).collect()
    });
    let mailler = mail_birlestir(kilit(&ONBELLEK).entry(hesap.to_string()).or_default(), gelen, &idler);
    let baslik = |m: &Value, ad: &str| m["payload"]["headers"].as_array().into_iter().flatten()
        .find(|h| h["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(ad))).and_then(|h| h["value"].as_str()).unwrap_or("").to_string();
    let liste: Vec<Value> = mailler.iter().map(|m| json!({
        "id": m["id"], "hesap": hesap, "konu": baslik(m, "Subject"), "kimden": baslik(m, "From"),         "zincir": m["threadId"], "t": m["internalDate"].as_str().and_then(|t| t.parse::<i64>().ok()),
        "yeni": m["id"].as_str().is_some_and(|i| okunmamis.iter().any(|o| o == i)),
        "ozet": m["snippet"].as_str().map(|t| t.chars().take(200).collect::<String>()),
    })).collect();
    Ok(json!({ "okunmamis": kutu["messagesUnread"], "liste": liste }))
}

/// Changes one message's labels: read (UNREAD off), unread (UNREAD on) or archive (INBOX off, it stays in All Mail).
/// These are the only changes RYOCS makes in the mailbox: nothing is deleted or sent.
fn mail_islem(req_body: &str) -> Result<Value, String> {
    let v: Value = serde_json::from_str(req_body).map_err(|e| e.to_string())?;
    let id = v["id"].as_str().filter(|i| mail_id(i)).ok_or("geçersiz id")?;
    let degisim = match v["islem"].as_str().unwrap_or("okundu") {
        "okundu" => json!({ "removeLabelIds": ["UNREAD"] }),
        "okunmadi" => json!({ "addLabelIds": ["UNREAD"] }),
        "arsiv" => json!({ "removeLabelIds": ["INBOX"] }),
        _ => return Err("geçersiz işlem".into()),
    };
    // an extra account by its address; the main one when none is given or it is the main address
    let hesap = v["hesap"].as_str().unwrap_or("");
    let auth = if !hesap.is_empty() && tokens_oku()["posta"][hesap].is_object() {
        format!("Bearer {}", posta_token(hesap)?)
    } else {
        let a = format!("Bearer {}", token(&provider("google")?)?);
        if !tokens_oku()["google"]["scope"].as_str().is_some_and(|s| s.contains("gmail.modify")) {
            return Err("izin_yok".into());
        }
        a
    };
    body(AGENT.post(&format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{id}/modify")).set("Authorization", &auth)
        .send_json(degisim))?;
    Ok(json!(true))
}

/// Adds the newly fetched messages to the memory, drops the ones no longer unread, returns them in the order of `idler`.
fn mail_birlestir(o: &mut HashMap<String, Value>, gelen: Vec<Value>, idler: &[String]) -> Vec<Value> {
    for m in gelen { if let Some(i) = m["id"].as_str() { o.insert(i.into(), m); } }
    o.retain(|i, _| idler.contains(i));
    idler.iter().filter_map(|i| o.get(i).cloned()).collect()
}
fn google_tam_izin() -> bool {
    tokens_oku()["google"]["scope"].as_str().is_some_and(|s| s.contains("calendar.calendarlist"))
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
    // "yok": the user turned public holidays off
    let tatil = r["tatil"].as_str().filter(|t| takvim_id(t) && t.ends_with("#holiday@group.v.calendar.google.com")).unwrap_or(TR_TATIL);
    if r["tatil"] != "yok" && !takvimler.iter().any(|c| c["id"] == tatil) {
        let ad = if tatil == TR_TATIL { l("Türkiye'deki tatiller", "Holidays in Turkey") } else { l("Tatiller", "Holidays") };
        takvimler.push(json!({ "id": tatil, "ad": ad, "renk": "#e9a25b", "yaz": false }));
    }
    let sonuc: Vec<Result<Value, String>> = thread::scope(|k| {
        let isler: Vec<_> = takvimler.iter().map(|c| {
            let (id, auth) = (c["id"].as_str().unwrap_or("primary").to_string(), &auth);
            k.spawn(move || {
                // a busy calendar has more than one page of 250 events (at most 4 pages: 1000 events per calendar)
                let (mut items, mut sayfa) = (vec![], None::<String>);
                for _ in 0..4 {
                    let mut q = AGENT.request_url("GET", &takvim_url(&id, &["events"])?)
                        .query("timeMin", min).query("timeMax", max)
                        .query("singleEvents", "true").query("orderBy", "startTime").query("maxResults", "250")
                        // only the fields the UI uses: on a slow network the response is several times smaller
                        .query("fields", "nextPageToken,items(id,summary,start,end,recurringEventId,location,htmlLink,hangoutLink,conferenceData/entryPoints(uri,entryPointType),extendedProperties,description,eventType)");
                    if let Some(t) = &sayfa {
                        q = q.query("pageToken", t);
                    }
                    let v = body(q.set("Authorization", auth).call())?;
                    items.extend(v["items"].as_array().cloned().unwrap_or_default());
                    match v["nextPageToken"].as_str() {
                        Some(t) => sayfa = Some(t.into()),
                        None => break,
                    }
                }
                Ok(json!({ "items": items }))
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

/// Edits an event: title and/or start + end (PATCH: only the fields sent change). Checked like adding, before any network call.
fn calendar_edit(b: &str) -> Result<Value, String> {
    let r: Value = serde_json::from_str(b).map_err(|e| e.to_string())?;
    let id = r["id"].as_str().filter(|s| etkinlik_id(s)).ok_or(l("Geçersiz etkinlik", "Invalid event"))?;
    let takvim = r["takvim"].as_str().unwrap_or("primary");
    if !takvim_id(takvim) {
        return Err(l("Geçersiz takvim", "Invalid calendar").into());
    }
    let mut ev = json!({});
    if let Some(s) = r["summary"].as_str() {
        let s = s.trim();
        if s.is_empty() || s.chars().count() > 300 {
            return Err(l("Başlık gerekli", "A title is required").into());
        }
        ev["summary"] = json!(s);
    }
    // the UI sends the other key as null ({date, dateTime: null}): PATCH merges objects, so a timed event can become all-day
    if r["start"].is_object() && r["end"].is_object() {
        ev["start"] = r["start"].clone();
        ev["end"] = r["end"].clone();
    }
    if ev.as_object().is_some_and(|o| o.is_empty()) {
        return Err(l("Değişiklik yok", "Nothing to change").into());
    }
    let t = token(&provider("google")?)?;
    body(AGENT.request_url("PATCH", &takvim_url(takvim, &["events", id])?).set("Authorization", &format!("Bearer {t}")).send_json(ev))
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
    geri_yukle(serde_json::from_str(&dpapi_coz(&s)?).map_err(|e| e.to_string())?)
}

/// The user's Downloads folder, also when it was moved (Known Folder API); None if Windows cannot tell.
fn indirilenler() -> Option<PathBuf> {
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{FOLDERID_Downloads, SHGetKnownFolderPath};
    unsafe {
        let mut p = std::ptr::null_mut();
        let ok = SHGetKnownFolderPath(&FOLDERID_Downloads, 0, std::ptr::null_mut(), &mut p) >= 0 && !p.is_null();
        let yol = ok.then(|| PathBuf::from(String::from_utf16_lossy(std::slice::from_raw_parts(p, (0..).take_while(|&i| *p.add(i) != 0).count()))));
        CoTaskMemFree(p as _); // also required when the call fails
        yol
    }
}

fn disa_aktar() -> Result<Value, String> {
    use std::os::windows::process::CommandExt;
    let klasor = match indirilenler() {
        Some(k) => k,
        None => PathBuf::from(std::env::var("USERPROFILE").map_err(|e| e.to_string())?).join("Downloads"),
    };
    let hedef = klasor.join(format!("RYOCS-yedek-{}.json", bugun()));
    // plain JSON on purpose: the only copy that still opens after a Windows reinstall or on another PC (the UI says it is not encrypted)
    fs::write(&hedef, serde_json::to_string_pretty(&veri_oku()?).unwrap_or_default()).map_err(|e| e.to_string())?;
    quiet("explorer").raw_arg(format!("/select,\"{}\"", hedef.display())).spawn().ok();
    Ok(json!(hedef))
}

/// A Spotify player command's query value, checked before any request (the UI is the only caller, but the values reach the URL).
fn spotify_deger(cmd: &str, b: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(b).unwrap_or_default();
    let hata = || l("Geçersiz değer", "Invalid value").to_string();
    match cmd {
        "volume" => v["v"].as_u64().filter(|v| *v <= 100).map(|v| v.to_string()).ok_or(l("Geçersiz ses düzeyi", "Invalid volume").into()),
        "seek" => v["ms"].as_u64().filter(|v| *v <= 24 * 3_600_000).map(|v| v.to_string()).ok_or_else(hata),
        "shuffle" => v["on"].as_bool().map(|v| v.to_string()).ok_or_else(hata),
        "repeat" => v["mod"].as_str().filter(|m| ["off", "context", "track"].contains(m)).map(String::from).ok_or_else(hata),
        // device ids are letters and digits; the id goes into the JSON body
        "transfer" => v["id"].as_str().filter(|i| !i.is_empty() && i.len() <= 128 && i.bytes().all(|c| c.is_ascii_alphanumeric())).map(String::from).ok_or_else(hata),
        _ => Ok(String::new()),
    }
}

fn spotify(cmd: &str, b: &str) -> Result<Value, String> {
    let deger = spotify_deger(cmd, b)?;
    let auth = format!("Bearer {}", token(&provider("spotify")?)?);
    let base = "https://api.spotify.com/v1/me/player";
    body(match cmd {
        // market: so the hundreds of country codes (available_markets) are not returned; episode: podcasts too (else item is null)
        "" => AGENT.get(base).query("market", "from_token").query("additional_types", "track,episode").set("Authorization", &auth).call(),
        "play" | "pause" => AGENT.put(&format!("{base}/{cmd}")).set("Authorization", &auth).send_string(""),
        "next" | "previous" => AGENT.post(&format!("{base}/{cmd}")).set("Authorization", &auth).send_string(""),
        "queue" | "devices" => AGENT.get(&format!("{base}/{cmd}")).set("Authorization", &auth).call(),
        "volume" => AGENT.put(&format!("{base}/volume")).query("volume_percent", &deger).set("Authorization", &auth).send_string(""),
        "seek" => AGENT.put(&format!("{base}/seek")).query("position_ms", &deger).set("Authorization", &auth).send_string(""),
        "shuffle" => AGENT.put(&format!("{base}/shuffle")).query("state", &deger).set("Authorization", &auth).send_string(""),
        "repeat" => AGENT.put(&format!("{base}/repeat")).query("state", &deger).set("Authorization", &auth).send_string(""),
        // keeps playing on the new device if something was playing
        "transfer" => AGENT.put(base).set("Authorization", &auth).send_json(json!({ "device_ids": [deger], "play": true })),
        _ => return Err("bilinmeyen komut".into()),
    })
}

/// Only requests from our own page: blocks DNS rebinding (Host) and POSTs from other sites (Origin).
fn allowed(host: Option<&str>, origin: Option<&str>) -> bool {
    host == Some(ADDR) && origin.is_none_or(|o| o == ORIGIN)
}
/// Key made at every start and given only to the app's own window (initialization script, not the served HTML).
/// Without it another Windows user or a browser tab on this PC could read /api/state or start programs through /api/uygulama.
static ANAHTAR: LazyLock<String> = LazyLock::new(rastgele);
/// /api/show stays open: a second RYOCS instance uses it to bring this window forward (it only shows the window).
fn anahtar_gerekli(path: &str) -> bool {
    path.starts_with("/api/") && path != "/api/show"
}

/// Poppins (SIL Open Font License, assets/yazi/OFL.txt) inside the exe: the UI looks the same on a PC without the font.
const YAZILAR: [(&str, &[u8]); 10] = [
    ("poppins-300-latin.woff2", include_bytes!("../assets/yazi/poppins-300-latin.woff2")),
    ("poppins-300-ext.woff2", include_bytes!("../assets/yazi/poppins-300-ext.woff2")),
    ("poppins-400-latin.woff2", include_bytes!("../assets/yazi/poppins-400-latin.woff2")),
    ("poppins-400-ext.woff2", include_bytes!("../assets/yazi/poppins-400-ext.woff2")),
    ("poppins-500-latin.woff2", include_bytes!("../assets/yazi/poppins-500-latin.woff2")),
    ("poppins-500-ext.woff2", include_bytes!("../assets/yazi/poppins-500-ext.woff2")),
    ("poppins-600-latin.woff2", include_bytes!("../assets/yazi/poppins-600-latin.woff2")),
    ("poppins-600-ext.woff2", include_bytes!("../assets/yazi/poppins-600-ext.woff2")),
    ("poppins-700-latin.woff2", include_bytes!("../assets/yazi/poppins-700-latin.woff2")),
    ("poppins-700-ext.woff2", include_bytes!("../assets/yazi/poppins-700-ext.woff2")),
];

fn send(req: Request, code: u16, ctype: &str, text: String) {
    let h = Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes()).unwrap();
    req.respond(Response::from_string(text).with_status_code(code).with_header(h)).ok();
}

/// Saves in progress and the time the last one started or ended (ms): quitting waits for them instead of a fixed time.
static KAYIT_SAYI: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static KAYIT_SON: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn simdi_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}
struct KayitSuruyor;
impl KayitSuruyor {
    fn basla() -> Self {
        KAYIT_SAYI.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        KAYIT_SON.store(simdi_ms(), std::sync::atomic::Ordering::SeqCst);
        KayitSuruyor
    }
}
impl Drop for KayitSuruyor {
    fn drop(&mut self) {
        KAYIT_SON.store(simdi_ms(), std::sync::atomic::Ordering::SeqCst);
        KAYIT_SAYI.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}
/// Quit only when no save runs and none started or ended in the last 400 ms (the UI sends queued saves one after another), at most 5 s.
fn kayitlari_bekle() {
    use std::sync::atomic::Ordering::SeqCst;
    let bas = simdi_ms();
    while simdi_ms() - bas < 5000 && (KAYIT_SAYI.load(SeqCst) > 0 || simdi_ms().saturating_sub(KAYIT_SON.load(SeqCst)) < 400) {
        thread::sleep(Duration::from_millis(50));
    }
}

fn handle(mut req: Request) {
    let hdr = |n: &'static str| req.headers().iter().find(|h| h.field.equiv(n)).map(|h| h.value.as_str().to_string());
    let (host, origin) = (hdr("Host"), hdr("Origin"));
    if !allowed(host.as_deref(), origin.as_deref()) {
        return send(req, 403, "text/plain", "yasak".into());
    }
    let url = req.url().to_string();
    let path = url.split('?').next().unwrap_or("");
    if anahtar_gerekli(path) && hdr("X-Ryocs").as_deref() != Some(ANAHTAR.as_str()) {
        return send(req, 403, "text/plain", "yasak".into());
    }
    let mut b = String::new();
    // 16 MB: data.json with years of notes must still save (1 MB cut it off and the save failed)
    req.as_reader().take(16 << 20).read_to_string(&mut b).ok();
    let post = *req.method() == Method::Post;
    const HTML: &str = "text/html; charset=utf-8";

    if !post && path == "/" {
        return send(req, 200, HTML, include_str!("ui.html").replace("__BOLGE__", &bolge()));
    }
    if let (false, Some(ad)) = (post, path.strip_prefix("/yazi/")) {
        let Some((_, b)) = YAZILAR.iter().find(|y| y.0 == ad) else { return send(req, 404, "text/plain", "yok".into()) };
        let h = |a: &str, d: &str| Header::from_bytes(a.as_bytes(), d.as_bytes()).unwrap();
        req.respond(Response::from_data(b.to_vec()).with_header(h("Content-Type", "font/woff2")).with_header(h("Cache-Control", "max-age=31536000"))).ok();
        return;
    }
    if let (false, Some(name)) = (post, path.strip_prefix("/callback/")) {
        let sonuc = provider(name).and_then(|p| callback(&p, &url));
        if sonuc.is_ok() {
            olay(Olay::Goster(true)); // the user should return from the browser to the app
        }
        let servis = match name { "google" => l("Google Takvim", "Google Calendar"), "gmail" => "Gmail", _ => "Spotify" };
        return send(req, 200, HTML, sonuc_sayfasi(servis, sonuc));
    }
    let out: Result<Value, String> = if !post && path == "/api/state" {
        veri_oku()
    } else if post && path == "/api/state" {
        let _k = KayitSuruyor::basla();
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string())
            .and_then(|v| { backup(); write_json("data.json", &v)?; esitle_gonder(); Ok(Value::Null) })
    } else if post && path == "/api/calendar/add" {
        calendar_add(&b)
    } else if post && path == "/api/calendar/edit" {
        calendar_edit(&b)
    } else if post && path == "/api/notify" {
        notify(&b)
    } else if post && path == "/api/open" {
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()).and_then(|v| open_url(v["url"].as_str().unwrap_or("")))
    } else if post && path == "/api/uygulama" {
        serde_json::from_str::<Value>(&b).map_err(|e| e.to_string()).and_then(|v| uygulama_ac(v["yol"].as_str().unwrap_or(""), v["yonetici"] == true))
    } else if post && path == "/api/simgeler" {
        simgeler(&b)
    } else if !post && path == "/api/favicon" {
        url::form_urlencoded::parse(url.split_once('?').map_or("", |x| x.1).as_bytes()).find(|(k, _)| k == "u").ok_or("u yok".to_string()).and_then(|(_, u)| favicon(&u))
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
        // ?acilan=1 (recovery screen): only backups this Windows account can decrypt
        let l = yedek_listesi();
        if !url.contains("acilan=1") { Ok(l) } else {
            Ok(l.as_array().into_iter().flatten().filter(|a| a.as_str().is_some_and(|a| fs::read_to_string(dir().join("yedekler").join(a)).ok().and_then(|s| dpapi_ac(&s)).is_some_and(|s| serde_json::from_str::<Value>(&s).is_ok()))).cloned().collect())
        }
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
    } else if post && path == "/api/log/open" {
        let p = dir().join("hata.log");
        fs::OpenOptions::new().create(true).append(true).open(&p).ok(); // Notepad would ask to create a missing file
        quiet("notepad").arg(p).spawn().map(|_| Value::Null).map_err(|e| e.to_string())
    } else if post && path == "/api/kisayol" {
        // index into the shortcut list, or null: none
        let i = serde_json::from_str::<Value>(&b).ok().and_then(|v| v["i"].as_u64()).map_or(usize::MAX, |i| i as usize);
        if tepsi::degistir(i) { Ok(json!(tepsi::kisayol())) } else { Err(l("Bu kısayolu başka bir uygulama kullanıyor", "Another app already uses this shortcut").into()) }
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
    } else if post && path == "/api/guncelle" {
        guncelle_kur()
    } else if !post && path == "/api/mail" {
        mail()
    } else if post && path == "/api/mail_islem" {
        mail_islem(&b)
    } else if post && path == "/api/posta_sil" {
        posta_sil(&b)
    } else if let (false, "/api/kur" | "/api/kur_ecb") = (post, path) {
        let q = |k: &str| url.split_once('?').is_some_and(|(_, s)| url::form_urlencoded::parse(s.as_bytes()).any(|(a, b)| a == k && b == "1"));
        if path == "/api/kur" { kur(q("ag"), q("dunya")) } else { kur_ecb(q("ag"), q("dunya")) }
    } else if !post && path == "/api/sistem" {
        Ok(sistem())
    } else if post && path == "/api/pano" {
        Ok(pano(&b))
    } else if post && path == "/api/ekran" {
        Ok(ekran(&b))
    } else if post && path == "/api/esitle" {
        esitle(&b)
    } else if !post && path == "/api/onbellek" {
        Ok(read_json("onbellek.json"))
    } else if post && path == "/api/onbellek" {
        // calendar and held-notification cache for a fast start (was localStorage, which is plain text on disk)
        serde_json::from_str::<Value>(&b).ok().filter(Value::is_object).ok_or("geçersiz".to_string()).and_then(|v| write_json("onbellek.json", &v)).map(|_| Value::Null)
    } else if post && path == "/api/klasor" {
        quiet("explorer").arg(dir()).spawn().map(|_| Value::Null).map_err(|e| e.to_string())
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
    const WM_KISAYOL: u32 = WM_APP + 2; // wParam: new shortcut index (usize::MAX: none); returns 1 if it was taken
    const AC: i32 = 1;
    const CIK: i32 = 2;
    static PENCERE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
    static TASKBAR: AtomicU32 = AtomicU32::new(0);
    /// The user's choice first (data.json ayar.kisayol), then in this order; Windows 11 reserves Win+Shift+R for screen recording.
    const KISAYOLLAR: [(u32, char, &str); 4] = [
        (MOD_WIN | MOD_CONTROL, 'R', "Win+Ctrl+R"),
        (MOD_WIN | MOD_ALT, 'Y', "Win+Alt+Y"),
        (MOD_CONTROL | MOD_ALT | MOD_SHIFT, 'R', "Ctrl+Alt+Shift+R"),
        (MOD_CONTROL | MOD_ALT, ' ', "Ctrl+Alt+Space"),
    ];
    static ALINAN: AtomicUsize = AtomicUsize::new(usize::MAX);
    pub fn kisayol() -> Option<&'static str> {
        KISAYOLLAR.get(ALINAN.load(Relaxed)).map(|k| k.2)
    }
    pub fn adlar() -> Vec<&'static str> {
        KISAYOLLAR.iter().map(|k| k.2).collect()
    }
    /// From the server thread: the tray thread owns the hotkey, so it is changed there (SendMessage waits for the answer).
    pub fn degistir(i: usize) -> bool {
        let h = PENCERE.load(Relaxed);
        !h.is_null() && unsafe { SendMessageW(h, WM_KISAYOL, i, 0) } != 0
    }
    /// Replaces the global shortcut; if the new one is used by another app the old one is registered again.
    unsafe fn kisayol_ata(h: HWND, i: usize) -> bool {
        let kaydet = |i: usize| KISAYOLLAR.get(i).is_some_and(|&(m, k, _)| RegisterHotKey(h, 1, m | MOD_NOREPEAT, k as u32) != 0);
        let eski = ALINAN.load(Relaxed);
        UnregisterHotKey(h, 1);
        let ok = i == usize::MAX || kaydet(i);
        if ok {
            ALINAN.store(i, Relaxed);
        } else {
            kaydet(eski);
        }
        ipucu(h);
        ok
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

    fn ipucu_yaz(d: &mut NOTIFYICONDATAW) {
        let tip = w(&kisayol().map_or("RYOCS".into(), |k| format!("RYOCS ({k})")));
        d.szTip[..tip.len()].copy_from_slice(&tip);
    }

    unsafe fn ekle(h: HWND) {
        let mut d = veri(h);
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        d.uCallbackMessage = WM_TEPSI;
        d.hIcon = LoadImageW(GetModuleHandleW(null()), 1 as _, IMAGE_ICON, GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_DEFAULTCOLOR);
        ipucu_yaz(&mut d);
        Shell_NotifyIconW(NIM_ADD, &d);
    }

    /// The tooltip shows the current shortcut.
    unsafe fn ipucu(h: HWND) {
        let mut d = veri(h);
        d.uFlags = NIF_TIP;
        ipucu_yaz(&mut d);
        Shell_NotifyIconW(NIM_MODIFY, &d);
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
            WM_KISAYOL => return kisayol_ata(h, wp) as LRESULT,
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
            let tercih = super::read_json("data.json")["ayar"]["kisayol"].clone();
            if tercih != "yok" {
                let ilk = tercih.as_u64().map(|i| i as usize).filter(|&i| i < KISAYOLLAR.len());
                for i in ilk.into_iter().chain((0..KISAYOLLAR.len()).filter(|&i| Some(i) != ilk)) {
                    let (m, k, _) = KISAYOLLAR[i];
                    if RegisterHotKey(h, 1, m | MOD_NOREPEAT, k as u32) != 0 {
                        ALINAN.store(i, Relaxed);
                        break;
                    }
                }
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

/// Saved window rectangle (physical px) if its top edge is still on a screen (a monitor may have been removed).
fn pencere_yeri(v: &Value, ekranlar: &[(i32, i32, i32, i32)]) -> Option<(i32, i32, u32, u32)> {
    let g = |k: &str| v[k].as_i64().and_then(|x| i32::try_from(x).ok());
    let (x, y, w, h) = (g("x")?, g("y")?, g("w")?, g("h")?);
    if w < 200 || h < 200 {
        return None;
    }
    // at least 100 px of the title bar row must be on a screen, so the window can be grabbed
    ekranlar.iter().any(|&(ex, ey, ew, eh)| x + w - 100 > ex && x + 100 < ex + ew && y >= ey - 10 && y + 40 < ey + eh).then_some((x, y, w as u32, h as u32))
}

/// Remembers maximized or not, and the normal size and position (only when the window is in its normal state).
fn pencere_kaydet(w: &tao::window::Window) {
    let mut v = read_json("pencere.json");
    v["maks"] = json!(w.is_maximized());
    if !w.is_maximized() && !w.is_minimized() {
        if let Ok(p) = w.outer_position() {
            let s = w.inner_size();
            (v["x"], v["y"], v["w"], v["h"]) = (json!(p.x), json!(p.y), json!(s.width), json!(s.height));
        }
    }
    write_json("pencere.json", &v).ok();
}

fn main() {
    std::panic::set_hook(Box::new(|bilgi| {
        let yer = bilgi.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let is = thread::current().name().unwrap_or("?").to_string();
        hata_kaydet(&format!("PANIC [{is}] {yer}: {}\n{}", bilgi.payload_as_str().unwrap_or("?"), std::backtrace::Backtrace::force_capture()));
    }));
    let cfg = dir().join("config.json");
    if !cfg.exists() {
        fs::write(&cfg, CONFIG_SABLON.replace("Istanbul", varsayilan_sehir())).ok();
    }
    // If the port is taken the app is already running: bring the existing window to front instead of a second one.
    let server = match Server::http(ADDR) {
        Ok(s) => s,
        Err(_) if AGENT.post(&format!("{ORIGIN}/api/show")).send_string("").is_ok() => return,
        Err(e) => {
            // another program holds the port: say so instead of an empty window (the port is fixed: OAuth redirect addresses use it)
            hata_kaydet(&format!("port {ADDR} busy: {e}"));
            let m = tepsi::w(&format!("{}\n\n{ADDR}", l("RYOCS açılamadı: gereken ağ bağlantı noktasını başka bir program kullanıyor. O programı kapatıp RYOCS'u yeniden açın.", "RYOCS could not start: another program is using the network port it needs. Close that program and start RYOCS again.")));
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};
                MessageBoxW(std::ptr::null_mut(), m.as_ptr(), tepsi::w("RYOCS").as_ptr(), MB_ICONERROR);
            }
            return;
        }
    };

    // only the instance that owns the port encrypts older files: two at once would write the same temporary files (the write lock is per process)
    sifrele_goc();
    // installers left in %TEMP% by one-click updates: the update is done once RYOCS runs again (a file still in use goes next time)
    for e in fs::read_dir(std::env::temp_dir()).into_iter().flatten().flatten() {
        if e.file_name().to_str().is_some_and(|n| n.starts_with("RYOCS-Setup-") && n.ends_with(".exe")) {
            fs::remove_file(e.path()).ok();
        }
    }

    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::platform::windows::{IconExtWindows, WindowBuilderExtWindows};
    use wry::WebViewBuilderExtWindows;
    let event_loop = EventLoopBuilder::<Olay>::with_user_event().build();
    *kilit(&PROXY) = Some(event_loop.create_proxy());
    thread::spawn(move || {
        for req in server.incoming_requests() {
            thread::spawn(move || {
                // the panic hook writes details to hata.log; here it is enough that the thread dies
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handle(req))).ok();
            });
        }
    });
    // Same ID as the Start menu shortcut: taskbar grouping and notifications under the RYOCS name
    unsafe { windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(tepsi::w(AUMID).as_ptr()) };
    let gizli = std::env::args().any(|a| a == "--gizli"); // at Windows startup: start in the tray only
    tepsi::baslat();
    *kilit(&PANO) = pano_sabit_oku();
    thread::spawn(pano_izle);
    thread::spawn(ekran_izle);
    thread::spawn(|| {
        aumid_kaydet();
        thread::sleep(Duration::from_secs(3));
        olay(Olay::Goster(false)); // the window must not stay hidden even if the page could not say "ready"
    });
    // last size and position (pencere.json); maximized the first time or when the saved place is off every screen
    let yer = read_json("pencere.json");
    let ekranlar: Vec<_> = event_loop.available_monitors().map(|m| (m.position().x, m.position().y, m.size().width as i32, m.size().height as i32)).collect();
    let normal = pencere_yeri(&yer, &ekranlar);
    let mut wb = tao::window::WindowBuilder::new()
        .with_title("RYOCS")
        .with_visible(false) // shown once the UI is ready: no white flash and no theme jump
        .with_maximized(normal.is_none() || yer["maks"].as_bool() != Some(false))
        .with_window_icon(tao::window::Icon::from_resource(1, None).ok())
        .with_taskbar_icon(tao::window::Icon::from_resource(1, None).ok())
        .with_inner_size(tao::dpi::LogicalSize::new(1180.0, 720.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(1060.0, 680.0)) // cards do not fit in anything smaller
        .with_decorations(false) // the page draws the title bar (neumorphic / glass)
        .with_undecorated_shadow(true);
    if let Some((x, y, w, h)) = normal {
        wb = wb.with_position(tao::dpi::PhysicalPosition::new(x, y)).with_inner_size(tao::dpi::PhysicalSize::new(w, h));
    }
    let window = wb.build(&event_loop).expect("pencere");
    unsafe {
        // Windows 11: rounded corners for the borderless window too (no effect on Windows 10)
        use tao::platform::windows::WindowExtWindows;
        use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
        DwmSetWindowAttribute(window.hwnd() as _, DWMWA_WINDOW_CORNER_PREFERENCE as u32, &DWMWCP_ROUND as *const i32 as _, 4);
    }
    let webview = wry::WebViewBuilder::new()
        .with_initialization_script(format!("window.RYOCS_ANAHTAR={};", json!(*ANAHTAR)))
        .with_url(ORIGIN)
        .with_background_color((233, 234, 236, 255))
        .with_browser_accelerator_keys(false) // browser shortcuts such as F5, Ctrl+P, Ctrl+F do not belong to the app
        .with_hotkeys_zoom(false)
        // wry's defaults, plus: a hidden page (tray, minimized) keeps its 1 s timers, so reminders and focus rounds stay on time.
        // Without this Chromium wakes chained timers only once a minute after 5 minutes hidden.
        .with_additional_browser_args("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,IntensiveWakeUpThrottling --disable-background-timer-throttling")
        .with_ipc_handler(|r| match r.body().as_str() {
            "hazir" => olay(Olay::Goster(false)),
            "koyu" => olay(Olay::Koyu(true)),
            "acik" => olay(Olay::Koyu(false)),
            m if m.len() < 20 => olay(Olay::Pencere(m.into())),
            _ => {}
        })
        .build(&window);
    let webview = match webview {
        Ok(w) => w,
        Err(e) => {
            // usually the WebView2 Runtime is missing (some Windows 10 LTSC / N installs): say so instead of closing silently
            hata_kaydet(&format!("WebView2: {e}"));
            let m = tepsi::w(l("RYOCS açılamadı: Microsoft Edge WebView2 Runtime gerekli.\n\nTamam'a basınca indirme sayfası açılır; kurduktan sonra RYOCS'u yeniden açın.",
                               "RYOCS could not start: Microsoft Edge WebView2 Runtime is required.\n\nPress OK to open the download page; start RYOCS again after installing it."));
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};
                MessageBoxW(std::ptr::null_mut(), m.as_ptr(), tepsi::w("RYOCS").as_ptr(), MB_ICONERROR);
            }
            open_url("https://developer.microsoft.com/microsoft-edge/webview2/").ok();
            return;
        }
    };
    let mut kapaniyor = false;
    let mut kucuk = false; // minimized: the web view is hidden too
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        // Hidden window: WebView2 is told too (IsVisible), so it stops drawing and compositing; hiding only the window does not.
        let gizle = || {
            window.set_visible(false);
            webview.set_visible(false).ok();
        };
        let goster = || {
            webview.set_visible(true).ok(); // before the window: no frame with an empty web view
            window.set_visible(true);
            window.set_minimized(false);
            window.set_focus();
            webview.focus().ok(); // the web view was hidden: keyboard input goes to the page again
            webview.evaluate_script("window.gizlendi&&gizlendi(false)").ok(); // the page redraws and polls again
        };
        match event {
            // Close button: hides to the tray; reminders and the focus timer keep running.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                hata_kaydet("info: close request → hidden to the tray");
                pencere_kaydet(&window);
                gizle();
                webview.evaluate_script("kapandi()").ok();
            }
            // Real exit: flush the pending save, give the save request time, then exit.
            Event::UserEvent(Olay::Kapat) if !kapaniyor => {
                kapaniyor = true;
                webview.evaluate_script("kaydetSimdi()").ok();
                pencere_kaydet(&window);
                window.set_visible(false);
                thread::spawn(|| {
                    thread::sleep(Duration::from_millis(400)); // the UI's last save starts
                    ekran_kaydet();
                    kayitlari_bekle();
                    olay(Olay::Cik);
                });
            }
            Event::UserEvent(Olay::Cik) => {
                tepsi::kaldir();
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(Olay::Goster(one)) if !kapaniyor && (one || (!gizli && !window.is_visible())) => goster(),
            // started in the tray (--gizli): the page is told it is hidden, or it polls and animates until the window is opened once
            Event::UserEvent(Olay::Goster(false)) if gizli && !window.is_visible() => {
                webview.set_visible(false).ok();
                webview.evaluate_script("window.gizlendi&&gizlendi(true)").ok();
            }
            Event::UserEvent(Olay::Degistir) if !kapaniyor => {
                if window.is_visible() && window.is_focused() && !window.is_minimized() {
                    hata_kaydet("info: hidden by the shortcut");
                    gizle();
                    webview.evaluate_script("window.gizlendi&&gizlendi(true)").ok();
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
                        pencere_kaydet(&window);
                        gizle();
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
                // minimized ↔ restored: WebView2 does not stop drawing for a minimized window by itself
                if window.is_minimized() != kucuk {
                    kucuk = !kucuk;
                    webview.set_visible(!kucuk).ok();
                    if !kucuk {
                        webview.focus().ok();
                        webview.evaluate_script("window.gizlendi&&gizlendi(false)").ok(); // the page redraws and polls again
                    }
                }
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
    fn yazilar_gomulu() {
        let ui = include_str!("ui.html");
        for (ad, b) in YAZILAR {
            assert!(b.starts_with(b"wOF2") && ui.contains(&format!("/yazi/{ad}")), "{ad}");
        }
        assert_eq!(ui.matches("url(/yazi/").count(), YAZILAR.len());
    }

    #[test]
    fn iptal_edilen_jeton() {
        assert!(iptal_edildi("400: {\n  \"error\": \"invalid_grant\",\n  \"error_description\": \"Token has been expired or revoked.\"\n}"));
        assert!(!iptal_edildi("400: {\"error\": \"invalid_client\"}") && !iptal_edildi(AG_YOK) && !iptal_edildi("500: invalid_grant"));
    }

    #[test]
    fn api_anahtarsiz_kapali() {
        assert!(anahtar_gerekli("/api/state") && anahtar_gerekli("/api/uygulama") && anahtar_gerekli("/api/favicon"));
        assert!(!anahtar_gerekli("/") && !anahtar_gerekli("/callback/google") && !anahtar_gerekli("/api/show"));
        assert!(ANAHTAR.len() >= 40 && !include_str!("ui.html").contains(ANAHTAR.as_str()));
    }

    #[test]
    fn veri_yalniz_dosya_yoksa_bos() {
        use std::io::{Error, ErrorKind};
        assert_eq!(veri_coz(Err(Error::from(ErrorKind::NotFound))), Ok(json!({})));
        assert!(veri_coz(Err(Error::from(ErrorKind::PermissionDenied))).is_err()); // locked: never "empty"
        assert!(veri_coz(Ok("[1]".into())).is_err());
        assert_eq!(veri_coz(Ok("\u{feff}{\"a\":1}".into())), Ok(json!({"a": 1})));
    }

    #[test]
    fn favicon_ayiklama() {
        assert_eq!(ikon_href(r#"<head><link rel="stylesheet" href="a.css"><LINK REL="Shortcut Icon" HREF="/Img/Fav.png"></head>"#).as_deref(), Some("/Img/Fav.png"));
        assert_eq!(ikon_href("<link rel=icon href=/f.ico>").as_deref(), Some("/f.ico"));
        assert_eq!(ikon_href("<link rel='apple-touch-icon' href='t.png'/>").as_deref(), Some("t.png"));
        assert_eq!(ikon_href("<link rel=stylesheet href=x.css>"), None);
        assert_eq!(simge_turu("application/octet-stream", &[0, 0, 1, 0, 1]).as_deref(), Some("image/x-icon"));
        assert_eq!(simge_turu("text/html", b"<html>"), None);
        assert!(favicon("file:///C:/x").is_err() && favicon("javascript:alert(1)").is_err());
    }

    #[test]
    fn bolge_yalniz_ulke_kodu() {
        assert_eq!([bolge_kodu("TR"), bolge_kodu("001"), bolge_kodu("tr"), bolge_kodu("")], ["TR", "", "", ""]);
        assert!(bolge().is_empty() || bolge().len() == 2); // the real Windows setting
        assert!(include_str!("ui.html").matches("__BOLGE__").count() == 1); // exactly one placeholder is replaced
    }

    #[test]
    fn sistem_gercek_degerler() {
        let v = sistem();
        assert!(v["bellek"]["toplam"].as_u64().unwrap() > v["bellek"]["bos"].as_u64().unwrap());
        let d = &v["diskler"][0];
        assert!(d["toplam"].as_u64().unwrap() >= d["bos"].as_u64().unwrap());
        assert_eq!(d["ad"].as_str().map(String::from), std::env::var("SystemDrive").ok().map(|d| d.to_ascii_uppercase())); // system drive first
        assert!(v["diskler"].as_array().is_some_and(|d| d.len() <= 4) && v["acik"].as_u64().is_some_and(|s| s > 0));
        thread::sleep(Duration::from_millis(120));
        let c = sistem()["cpu"].as_f64().unwrap(); // second reading: a percent comes
        assert!((0.0..=100.0).contains(&c));
        // a reading a minute old is not used (one test: the CPU state is shared; skipped on a machine up for less than a minute)
        if let Some(eski) = std::time::Instant::now().checked_sub(Duration::from_secs(60)) {
            *kilit(&CPU_ONCEKI) = Some(((0, 0), eski));
            assert_eq!(sistem()["cpu"], Value::Null);
        }
        let _ = pano_oku(); // must return without crashing (whatever the content)
    }

    /// Tests that use files point APPDATA at their own folder: they run one at a time, holding this lock.
    static DOSYA_TESTI: Mutex<()> = Mutex::new(());
    fn test_klasoru(ad: &str) -> (std::sync::MutexGuard<'static, ()>, PathBuf) {
        let k = kilit(&DOSYA_TESTI);
        let kok = std::env::temp_dir().join(format!("ryocs-test-{ad}-{}", simdi_ms()));
        fs::create_dir_all(kok.join("RYOCS")).unwrap();
        unsafe { std::env::set_var("APPDATA", &kok) }; // dir() reads from here
        (k, kok)
    }

    #[test]
    fn ek_gmail_hesaplari() {
        let (_k, kok) = test_klasoru("posta");
        fs::write(kok.join("RYOCS").join("config.json"), r#"{"google":{"client_id":"x.apps.googleusercontent.com","client_secret":"gizli"}}"#).unwrap();
        assert_eq!(creds("gmail").unwrap(), creds("google").unwrap()); // the main Google client
        posta_kaydet("is@gmail.com", Some(json!({ "access_token": "a", "refresh_token": "r", "expires_in": 3600 })));
        assert_eq!(posta_token("is@gmail.com").unwrap(), "a"); // still valid: no refresh
        posta_kaydet("is@gmail.com", Some(json!({ "access_token": "b", "expires_in": 3600 })));
        assert_eq!(tokens_oku()["posta"]["is@gmail.com"]["refresh_token"], "r"); // a refresh without a new refresh token keeps the old one
        posta_kaydet("is@gmail.com", None); // revoked: listed, to reconnect
        assert_eq!(config_get()["posta"], json!([{ "eposta": "is@gmail.com", "durum": "bagli_degil" }]));
        assert_eq!(posta_token("is@gmail.com"), Err("bagli_degil".into()));
        // the main account's address survives a token refresh, a new sign-in drops it
        save_token("google", json!({ "access_token": "x", "refresh_token": "r", "expires_in": 3600 }));
        let mut all = tokens_oku(); all["google"]["eposta"] = json!("ben@gmail.com"); tokens_yaz(&all).unwrap();
        save_token("google", json!({ "access_token": "y", "expires_in": 3600 }));
        assert_eq!(tokens_oku()["google"]["eposta"], "ben@gmail.com");
        save_token("google", json!({ "access_token": "z", "refresh_token": "r2", "expires_in": 3600 }));
        assert!(tokens_oku()["google"]["eposta"].is_null());
        posta_sil(r#"{"eposta":"is@gmail.com"}"#).unwrap();
        assert_eq!(config_get()["posta"], json!([]));
        assert!(fs::read_to_string(kok.join("RYOCS").join("tokens.json")).unwrap().starts_with(DPAPI_ONEK));
        fs::remove_dir_all(&kok).ok();
    }

    #[test]
    fn pano_sabitleri_kalici() {
        let (_k, kok) = test_klasoru("pano");
        let mut l = Vec::new();
        for (i, m) in ["bir", "iki", "üç"].iter().enumerate() { pano_ekle(&mut l, m.to_string(), i as u64); }
        l.iter_mut().find(|x| x.m == "iki").unwrap().sabit = true;
        pano_sabit_kaydet(&l);
        let d = fs::read_to_string(kok.join("RYOCS").join("pano.json")).unwrap();
        assert!(d.starts_with(DPAPI_ONEK) && !d.contains("iki")); // encrypted, like the other files with personal text
        let o = pano_sabit_oku();
        assert_eq!(o.iter().map(|x| (x.m.as_str(), x.sabit)).collect::<Vec<_>>(), [("iki", true)]); // only the pinned one comes back
        pano_sabit_kaydet(&[]);
        assert!(pano_sabit_oku().is_empty());
        fs::remove_dir_all(&kok).ok();
    }

    #[test]
    fn ekran_dosyasi_acilmazsa_saklanir() {
        let (_k, kok) = test_klasoru("ekran");
        let r = kok.join("RYOCS");
        assert_eq!(ekran_yukle(), json!({})); // no file: empty, nothing renamed
        write_json("ekran.json", &json!({ "g": { "2026-11-01": { "A": 5 } } })).unwrap();
        assert_eq!(ekran_yukle()["g"]["2026-11-01"]["A"], 5);
        fs::write(r.join("ekran.json"), format!("{DPAPI_ONEK}AAAA")).unwrap(); // encrypted elsewhere
        assert_eq!(ekran_yukle(), json!({}));
        assert!(!r.join("ekran.json").exists() && r.join("ekran.bozuk.json").exists());
        fs::remove_file(r.join("ekran.bozuk.json")).unwrap();
        fs::create_dir(r.join("ekran.json")).unwrap(); // unreadable (a read error other than "not found"): kept aside too
        assert_eq!(ekran_yukle(), json!({}));
        assert!(!r.join("ekran.json").exists());
        fs::remove_dir_all(&kok).ok();
    }

    #[test]
    fn eski_dosyalar_sifrelenir() {
        let (_k, kok) = test_klasoru("goc");
        let r = kok.join("RYOCS");
        fs::create_dir_all(r.join("yedekler")).unwrap();
        fs::write(r.join("data.json"), "\u{feff}{\"todos\":[{\"t\":\"süt\"}]}").unwrap();
        fs::write(r.join("yedekler").join("data-2026-01-01.json"), r#"{"a":1}"#).unwrap();
        fs::write(r.join("yedekler").join("data-2026-01-02.json"), "bozuk").unwrap();
        fs::write(r.join("config.json"), r#"{"google":{"client_id":"x.apps.googleusercontent.com","client_secret":"gizli1"},"spotify":{"client_id":"","client_secret":""}}"#).unwrap();
        use base64::Engine;
        let eski_belirtec = base64::engine::general_purpose::STANDARD.encode(dpapi(br#"{"google":{"refresh_token":"r"}}"#, true).unwrap());
        fs::write(r.join("tokens.json"), eski_belirtec).unwrap(); // 1.0.7 format: no prefix
        sifrele_goc();
        let oku = |p: &str| fs::read_to_string(r.join(p)).unwrap();
        assert!(oku("data.json").starts_with(DPAPI_ONEK) && oku("yedekler/data-2026-01-01.json").starts_with(DPAPI_ONEK) && oku("tokens.json").starts_with(DPAPI_ONEK));
        assert_eq!(veri_oku().unwrap()["todos"][0]["t"], "süt");
        assert_eq!(oku("yedekler/data-2026-01-02.json"), "bozuk"); // damaged: left alone
        assert_eq!(tokens_oku()["google"]["refresh_token"], "r");
        assert!(oku("config.json").contains(DPAPI_ONEK) && !oku("config.json").contains("gizli1"));
        assert_eq!(creds("google").unwrap().1, "gizli1");
        assert_eq!(creds("spotify"), Err("kimlik_yok".into()));
        let once = oku("data.json");
        sifrele_goc(); // second start: nothing to do
        assert_eq!(oku("data.json"), once);
        fs::remove_dir_all(&kok).ok();
    }

    #[test]
    fn esitleme_uctan_uca() {
        let (_k, kok) = test_klasoru("esitle");
        let (od, yerel) = (kok.join("od"), kok.join("RYOCS").join("data.json"));
        fs::create_dir_all(&od).unwrap();
        let uzak = od.join("RYOCS").join("data.json");
        let d = |b: &str| esitle(b).unwrap()["durum"].as_str().unwrap().to_string();
        let duz = || dpapi_coz(&fs::read_to_string(&yerel).unwrap()).unwrap();
        write_json("data.json", &json!({"a": 1})).unwrap();
        assert!(fs::read_to_string(&yerel).unwrap().starts_with(DPAPI_ONEK)); // encrypted on disk
        assert_eq!(d("{}"), "kapali");
        assert!(esitle(&json!({"acik": true, "klasor": kok.join("yok")}).to_string()).is_err());
        assert_eq!(d(&json!({"acik": true, "klasor": od}).to_string()), "ok"); // first start: no remote, send
        assert_eq!(fs::read_to_string(&uzak).unwrap(), duz()); // no password: plain copy as before
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
        assert_eq!(fs::read_to_string(&uzak).unwrap(), duz());
        assert_eq!(dpapi_coz(&fs::read_to_string(dir().join("yedekler").join("data-onedrive.json")).unwrap()).unwrap(), r#"{"a":4}"#);
        // password: the copy is encrypted, nothing changes for the sync decision, the other PC needs the same password
        assert!(esitle(r#"{"parola":"kisa"}"#).is_err());
        assert_eq!(d(r#"{"parola":"uzun-parola-1"}"#), "ok");
        assert!(fs::read_to_string(&uzak).unwrap().starts_with(PAROLA_ONEK));
        assert_eq!(d("{}"), "ok"); // same plain text: no conflict
        assert_eq!(d(r#"{"parola":"yeni-parola-22"}"#), "ok"); // change: the copy is written again with the new one
        assert!(parola_coz("yeni-parola-22", &fs::read_to_string(&uzak).unwrap()).is_some());
        let mut e = read_json("esitle.json");
        e.as_object_mut().unwrap().remove("parola"); // like a second PC that has no password yet
        write_json("esitle.json", &e).unwrap();
        assert_eq!(d("{}"), "parola"); // no password on this PC: neither sends nor pulls
        assert!(esitle(r#"{"parola":""}"#).is_err()); // cannot remove what it cannot open
        assert!(esitle(r#"{"parola":"yanlis-parola"}"#).is_err()); // does not open the copy: refused
        assert_eq!(d(r#"{"parola":"yeni-parola-22"}"#), "ok");
        let v = esitle(r#"{"parola":""}"#).unwrap(); // removed: the copy is plain again and sync goes on
        assert_eq!((v["durum"].as_str(), v["parola"].as_bool()), (Some("ok"), Some(false)));
        assert_eq!(fs::read_to_string(&uzak).unwrap(), duz());
        assert_eq!(d(r#"{"parola":"uzun-parola-1"}"#), "ok");
        fs::write(&uzak, "bozuk").unwrap(); // remote file corrupt: local is kept, an error returns
        assert!(esitle(r#"{"baslangic":true}"#).is_err());
        assert_eq!(veri_oku().unwrap()["a"], 3);
        fs::remove_dir_all(&kok).ok();
    }

    #[test]
    fn kapanis_kayitlari_bekler() {
        let k = KayitSuruyor::basla();
        let t = thread::spawn(|| { let b = simdi_ms(); kayitlari_bekle(); simdi_ms() - b });
        thread::sleep(Duration::from_millis(300));
        drop(k); // the save ends; quitting still waits 400 ms for a queued one
        let gecen = t.join().unwrap();
        assert!((650..3000).contains(&gecen), "{gecen} ms");
        thread::sleep(Duration::from_millis(450));
        let b = simdi_ms();
        kayitlari_bekle(); // nothing saving: returns at once
        assert!(simdi_ms() - b < 200);
    }

    #[test]
    fn spotify_degerleri() {
        assert_eq!(spotify_deger("seek", r#"{"ms":61000}"#), Ok("61000".into()));
        assert!(spotify_deger("seek", r#"{"ms":-1}"#).is_err() && spotify_deger("volume", r#"{"v":101}"#).is_err());
        assert_eq!(spotify_deger("shuffle", r#"{"on":true}"#), Ok("true".into()));
        assert_eq!(spotify_deger("repeat", r#"{"mod":"track"}"#), Ok("track".into()));
        assert!(spotify_deger("repeat", r#"{"mod":"all&x=1"}"#).is_err());
        assert!(spotify_deger("transfer", r#"{"id":"a1b2"}"#).is_ok() && spotify_deger("transfer", r#"{"id":"../x"}"#).is_err());
    }

    #[test]
    fn dun_ayirma() {
        let saatler: Vec<String> = (0..48).map(|h| format!("2026-11-{}T{:02}:00", 20 + h / 24, h % 24)).collect();
        let mut w = json!({"current": {"time": "2026-11-21T14:15"},
            "hourly": {"time": saatler, "temperature_2m": (0..48).collect::<Vec<_>>()},
            "daily": {"time": ["2026-11-20", "2026-11-21"], "temperature_2m_max": [9, 12]}, "daily_units": {"time": "iso8601"}});
        dun_ayir(&mut w);
        assert_eq!(w["dun"], json!(14)); // yesterday 14:00
        assert_eq!((w["hourly"]["time"][0].as_str(), w["hourly"]["temperature_2m"].as_array().map(|a| a.len())), (Some("2026-11-21T00:00"), Some(24)));
        assert_eq!(w["daily"], json!({"time": ["2026-11-21"], "temperature_2m_max": [12]}));
        let mut eski = json!({"current": {"time": "2026-11-21T01:00"}, "hourly": {"time": ["2026-11-21T01:00"]}});
        dun_ayir(&mut eski); // no yesterday in the data: no "dun", nothing breaks
        assert_eq!(eski["dun"], Value::Null);
    }

    #[test]
    fn mail_onbellegi() {
        let mut o = HashMap::new();
        let ids = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let r = mail_birlestir(&mut o, vec![json!({"id": "a"}), json!({"id": "b"})], &ids(&["b", "a"]));
        assert_eq!(r, vec![json!({"id": "b"}), json!({"id": "a"})]); // Gmail's order, not the fetch order
        let r = mail_birlestir(&mut o, vec![json!({"id": "c"})], &ids(&["c", "a"])); // b was read: gone from memory too
        assert_eq!(r, vec![json!({"id": "c"}), json!({"id": "a"})]);
        assert!(mail_id("18c2ab9F") && !mail_id("") && !mail_id("a/../b") && !mail_id(&"a".repeat(65)));
        assert!(mail_islem(r#"{"id":"../labels"}"#).is_err_and(|e| e == "geçersiz id")); // refused before any request
        assert!(mail_islem(r#"{"id":"abc","islem":"sil"}"#).is_err_and(|e| e == "geçersiz işlem")); // nothing else, deleting least of all
        assert_eq!(o.len(), 2);
    }

    #[test]
    fn sifreli_dosyalar() {
        let s = dpapi_sar(r#"{"not":"gizli"}"#).unwrap();
        assert!(s.starts_with(DPAPI_ONEK) && !s.contains("gizli"));
        assert_eq!(dpapi_coz(&s).unwrap(), r#"{"not":"gizli"}"#);
        assert_eq!(dpapi_coz("\u{feff}{\"a\":1}\n").unwrap(), r#"{"a":1}"#); // plain file of an older version
        assert!(dpapi_coz(&format!("{DPAPI_ONEK}AAAA")).is_err()); // another account's file: an error, never empty data
        assert!(veri_coz(Ok(format!("{DPAPI_ONEK}AAAA"))).is_err());
        assert_eq!(veri_coz(Ok(s)).unwrap()["not"], "gizli");
        assert!(sifreli_dosya("data.json") && sifreli_dosya("yedekler/data-2026-01-01.json") && !sifreli_dosya("config.json") && !sifreli_dosya("esitle.json"));
        let p = parola_sifrele("doğru parola", "{\"ş\":1}").unwrap();
        assert!(p.starts_with(PAROLA_ONEK) && p != parola_sifrele("doğru parola", "{\"ş\":1}").unwrap()); // new nonce each time
        assert_eq!(parola_coz("doğru parola", &p).as_deref(), Some("{\"ş\":1}"));
        assert!(parola_coz("yanlış", &p).is_none());
        let mut b = p.clone().into_bytes();
        let n = b.len() - 3;
        b[n] = if b[n] == b'A' { b'B' } else { b'A' }; // a changed byte fails the GCM tag
        assert!(parola_coz("doğru parola", &String::from_utf8(b).unwrap()).is_none());
    }

    #[test]
    fn dpapi_gidis_donus() {
        let s = dpapi(br#"{"google":{"refresh_token":"x"}}"#, true).unwrap();
        assert!(!s.windows(13).any(|w| w == b"refresh_token")); // not plain text on disk
        assert_eq!(dpapi(&s, false).unwrap(), br#"{"google":{"refresh_token":"x"}}"#);
        assert!(dpapi(b"bozuk", false).is_none()); // a damaged file means "not connected", not a crash
    }

    #[test]
    fn cpu_iki_olcum_farkindan() {
        assert_eq!(cpu_yuzde(None, (10, 100)), None); // ilk okuma
        assert_eq!(cpu_yuzde(Some((10, 100)), (10, 100)), None); // time has not passed
        assert_eq!(cpu_yuzde(Some((0, 0)), (75, 100)), Some(25.0));
        assert_eq!(cpu_yuzde(Some((50, 100)), (40, 200)), None); // the counter went backwards
    }

    #[test]
    fn ekran_toplama() {
        let mut v = json!({});
        for p in ["msedge", "msedge", "Code"] { ekran_ekle(&mut v, "2026-11-01", p, 5); }
        assert_eq!(v["g"]["2026-11-01"], json!({ "msedge": 10, "Code": 5 }));
        for d in 2..=31 { ekran_ekle(&mut v, &format!("2026-11-{d:02}"), "x", 1); }
        let g = v["g"].as_object().unwrap();
        assert_eq!((g.len(), g.contains_key("2026-11-01"), g.contains_key("2026-11-31")), (30, false, true)); // the oldest day goes
        // a broken shape is replaced, not a panic
        let mut b = json!({ "g": { "2026-11-01": 5 } });
        ekran_ekle(&mut b, "2026-11-01", "a", 5);
        let mut c = json!([1]);
        ekran_ekle(&mut c, "2026-11-01", "a", 5);
        assert_eq!((b["g"]["2026-11-01"]["a"].as_u64(), sifreli_dosya("ekran.json")), (Some(5), true));
        // readable names from version info
        let t = ["Steam Client WebHelper", "Battle.net Helper", "Microsoft® Windows® Operating System", "Adobe® Photoshop™", "Helper", "Windows Explorer", "Electron"].map(ad_temizle);
        assert_eq!(t, ["Steam", "Battle.net", "", "Adobe Photoshop", "Helper", "Windows Explorer", ""]);
        let not = std::env::var("SystemRoot").unwrap_or("C:\\Windows".into()) + "\\explorer.exe";
        assert!(!program_adi(&not).eq_ignore_ascii_case("explorer"), "explorer.exe has a description"); // reads the real version resource
        assert_eq!(program_adi("C:\\yok\\benim-oyunum.exe"), "Benim-oyunum"); // no file: the file name
        // Spotify gives its string lengths in bytes: only checked where Spotify is installed
        let sp = std::env::var("APPDATA").unwrap_or_default() + "\\Spotify\\Spotify.exe";
        if std::path::Path::new(&sp).exists() { assert_eq!(program_adi(&sp), "Spotify"); }
    }

    #[test]
    fn pano_son_yirmi_ve_tekrarsiz() {
        let mut l = Vec::new();
        for i in 0..25 { pano_ekle(&mut l, format!("m{i}"), i); }
        assert_eq!((l.len(), l[0].m.as_str(), l[19].m.as_str()), (20, "m24", "m5"));
        assert!(pano_ekle(&mut l, "m10".into(), 99)); // a re-copied item moves to the top, no duplicates
        assert_eq!((l.len(), l[0].m.as_str(), l[0].t), (20, "m10", 99));
        assert!(!pano_ekle(&mut l, "   ".into(), 1) && !pano_ekle(&mut l, "x".repeat(5001), 1));
        assert_eq!(l[0].m, "m10");
        // pinned: stays on top, never drops off, keeps its pin when copied again
        l[5].sabit = true;
        let s = l.remove(5);
        l.insert(0, s);
        for i in 0..30 { pano_ekle(&mut l, format!("y{i}"), 200 + i); }
        assert_eq!((l.len(), l[0].sabit, l[1].m.as_str()), (21, true, "y29"));
        let m = l[0].m.clone();
        pano_ekle(&mut l, m.clone(), 300);
        assert_eq!((l[0].m.as_str(), l[0].sabit), (m.as_str(), true));
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
    fn son_dakika_kaynaklari() {
        // every address is https and fixed; one failing feed does not stop the others; all failing gives the error
        assert!(SON_DAKIKA.iter().all(|(_, u)| u.starts_with("https://")));
        let r = son_dakika(|u| if u.contains("sabah") { Err("500".into()) } else { Ok(format!("<rss>{u}</rss>")) }).unwrap();
        let l = r.as_array().unwrap();
        assert_eq!(l.len(), 6);
        assert!(l.iter().any(|x| x["k"] == "TRT Haber" && x["x"].as_str().unwrap().contains("trthaber")) && !l.iter().any(|x| x["k"] == "Sabah"));
        assert_eq!(son_dakika(|_| Err(AG_YOK.into())), Err(AG_YOK.into()));
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
        assert!(yedek_adi(&format!("data-{}.json", bugun())));
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
    fn pencere_yeri_ekranda_olmali() {
        let ekran = [(0, 0, 1920, 1080)];
        let v = |x: i32, y: i32| json!({"x": x, "y": y, "w": 1200, "h": 800, "maks": false});
        assert_eq!(pencere_yeri(&v(100, 50), &ekran), Some((100, 50, 1200, 800)));
        assert_eq!(pencere_yeri(&v(2500, 50), &ekran), None); // the second monitor was removed
        assert_eq!(pencere_yeri(&v(100, -400), &ekran), None); // title bar above the screen
        assert_eq!(pencere_yeri(&v(-1150, 50), &ekran), None); // only 50 px visible
        assert!(pencere_yeri(&v(2500, 50), &[(0, 0, 1920, 1080), (1920, 0, 2560, 1440)]).is_some());
        assert_eq!(pencere_yeri(&json!({}), &ekran), None);
        assert_eq!(pencere_yeri(&json!({"x": 0, "y": 0, "w": 50, "h": 50}), &ekran), None);
    }

    #[test]
    fn etkinlik_duzenleme_dogrulama() {
        // rejected before any network call
        assert!(calendar_edit(r#"{"id":"a/b","summary":"x"}"#).is_err());
        assert!(calendar_edit(r#"{"id":"abc","takvim":"x y","summary":"x"}"#).is_err());
        assert!(calendar_edit(r#"{"id":"abc","summary":"   "}"#).is_err());
        assert!(calendar_edit(r#"{"id":"abc"}"#).is_err());
        assert!(calendar_edit(r#"{"id":"abc","start":{"date":"2026-01-01"}}"#).is_err()); // start without end
    }

    #[test]
    fn surum_karsilastirma() {
        assert_eq!(surum_sayi("v0.3.10"), Some((0, 3, 10)));
        assert!(surum_sayi("v0.3.10") > surum_sayi("0.3.9"));
        assert!(surum_sayi("v1.0.0") > surum_sayi("0.9.9"));
        assert_eq!(surum_sayi("v1.2"), None);
        let h = "ab".repeat(32);
        let t = format!("{}  RYOCS-Setup-1.2.0.exe\n{h}  RYOCS-Setup-1.1.0.exe\r\nxyz  RYOCS-Setup-1.3.0.exe\n", "CD".repeat(32));
        assert_eq!(sha_satiri(&t, "RYOCS-Setup-1.2.0.exe"), Some("cd".repeat(32)));
        assert_eq!(sha_satiri(&t, "RYOCS-Setup-1.1.0.exe"), Some(h));
        assert_eq!(sha_satiri(&t, "RYOCS-Setup-1.3.0.exe"), None); // not a hash
        assert_eq!(sha_satiri(&t, "RYOCS-Setup-1.0.0.exe"), None);
        assert_eq!(hex(&[0, 255, 16]), "00ff10");
        assert_eq!(surum_sayi("v1.2.3.4"), None);
        assert_eq!(surum_sayi("v1.2.0-beta"), None);
    }

    #[test]
    fn yalnizca_web_adresi_acilir() {
        assert!(open_url("file:///C:/Windows/notepad.exe").is_err());
        assert!(open_url("calc").is_err());
    }
}
