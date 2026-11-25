# Aynı adlı şehirler — uygulama planı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** RYOCS, sadece adla kaydedilmiş bir şehir için en olası yeri kendisi seçsin, seçtiği yerin bölgesini hava kartında göstersin ve açılış sorusunun neden çıkmadığını `hata.log`'a yazsın.

**Architecture:**
- Rust, ham geocoding sonuçlarını `yer()` ile kısaltmadan önce sıralar.
- UI tarafında iki değişiklik var: kart başlığına bölge eklenir ve `konumDenetle` adımları günlüğe yazılır.

**Tech Stack:** Rust (serde_json, ureq) ve `src/ui.html` (vanilla JS). Testler `cargo test` ve `python tests/ui/calis.py`.

**Spec:** `docs/superpowers/specs/2026-11-25-sehir-karismasi-design.md`

## Global Constraints
- Yeni crate eklenmez. Kullanıcının konumu ve koordinatları hiçbir dışa giden isteğe ya da günlüğe yazılmaz.
- Eski data.json / config.json her zaman açılmalı. Koordinatsız (`sehir` dizesi) kayıt çalışmaya devam eder.
- Kodun içindeki açıklamalar İngilizce, kullanıcının gördüğü metin TR/EN (`L()` / `l()`).
- Doğrulama: `cargo clippy --release --offline -- -D warnings`, `cargo test --release --offline`, `python tests/ui/calis.py`.
- RYOCS kabuktan başlatılmaz. Exe derlemeden önce kullanıcı RYOCS'tan çıkar.

## Review Focus
- Büyük harfle yazılmış ad (`KÖYCEĞİZ`) ya da `İ` ile başlayan ad (`İdil`) da ilçe adıyla eşleşmeli. Bunu Görev 1'in testi kapsıyor.
- `admin2` alanı olmayan sonuç (il merkezi, yurt dışı) çökmemeli, sıralamada eşleşmeyenler arasına girmeli. Bunu Görev 1'in testi kapsıyor.
- Bölgesi boş yerde başlıkta artık ` · ` kalmamalı. Bunu Görev 2'nin testi kapsıyor.
- Hava kartı her saniye yeniden çiziliyor; başlık her seferinde yeniden yazılmamalı (mevcut `textContent` karşılaştırması korunur). Görev 2.
- Günlük satırları koordinat içermemeli. Bunu Görev 3'ün testi kapsıyor.

---

### Görev 1: Rust — en olası yer başta

**Files:**
- Modify: `src/main.rs:411-455` (`konum`, `sehir_ara`)
- Test: `src/main.rs` `mod tests`

**Interfaces:**
- Produces: `fn sirala(r: &mut [Value], ad: &str)`. Ham Open-Meteo sonuçlarını (name / admin2 / population alanları) yerinde, kararlı bir sıralamayla dizer.

- [ ] **Adım 1: Başarısız testi yaz** (`fn sirala_ilce_merkezi_basta`)

```rust
let mut r = vec![
    json!({"name": "Köyceğiz", "admin1": "Şırnak", "admin2": "İdil İlçesi", "population": 714}),
    json!({"name": "Köyceğiz", "admin1": "Amasya", "admin2": "Amasya", "population": 122}),
    json!({"name": "Köyceğiz", "admin1": "Muğla", "admin2": "Köyceğiz İlçesi"}),
];
sirala(&mut r, "KÖYCEĞİZ");
assert_eq!(r[0]["admin1"], "Muğla");
assert_eq!(r[1]["admin1"], "Şırnak"); // then population
let mut r = vec![json!({"name": "İdil"}), json!({"name": "İdil", "admin2": "İdil District"})];
sirala(&mut r, "idil");
assert_eq!(r[0]["admin2"], "İdil District");
```

- [ ] **Adım 2:** `cargo test --release --offline sirala` → derlenmez (`sirala` yok).
- [ ] **Adım 3: `sirala` fonksiyonunu yaz.**
  - Türkçe küçük harf için yardımcı: `İ`→`i`, `I`→`ı`, sonra `to_lowercase()`.
  - İlçe adı, `admin2`'den " İlçesi" / " District" eki atılarak bulunur.
  - Anahtar: `(eşleşmiyor: bool, -population)` ile `sort_by_key`. Kararlı olduğu için eşitlikte API sırası korunur.
- [ ] **Adım 4: `sehir_ara` ve `konum`'u bağla.**
  - `sehir_ara`: sonuçlar `yer` ile kısaltılmadan önce `sirala` edilir.
  - `konum`: `count` 8 olur; sonuçlar `sirala` edilir, sonra ilk eleman alınır.
