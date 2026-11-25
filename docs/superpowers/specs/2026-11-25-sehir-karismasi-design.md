# Aynı adlı şehirlerin karışmaması — kısa tasarım

## Neden
"Köyceğiz" kaydı Şırnak'taki (İdil) Köyceğiz'e gidiyordu. Kullanıcı bunu ancak gün batımı saati yanlış çıkınca fark etti, çünkü ekranda hangi yerin seçildiği görünmüyordu.

3bc0c62'de çözüm olarak açılışta "hangisi?" sorusu eklendi, ama bu soru gerçek uygulamada iki kez çıkmadı. Testlerde çıkıyor, nedeni bilinmiyor. Bu yüzden düzeltme açılıştaki pencereye bağlı kalmamalı.

## Hedefler
1. **En olası yer kendiliğinden seçilsin.**
   - Rust'ta `sehir_ara` ve `konum()` sonuçları şu sırayla dizilir:
     - önce adı ilçesinin adıyla aynı olan yer (ilçe merkezi);
     - sonra nüfusu en kalabalık olan;
     - sonra API'nin döndürdüğü sıra.
   - `konum()` en üstteki yeri alır (count 1'den 8'e çıkar). Sadece adla kaydedilmiş "Köyceğiz" böylece pencere açılmadan Muğla'ya gider.
2. **Bölge hava kartında görünsün.**
   - Kart başlığında şehir adının yanında bölge soluk renkte yazılır: `Köyceğiz · Muğla`.
   - Bölgesi olmayan yerde (Ankara) yalnızca ad görünür.
   - Yanlış yer seçilirse bu sayede bir bakışta anlaşılır.
3. **Açılış sorusunun neden çıkmadığı bulunsun.**
   - `konumDenetle` her adımında `hata.log`'a kısa bir "bilgi:" satırı yazar:
     - başladı / kilitli;
     - config'deki `konum` değeri;
     - kaç yer bulundu ya da hangi hata geldi;
     - pencere sonucu (seçildi, iptal edildi ya da kapandı);
     - kaydın sonucu.
   - Yalnızca koordinatsız eski kayıt varken çalışır, yani normalde günlüğe hiçbir şey yazmaz.

## Hedef olmayanlar
- Bilgisayarın konumu ya da saat dilimi ile doğrulama (gizlilik açısından ağır ve işe yaramıyor).
- Eski ek şehir dizelerini açılışta tek tek sormak. Bu, açılış sorusu güvenilir hale gelene kadar ertelendi.
- Hava kartında "Seç" düğmesi (2. yol, seçilmedi).

## Yaklaşımlar
1. **Akıllı seçim ve bölgeyi göstermek.** Seçildi; pencereye ihtiyaç duymuyor.
2. **Kartta "Hangi Köyceğiz?" düğmesi.** Pencereye bağlı değil ama tıklamayı kullanıcıya bırakıyor.
3. **Tanı günlüğü.** Seçildi, 1. yolun yanında; tek başına bir şey düzeltmiyor.

## Kararlar (benim seçimlerim)
- **Karşılaştırma:**
  - Ad, admin2'den " İlçesi" / " District" eki atılarak elde edilen ilçe adıyla karşılaştırılır.
  - Büyük-küçük harf farkı Türkçe kurallarıyla yok sayılır.
  - Rust'ta bir birim testi yazılır: `[Şırnak/İdil, Amasya, Muğla/Köyceğiz]` → Muğla başta.
- **Açılış sorusu yerinde kalır.**
  - Rust zaten en olası yeri seçtiği için sorunun varsayılan seçeneği de o yer olur.
  - Soru çıkarsa kullanıcı onaylar ya da yeri değiştirir.
- **Bölge gösterimi:**
  - Bölge `.hvAd` içinde `<small>` olarak durur.
  - Dar kartta yazı üç noktayla kısalır; bölgenin tamamı `title` içinde kalır.
- **Günlük:** Satırlar mevcut `/api/log` ile yazılır. Yer adı ve sonuç sayısı yazılır, koordinat yazılmaz.

## Testler
- **Rust:** sıralama birim testi.
- **UI:**
  - Kart başlığında bölge görünür; bölgesi olmayan yerde görünmez.
  - `konumDenetle` günlük satırlarını gönderir; konum kayıtlıyken hiçbir satır göndermez.
