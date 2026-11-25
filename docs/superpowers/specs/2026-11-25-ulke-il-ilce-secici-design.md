# Ülke → il → ilçe seçici — kısa tasarım

## Neden
Hava durumu şehri ada göre aranıyor. Aynı adı taşıyan yerler (Köyceğiz: Muğla ve Şırnak) karışıyor. Kullanıcının kayıtlı konumu da bölgesiz kaydedildiği için hangi yer olduğu görünmüyor. Bu ayarla kullanıcı yeri listeden seçecek, yer hiç tahmin edilmeyecek.

## Hedefler
- **Kademeli seçim:** Ayarlar'da hava durumu şehri için bir "Listeden seç" düğmesi olacak. Düğme üç basamaklı bir pencere açar: Ülke → İl → İlçe. Her seçim bir sonraki listeyi doldurur.
  - Ülkede ilçe düzeyi yoksa il seçimi yeter.
- **Kayıt biçimi:** Seçim, bugünkü yer kaydıyla aynı biçimde kaydedilir: `{ad, bolge, lat, lon}`.
  - Örnek: `{ad:"Köyceğiz", bolge:"Muğla", ...}`.
  - Ana şehir için `/api/config {konum}` kullanılır, ek şehirler için `S.ayar.sehirler`. Bu sayede kart, tema ve gün batımı başka bir değişiklik gerektirmeden çalışır.
- **Veri uygulamanın içinde:** Liste internetsiz çalışır, hesap istemez, seçimler dışarıya gönderilmez.

## Hedef olmayanlar
- Köy ya da mahalle düzeyi (kullanıcı ilçe düzeyini yeterli buldu).
- Bugünkü adla arama kalkmıyor. Yazarak arama da seçici de birlikte duruyor.
- Veriyi otomatik güncellemek. Veri, sürüm çıkarılırken elle yenilenir.

## Yaklaşımlar
1. **GeoNames verisini RYOCS'un içine gömmek.** Kullanıcı bunu seçti. İnternetsiz ve hesapsız çalışır; exe birkaç MB büyür.
2. **GeoNames'in canlı servisi.** Hesap ister ve her tıklama internete gider.

## Kararlar (benim seçimlerim)
- **Kaynak:** GeoNames'in herkese açık dosyaları (`countryInfo`, `admin1CodesASCII`, `admin2Codes`, `cities500`). Lisansı CC BY 4.0; atıf README'ye ve Hakkında'ya eklenir.
- **Koordinat:** İlçe için o ilçenin merkezi kullanılır (GeoNames'te `PPLA2` olarak işaretli yer). Merkez yoksa ilçedeki en kalabalık yer alınır. İl düzeyinde de il merkezi (`PPLA`) alınır.
  - Böylece Köyceğiz seçilince Köyceğiz kasabasının koordinatı kullanılır.
- **Veri dosyası:** Bir Python aracı (`tools/yerler.py`) indirilen GeoNames dosyalarından tek bir `data/yerler.tsv` dosyası üretir. Satır biçimi: `ülke kodu | il kodu | ilçe kodu | ad | lat | lon`.
  - Dosya repoya eklenir ve `include_str!` ile exe'ye gömülür.
  - Tahmini boyutu 2 MB'tan az. Yeni crate eklenmez, veri sıkıştırılmaz.
- **Ülke adları:** WebView'in `Intl.DisplayNames` özelliğiyle arayüz dilinde gösterilir (TR/EN). Veriye ülke adı eklenmez.
- **İl ve ilçe adları:** GeoNames'teki adlar kullanılır (Muğla, Köyceğiz). İngilizce arayüzde de aynı adlar görünür.
- **API:** `GET /api/yerler?ulke=TR` illeri döndürür. `GET /api/yerler?ulke=TR&il=48` o ilin ilçelerini döndürür.
  - Her öğe `{kod, ad, lat, lon}` içerir. Liste ada göre, Türkçe sıralamayla dizilir.
  - Veri ilk istekte bir kez ayrıştırılır ve bellekte tutulur.
- **Pencere:** Üç `<select>` kutusu olur; klavyeyle harf yazınca listede o harfe atlanır. Varsayılan ülke Türkiye'dir, başka ülke seçilmişse o ülke gelir.
  - Ek şehirlerin alanının yanında da aynı düğme bulunur.

## Testler
- **Rust:**
  - TSV ayrıştırılıyor.
  - TR → Muğla illerde var.
  - Muğla'nın ilçelerinde Köyceğiz var ve koordinatı Muğla'daki Köyceğiz'in koordinatına yakın (~36.97, 28.68).
  - İlçesi olmayan bir ülkede ilçe listesi boş dönüyor.
- **UI:**
  - Seçici sırasıyla Türkiye → Muğla → Köyceğiz seçilince ana şehir `{ad:'Köyceğiz', bolge:'Muğla', lat, lon}` olarak kaydediliyor.
  - İlçesi olmayan ülkede il seçimi kayıt için yeterli oluyor.

## İzin gereken adım
GeoNames dosyalarının bir kez indirilmesi (`download.geonames.org`, yaklaşık 15 MB). Bunun için kullanıcıdan ayrıca onay alınacak.
