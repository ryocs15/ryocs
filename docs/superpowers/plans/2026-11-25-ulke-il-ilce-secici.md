# Ülke → il → ilçe seçici — uygulama planı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Hava durumu şehri (ana şehir ve ek şehirler) ülke → il → ilçe listelerinden seçilebilsin.

**Architecture:**
- Ülke, il ve ilçe listeleri GeoNames'ten üretilen bir TSV dosyasında tutulur ve exe'ye gömülür.
- Koordinat, seçim yapıldığında Open-Meteo'nun `/v1/get?id=<geonameid>` uç noktasından alınır.
- Kayıt bugünkü `{ad,bolge,lat,lon}` biçiminde yapılır.

**Tech Stack:** Python (veri aracı), Rust (`include_str!`, mevcut `AGENT`/`yer()`), `src/ui.html`.

**Spec:** `docs/superpowers/specs/2026-11-25-ulke-il-ilce-secici-design.md`

## Spec'ten sapma (veriye bakınca verdiğim karar)
**Durum:** `cities500` dosyasında 47.752 ilçenin yalnızca 19.312'sinin merkezi var. Muğla'daki Köyceğiz de bunlardan değil: nüfusu girilmediği için dosyaya alınmamış. Öte yandan Open-Meteo, ilçenin GeoNames kimliğiyle sorgulanınca koordinatı döndürüyor (8631961 → 36.978, 28.724).

**Karar:**
- Listeler uygulamanın içinde kalır.
- Koordinat seçim anında bu tek istekle alınır. İstekte yalnızca herkese açık ilçe kimliği gider; hava durumu isteği zaten bu servise konumu gönderiyor.
- `cities500` kullanılmaz.

## Global Constraints
- Yeni crate eklenmez. Kodun içindeki açıklamalar İngilizce, kullanıcının gördüğü metin TR/EN.
- Eski config/data her zaman açılmalı. Bugünkü yazarak arama aynen kalır.
- GeoNames CC BY 4.0: README'ye ve Hakkında'ya "Yer verisi: GeoNames (CC BY 4.0)" atfı.
- Doğrulama: `cargo clippy --release --offline -- -D warnings`, `cargo test --release --offline`, `python tests/ui/calis.py`.
- Exe derlemeden önce kullanıcı RYOCS'tan çıkar. RYOCS kabuktan başlatılmaz.

## Review Focus
- İlçe düzeyi olmayan ülke (ör. `MC`, `SG`): il listesi boş ya da ilçe listesi boş olsa bile seçim kaydedilebilmeli. Görev 2 ve 3.
- Ağ yokken koordinat alınamazsa kayıt yapılmamalı, kullanıcıya tost ile hata gösterilmeli. Görev 3'ün testi kapsıyor.
- `?id=` parametresine sayı dışında bir değer verilirse Rust hata döndürmeli, dışarıya istek atmamalı. Görev 2.
- İlçe adındaki " İlçesi" eki listede ve kayıtta görünmemeli (`Köyceğiz`, `Köyceğiz İlçesi` değil). Görev 1 ve 2.
- Ek şehirler 4'ü geçmemeli, aynı yer iki kez eklenmemeli, ana şehir ek şehir olarak eklenmemeli. Görev 3.

---

### Görev 1: Veri aracı ve `data/yerler.tsv`

**Files:**
- Create: `tools/yerler.py`
- Create: `data/yerler.tsv`
- Modify: `README.md` (atıf)

**Interfaces:**
- Produces: `data/yerler.tsv`. UTF-8, başlık satırı yok. Her satır şu sekmeyle ayrılmış alanlardan oluşur: `ülke` `il` `ilçe` `geonameid` `ad`.
  - **İl satırı:** `ilçe` alanı boş.
  - **İlçe satırı:** `il` ve `ilçe` alanlarının ikisi de dolu.
  - **Ad:** Sonundaki " İlçesi" / " District" eki kırpılmış hali.
  - **Ülke:** ISO-2 kodu.

- [ ] **Adım 1: Aracı yaz.**
  - Kullanım: `python tools/yerler.py <geonames klasörü>`. Klasörde `admin1CodesASCII.txt` ve `admin2Codes.txt` olur.
  - Araç `data/yerler.tsv` dosyasını yazar ve sonunda bir öz-denetim yapar (assert):
    - `TR 48 '' 304183 Muğla` satırı var;
    - `TR 48 8631961 8631961 Köyceğiz` satırı var;
    - hiçbir ad " İlçesi" ile bitmiyor.