- [ ] **Adım 5:** `cargo test --release --offline` ve clippy → PASS.
- [ ] **Adım 6: Commit:** "Pick the district seat first among same-named places"

### Görev 2: Kart başlığında bölge

**Files:**
- Modify: `src/ui.html:1593` (`.hvAd` doldurma) ve `.hvAd` CSS'i
- Test: `tests/ui/hava.test.js`

**Interfaces:**
- Consumes: hava yanıtındaki `w.bolge` (Rust zaten `main.rs:494`'te ekliyor).

- [ ] **Adım 1: Testi yaz.**
  - Demo verisinde `bolge` "Muğla". Başlıkta ad ve bölge ayrı görünmeli:

    ```js
    esit('region in title', [q('.hvAd').firstChild.textContent, q('.hvAd small')?.textContent], ['Köyceğiz', 'Muğla']);
    ```
  - `bolge:''` ile yeniden çizildikten sonra: `esit('no region', q('.hvAd small'), null)`.
  - Başlıkta `title` olarak "Köyceğiz, Muğla" durmalı.
- [ ] **Adım 2:** `python tests/ui/calis.py hava` → FAIL.
- [ ] **Adım 3: Başlığı yaz.**
  - `.hvAd` içine `esc(ad)` ve, bölge varsa, `<small>${esc(bolge)}</small>` yazılır.
  - Karşılaştırma anahtarı `ad|bolge` olur ve `ad.dataset` içinde tutulur; başlık yalnızca anahtar değişince yeniden yazılır.
  - CSS: `small` soluk renkte (`var(--mu)`), önünde `· `. `.hvAd` tek satır: `white-space:nowrap; overflow:hidden; text-overflow:ellipsis; min-width:0`.
- [ ] **Adım 4:** `python tests/ui/calis.py hava hava2` → PASS.
- [ ] **Adım 5: Commit:** "Show the region next to the weather city"

### Görev 3: `konumDenetle` tanı günlüğü

**Files:**
- Modify: `src/ui.html:1545-1549` (`konumDenetle`)
- Test: `tests/ui/hava.test.js` (mevcut `konumDenetle` testlerinin yanına)

**Interfaces:**
- Consumes: `POST /api/log {mesaj}`. Rust bunu `ui: <mesaj>` olarak `hata.log`'a yazıyor.
- Produces: `konumLog(m: string)`, `api('/api/log',{mesaj:'konum: '+m})`. `hataGonder`'in 20 satır sınırından ayrıdır.

- [ ] **Adım 1: Testi yaz.** Fetch stub'ı `/api/log` gövdelerini bir diziye toplar.
  - **Adla kayıtlı Yeşilova senaryosu:** mesajlar sırayla tam olarak şunlar olmalı:
    - `konum: başladı`
    - `konum: config konum=false`
    - `konum: seçildi Yeşilova, Burdur`
    - `konum: kaydedildi ok`

    Hiçbir mesaj `37.5` içermemeli.
  - **`konum:true` senaryosu:** yalnızca `konum: başladı` ve `konum: config konum=true` yazılır; seçici açılmaz.
  - **Kilitli senaryo:** yalnızca `konum: kilitli` yazılır.
- [ ] **Adım 2:** `python tests/ui/calis.py hava` → FAIL.
- [ ] **Adım 3: Günlük satırlarını ekle.**
  - `konumDenetle` içindeki her dönüş noktası ve `yerSec` sonucu yukarıdaki satırlardan birini yazar.
  - Hata ya da iptal durumunda: `hata: <metin>`, `iptal`.
  - `yerSec`'in kendisi değişmez. Bulunan yer sayısı `konumDenetle`'de ayrı bir `sehir_ara` çağrısıyla alınmaz; seçilen yer de günlüğe ad + bölge olarak yazılır.
- [ ] **Adım 4:** `python tests/ui/calis.py` (hepsi) ve clippy → PASS.
- [ ] **Adım 5: Changelog ve commit.**
  - `CHANGELOG.md` "Unreleased" bölümüne tek satır: aynı adlı yerlerde ilçe merkezi seçilir, bölge kartta görünür.
  - Daha önce commit edilmemiş açılış değişikliği (`yukle().then(()=>{konumDenetle();…})`) de bu commit'e girer.
  - Commit mesajı: "Log the city check at startup; ask it without waiting for the cards". Ardından push.

### Son: exe
- [ ] Kullanıcı "çıktım" dedikten sonra `cargo build --release --offline`.
- [ ] Kullanıcı açtıktan sonra iki şeye bakılır:
  - Kartta `Köyceğiz · Muğla` yazıyor mu?
  - `hata.log`'daki `ui: konum:` satırları ne diyor?