- [ ] **Adım 2:** Scratchpad'teki indirilmiş klasörle çalıştır → `OK <satır sayısı>`. Dosya ~2 MB'tan küçük olmalı.
- [ ] **Adım 3:** README'ye atfı ekle. Commit: "Add the country, province and district list from GeoNames"

### Görev 2: Rust — `/api/yerler`

**Files:**
- Modify: `src/main.rs` (yeni fonksiyonlar `sehir_ara`'nın yanında, yönlendirme `/api/sehir_ara`'nın yanında, testler `mod tests`'te)

**Interfaces:**
- Consumes: `data/yerler.tsv` (`include_str!("../data/yerler.tsv")`), `yer(&Value)`, `AGENT`, `GEO`.
- Produces:
  - `fn yer_listesi(ulke: &str, il: Option<&str>) -> Value`
    - `il` yoksa o ülkenin illeri, varsa o ilin ilçeleri.
    - Öğeler `[{ "kod", "id", "ad" }]` biçiminde.
    - Ada göre, Türkçe kurallarıyla küçük harfe çevrilip sıralanır.
  - `fn ulkeler() -> Value`: veride geçen ülke kodları, sıralı (`["AD","AE",…]`).
  - `fn yer_getir(id: &str) -> Result<Value, String>`
    - Yalnızca rakamlardan oluşan ve en fazla 10 haneli id kabul edilir.
    - `GET {GEO tabanı}/get?id=..&language=tr|en` isteği atılır ve sonuç `yer()` biçiminde döner. Ad'ın " İlçesi" / " District" eki kırpılır.
    - Çağrı uç noktası `https://geocoding-api.open-meteo.com/v1/get`.
  - Rotalar:
    - `GET /api/yerler` → `ulkeler()`;
    - `GET /api/yerler?ulke=TR` → iller;
    - `GET /api/yerler?ulke=TR&il=48` → ilçeler;
    - `GET /api/yerler?id=8631961` → `yer_getir`.

- [ ] **Adım 1: Testi yaz** (`fn yer_listesi_tr_mugla`)

```rust
let iller = yer_listesi("TR", None);
assert!(iller.as_array().unwrap().iter().any(|y| y["ad"] == "Muğla" && y["kod"] == "48"));
let ilceler = yer_listesi("TR", Some("48"));
assert!(ilceler.as_array().unwrap().iter().any(|y| y["ad"] == "Köyceğiz" && y["id"] == "8631961"));
assert_eq!(yer_listesi("XX", None), json!([]));
assert!(ulkeler().as_array().unwrap().iter().any(|u| u == "TR"));
assert!(yer_getir("12a").is_err() && yer_getir("").is_err() && yer_getir("12345678901").is_err());
// sorted: Bodrum before Köyceğiz
let a: Vec<&str> = ilceler.as_array().unwrap().iter().filter_map(|y| y["ad"].as_str()).collect();
assert!(a.iter().position(|x| *x == "Bodrum") < a.iter().position(|x| *x == "Köyceğiz"));
```

- [ ] **Adım 2:** `cargo test --release --offline yer_listesi` → derlenmez.
- [ ] **Adım 3: Kodu yaz.**
  - TSV bir kez ayrıştırılır ve `LazyLock<Vec<[&'static str; 5]>>` içinde tutulur.
  - Türkçe küçük harf karşılaştırması `sirala` içindeki `kucuk` ile aynı mantıkta olmalı. Bunun için `kucuk` dışarı alınıp `fn tr_kucuk(s: &str) -> String` olarak paylaşılır.
- [ ] **Adım 4:** testler ve clippy → PASS.
- [ ] **Adım 5: Commit:** "Serve the place lists and look up a place by id"

### Görev 3: UI — "Listeden seç"

**Files:**
- Modify: `src/ui.html` (Ayarlar'daki `#aSehir` satırı ve satır 1027, kart ayarlarındaki `sat()` satırları, `sehirKaydet`)
- Test: `tests/ui/yerliste.test.js` (yeni)

**Interfaces:**
- Consumes: Görev 2'deki `/api/yerler` rotaları.
- Produces:
  - `async function yerListe(): Promise<{ad,bolge,lat,lon}|{hata}|null>`. Seçim penceresini `#sor2` içinde açar.
  - `async function konumKaydet(y)`. `sehirKaydet`'teki kayıt kısmı bu fonksiyona taşınır, `sehirKaydet` de onu çağırır.
  - `function ekSehirEkle(y)`. Ek şehir listesine ekler:
    - en fazla 4 şehir;
    - aynı ad+bölge tekrar eklenmez;
    - ana şehir eklenmez;
    - ekleyince `kaydet()` ve `havaDiger(true)` çağrılır.

- [ ] **Adım 1: Testi yaz.**
  - **Fetch stub'ı:** Yanıtlar şöyle:
    - `/api/yerler` → `['DE','TR']`;
    - `?ulke=TR` → `[{kod:'07',id:'323776',ad:'Antalya'},{kod:'48',id:'304183',ad:'Muğla'}]`;
    - `?ulke=TR&il=48` → `[{kod:'7733242',id:'7733242',ad:'Bodrum'},{kod:'8631961',id:'8631961',ad:'Köyceğiz'}]`;
    - `?id=8631961` → `{ad:'Köyceğiz',bolge:'Muğla',ulke:'Türkiye',lat:36.978,lon:28.724}`.
  - **Kontrol edilecekler:**
    - `'picker: Türkiye default'`: ülke kutusunun değeri `'TR'`, seçeneği "Turkey" ya da "Türkiye" (`Intl`).
    - Muğla seçilince ilçe kutusunda `['—','Bodrum','Köyceğiz']` var. İlk seçenek "yalnızca il" demek.
    - Köyceğiz seçilip onaylanınca `'picker: main city saved'`: `cfg.konum` = `{ad:'Köyceğiz',bolge:'Muğla',ulke:'Türkiye',lat:36.978,lon:28.724}`.
    - `'picker: no network'`: `?id=` için `{durum:'Sunucuya ulaşılamadı'}` dönünce kayıt yapılmıyor ve tost görünüyor.
    - `'extra city added once'`: `ekSehirEkle(y)` iki kez çağrılınca listede tek kayıt kalıyor.
    - Ana şehirle aynı yer eklenmiyor.
- [ ] **Adım 2:** `python tests/ui/calis.py yerliste` → FAIL.
- [ ] **Adım 3: Pencereyi ve düğmeleri yaz.**
  - Pencerede üç `<select>` olur: Ülke, İl, İlçe.
  - Ülke adları `new Intl.DisplayNames([LOC],{type:'region'})` ile gösterilir ve ada göre sıralanır.
  - Varsayılan ülke: kayıtlı `A('yerUlke')`, yoksa `'TR'`. Seçilen ülke `S.ayar.yerUlke` olarak saklanır.
  - İl ya da ilçe değişince sonraki liste `/api/yerler` ile doldurulur.
  - "Seç" basılınca ilçe seçilmişse onun `id`'si, ilçe "—" ise ilin `id`'si `?id=` ile çözülür.
  - Düğmeler:
    - Ayarlar'da `#aSehir`'in yanında "Listeden seç" (`rb` düğmesi; mevcut sınıflar kullanılır) → `konumKaydet`.
    - Kart ayarlarında `sehir` ve `sehirler` satırlarının yanında aynı düğme → `konumKaydet` / `ekSehirEkle`. Ardından alanın değeri `deger(key)` ile yenilenir.
- [ ] **Adım 4:** `python tests/ui/calis.py` (hepsi) → PASS.
- [ ] **Adım 5: Hakkında ve commit.**
  - Hakkında'ya GeoNames atfı eklenir, CHANGELOG'a "Unreleased" maddesi yazılır.
  - Commit mesajı: "Pick the weather city from country, province and district lists". Ardından push.

### Son
- [ ] Tüm dal için son inceleme yapılır, bulgular düzeltilir.
- [ ] Kullanıcı "çıktım" dedikten sonra exe derlenir.
- [ ] Kullanıcı Ayarlar → Listeden seç → Türkiye → Muğla → Köyceğiz yolunu izler. Kart başlığında `Köyceğiz · Muğla` görünmeli.
