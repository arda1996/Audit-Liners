# Banka, Fatura ve Muhasebe Kaydının Birleştirilmesi — 19–24 Temmuz 2026

Bu yazı, 19–24 Temmuz arasında yapılan işi anlatıyor. Tarihler `.claude/sessions/journal.md`'den
alındı. Amacım ne kurulduğunu, hangi kararı neden aldığımızı ve neyin hâlâ eksik olduğunu düz bir
dille kayda geçirmek.

Aralığın başında elimizde üç ayrı parça vardı ve birbirlerine bağlı değillerdi: banka verisi bir
tarafta, faturalar ve e-belgeler başka tarafta, muhasebe defteri üçüncü tarafta duruyordu. Aralığın
sonunda bunlar tek bir hatta bağlandı. Hat şöyle işliyor: bankaya bir hareket düşüyor, bu hareket
faturaya değil cariye ait bir tahsilat olarak kaydediliyor, tahsis motoru bu parayı hangi faturanın
kapattığına karar veriyor, karar gerçek yevmiye kaydına dönüşüyor ve mizana yansıyor.

Sayfa mimarisinde erken bir karar aldık ve buna sonuna kadar uyduk: her sayfa kendi dosyasında,
kendi URL'inde yaşıyor. Banka `Banka.tsx` ve `#/banka`, Belgeler `Belgeler.tsx` ve `#/belgeler`,
iz-sürme penceresi `EslesmeModal.tsx`. Böylece birindeki hata diğerini etkilemiyor. Bundan daha
önemlisi, sayfaların beslendiği uçların sözleşmesi sabit tutuldu; gerçek banka ya da entegratör
bağlandığında uçlar aynı kalacak, sayfa dosyaları değişmeyecek.

Bu aralıkta bir kararı geri aldık. E-Fatura araştırmasından sonra ayrı bir Fatura sayfası kurduk;
sekmeleri vardı, giden, gelen, e-arşiv ve taslak diye ayrılıyordu. Sonra sildik. Sebebi şuydu:
Belgeler sayfasının kopyasıydı ve sekmeler, akıllı filtre çubuğunun zaten yaptığı işi tekrar
ediyordu. E-belge boyutunu, yani belgenin e-Fatura mı e-Arşiv mi olduğunu ve GİB zarf durumunu,
Belgeler'in tek tablosuna erittik. Sonuçta kolon sayısı dokuzdan sekize düşerken tablo iki boyut
daha taşımaya başladı. Buradan çıkan ders, "çok iş yapıyoruz" görüntüsü için parçalı yapı kurmanın
kullanıcıya maliyet yazdığı.

Verinin temelinde deterministik bir simülasyon var. Sabit tohum kullanıyor, `Date::now` ya da
rastgelelik içermiyor, bu yüzden her çalıştırmada aynı sonucu veriyor. Yaklaşık 2.228 fatura ve
370 milyon TL'lik yevmiye hacmi üretiyor ve her ölçümde borç toplamı alacak toplamına eşit çıkıyor.
Ödeme enstrümanlarını bilerek dağıttık: banka yüzde 45, nakit yüzde 25, çek yüzde 15, senet yüzde 10,
açık yüzde 5. Her şeyin bankadan geçtiğini varsaymak Türkiye ticaretinin gerçeğine aykırı olurdu.
Bir de e-belge durumunu ödeme durumundan ayrı bir boyut olarak tuttuk, çünkü bir fatura kabul
edilmiş olduğu halde tahsil edilmemiş olabilir; bunları tek kolonda karıştırmak yanlış olurdu.

Aralığın en temel değişikliği kısmi tahsilat ve FIFO tahsis motoruydu. Önceden her fatura kendi
tahsilatını taşıyordu, bu gerçek muhasebeye uymuyordu. Artık para cariye giriyor ve hangi faturayı
kapattığına tahsis motoru karar veriyor. Motor üç katmanlı çalışıyor: deftere geçmiş dağıtımlar
donmuş kabul ediliyor, muhasebecinin manuel kararları bunların ardından sabitleniyor, kalan para
FIFO ile en eski açık faturadan başlayarak akıyor. FIFO'yu tercih ettiğimiz için değil, kanun böyle
dediği için kullanıyoruz: Türk Borçlar Kanunu 100'e göre borçlu hangi borcu ödediğini belirtmemişse
muaccel ya da en eski borç önce kapanır. Buna bir kural daha ekledik, tahsilat kendisinden sonraki
tarihli faturayı kapatamıyor, çünkü henüz doğmamış bir borç ödenmiş sayılamaz. Doğrulamada bir
carinin iki yüz satış faturasına baktık; ilk 187'si tarih sırasıyla tam kapanmış, sonrasında kapanan
tek fatura yok, yani FIFO düzeni bozulmuyor.

Burada bir kusur çıktı ve düzelttik. İlk çalışan sürümde tek bir manuel düzeltme 365 yeni kayıt ve
366 storno üretti. Sebebi, FIFO'nun tüm geçmişi baştan hesaplaması ve bir tahsilat yer değiştirince
sonraki bütün tahsilatların kaymasıydı. Matematiksel olarak tutarlıydı ama muhasebe olarak yanlıştı,
çünkü kayda geçmiş bir tahsis kendiliğinden değişemez. Deftere fişi olan dağıtımı donduran bir
katman ekledik; artık yalnız düzeltilen tahsilat serbest bırakılıyor ve aynı düzeltme bir storno ile
bir yeni kayıt üretiyor.

Otomatik eşleşmeyi asıl, manuel müdahaleyi istisna olarak kurduk. Otomatik eşleşmenin tek ön koşulu
karşı taraf ünvanının bilinmesi. Ekstrede ünvan yoksa motor eşleştirme yapmıyor ve bunu bilerek
yapmıyor, çünkü hangi cariye ait olduğu bilinmeyen parayı tahmin edip bir cariye yazmak yanlış bakiye
üretir. Ünvan varsa hareket otomatik eşleşiyor ve muhasebeciye hiç görünmüyor. Ünvan yoksa bekleyen
duruma düşüyor ve maker kuyruğuna giriyor; orada muhasebeci önce firmayı seçiyor, sonra o firmanın
açık faturaları FIFO sırasıyla listeleniyor ve seçim yapıyor. Ölçtüğümüzde 998 banka hareketinin
yalnız altısı, yani yüzde 0,6'sı maker'a düşüyor ve hepsinin sebebi aynı. Yani manuel yük sistemin
eksikliğinden değil verinin eksikliğinden geliyor.

Muhasebeci bir tahsilatı elle bir faturaya bağladığında o tahsilatın önceki otomatik eşleşmesi
kalkıyor. Bunu ayrı bir kaldırma koduyla yapmıyoruz, kurgu gereği oluyor: manuel tahsisler FIFO'dan
önce işlendiği için dağıtım baştan hesaplandığında eski eşleşme kendiliğinden ortadan kalkıyor,
artan para yine FIFO ile akıyor. Bir örnekte 124.912,64 liralık tahsilat otomatik olarak bir faturaya
gitmişti; manuel olarak başka bir faturaya bağlandığında eski eşleşme kaldırıldı, hedef fatura
124.328,38 aldı ve artan 584,26 lira FIFO'ya aktı. Toplam kuruşu kuruşuna tahsilat tutarına eşit
kaldı, yani para ne kayboldu ne çoğaldı.

İptal işlemini muhasebe kuralına göre kurduk. Vergi Usul Kanunu 219 ve Türk Ticaret Kanunu 65
gereği kayıt silinemez, karalanamaz; düzeltme ters kayıtla yapılır. Bir eşleşme değiştiğinde asıl
kayıt defterde kalıyor ve durumu iptal olarak işaretleniyor, karşısına borç ve alacağı yer değiştiren
bir storno fişi açılıyor, yeni eşleşme ise ayrı bir fiş olarak giriliyor. Asıl kayıt ile storno net
sıfır ediyor ve üçü de defterde görünüyor, böylece denetim izi kopmuyor. Mükerrer kaydı engellemek
için her tahsise doğal bir anahtar verdik; tahsilat, fatura ve tutarın birleşimi. Senkronizasyonu
dört kez üst üste çağırdık, ilkinde 3.999 fiş üretti, sonraki üçünde sıfır. İnce bir davranış da
doğru çalışıyor: bir tahsilat yeniden dağıtıldığında değişmeyen payı storno edilmiyor, yalnız değişen
kısım storno ve yeni kayıt üretiyor.

Aralığın sonunda sistemin dikişlerini denetledik, çünkü parçaların birleşiminde sorun olduğu
sezilmişti ve bu sezgi doğru çıktı. En büyük sorun şuydu: muhasebe kayıtları gerçek deftere hiç
girmiyordu. Mizan sıfır satırdı, uygulamanın fiş listesi boştu, ama tahsis defterinde 4.003 kayıt ve
simülasyonda 4.132 fiş vardı. Yani motor çalışıyor, tahsis defteri tutuyordu, ama yevmiye, kebir ve
mizan bunların hiçbirini görmüyordu. Tahsis defteri bir kayıt değil, kayıt adayıydı ve kimse onu
deftere geçirmiyordu. Bunun için bir aktarım ucu yazdık. Aktarım iki katman üretiyor: fatura
tahakkuku, yani satışta 120 borç ile 600 ve 391 alacak, alışta 153 ve 191 borç ile 320 alacak; ve
eşleşen para hareketi, yani tahsilatta 102 borç ile 120 alacak, ödemede 320 borç ile 102 alacak.
Aktarım sonucunda 6.227 fiş deftere girdi, mizan doldu ve borç toplamı alacak toplamına eşit çıktı.
Tutarlılığı ayrıca ölçtük: 120 hesabının bakiyesi ile 320 hesabının bakiyesinin toplamı 29.388.406,22
lira, bu da simülasyonun açık bakiyesine birebir eşit.

İkinci kopukluk iz-sürmedeydi. İz ucu saf FIFO hesabını okuyordu, yani manuel eşleştirmeleri ve
deftere geçmiş donmuş dağıtımı yok sayıyordu. Muhasebeci ünvansız bir girişi elle eşlediğinde defter
o kaydı manuel olarak gösteriyor, ama iz-sürme penceresi bomboş dönüyor ve eşleşmeyen diyordu. Bu
denetim izi için ciddi bir kusurdu, çünkü izin gösterdiği gerçeklik defterin gerçekliğinden farklıydı.
İz ucu artık listelerle aynı kaynağı okuyor.

Üçüncüsü daha küçük ama öğreticiydi. Storno ve düzeltme kayıtları deftere giremiyor, kronoloji
nedeniyle atlanıyordu. Sebebi teknik değil muhasebîydi: yevmiye geriye tarihlenemez. Doğru davranış
zaten muhasebe pratiğinin kendisi; düzeltme kaydı olayın tarihine değil, kaydın yapıldığı tarihe
atılır, belgenin kendi tarihi ise dayanakta korunur, böylece iz kopmaz. Bunu uyguladıktan sonra
düzeltmeler deftere girmeye başladı.

Dürüstlük payı olarak şunu da yazayım: bu denetim sırasında iki kez yanlış alarm verdim. Birincisinde
banka hareketi ile defteri karşılaştırırken storno kayıtlarını negatif işaretlemeyi unuttum ve olmayan
bir fark gördüm; doğru formülle 998 hareket ve 200 fatura üzerinde hiç fark yok. İkincisinde defter
sorgusunun bin kayıtta kırpıldığını gözden kaçırdım ve manuel kayıtların olmadığını sandım, oysa
listenin sonundaydılar. Yani banka, fatura ve tahsis arasındaki hiza zaten sağlamdı; bozuk olan bu
hizanın deftere ulaşmamasıydı.

E-Fatura ve e-Arşiv tarafında bu aralıkta kod yazmadık ama entegrasyon hazırlığını araştırdık. İki
ajan yalnız istek tabanlı olarak, tarayıcı açmadan çalıştı. Öğrendiğimiz en önemli şey belgenin JSON
değil UBL-TR 1.2 XML olduğu; JSON olan kısım entegratörün taşıma katmanı. Üç yol var: GİB Portal'ın
resmî bir API'si yok, dolayısıyla üretimde kullanılmaz; doğrudan entegrasyon mümkün ama ağır; özel
entegratör ise piyasanın çoğunluğunun kullandığı yol ve bizim yolumuz. Bütün entegratörler aynı beş
fiili sunuyor: giriş, gönder, listele, indir ve durum sorgula. Bu yüzden iç API'yi tek bir desene
oturtup her entegratör için ayrı adapter yazmayı planladık. Ayrıntısı `docs/tasarim/efatura-entegrasyon.md`'de.

Aynı aralıkta yardımcı sistemler de kuruldu. Filtre çubuğu tablo içi hücre filtrelerinin yerini aldı
ve tamamen sunucu tarafında çalışıyor; bin firma olduğunda istemci tarafı filtre anlamsızlaşıyordu.
Firma, durum, yön, ödeme enstrümanı, tutar aralığı, tarih aralığı, e-belge durumu ve tipi hepsi aynı
çubuktan süzülüyor. Havada bakiye kavramını ekledik; hiçbir faturaya bağlanmayan banka girişi tamlık
ihlali ve kayıt dışı hasılat riski taşıyor, muhasebeci eşleştirince listeden düşüyor. İz-sürme
penceresinde zincir bandı, kaynak rozetleri ve sayfalar arası tıklanabilir geçiş var; çekişmeli ajan
denetiminin sonucu olarak güven etiketini yüzde yüz göstermeyi bıraktık, artık otomatik ipucu olduğunu
ve denetçi teyidi gerektiğini yazıyor, ayrıca simülasyon verisinin kanıt olmadığını belirten bir bant
duruyor. Fiş şablonlarına muhasebecinin kendi şablonlarını ekleme, düzenleme ve silme yeteneği geldi;
tutarları bilerek saklamıyoruz, çünkü kira otuz binken kaydedilen şablon kira otuz iki bin olduğunda
sessizce yanlış fiş üretirdi. Şablonun en az bir borç ve bir alacak bacağı olması zorunlu, hesabın
planda bulunması gerekiyor ve hazır şablonlar korunuyor. Kullanım sayacı ekledik, sık kullanılanlar
listenin başına geliyor. Sidebar on beş satırdan dört satıra indi, üzerine gelince açılan bir pencere
kullanıyor ve bu pencere portal ile basılıyor ki sayfa içi açılır kutuların altında kalmasın. Vergi
ve raporlama ile bağımsız denetimi ayırdık, çünkü biri VUK biri BDS ve TFRS katmanı; UFRS WorkSheet'i
denetim tarafına aldık. Banka ve Belgeler ayrı bir Finans başlığı olmaktan çıkıp muhasebenin
fonksiyonu haline geldi.

Eksikleri de açıkça yazmam gerekiyor. En önemlisi kalıcılığın olmaması. Şablonlar, manuel tahsis
kararları, tahsis defteri ve deftere aktarılan 6.230 fiş, hepsi bellekte duruyor ve sunucu yeniden
başladığında kayboluyor. Bu artık bir kolaylık meselesi değil; storno zinciri bir denetim izidir ve
kaybolması "kayıt silinmez" ilkesini fiilen ihlal eder. Sıradaki iş bu olmalı. İkinci eksik, eşleşme
ipucunun tek boyutlu olması; şu an yalnız ünvana bakıyoruz, oysa IBAN'dan cariye eşleme, açıklamadaki
fatura numarasını yakalama ve tutar ile tarihin birebir eşleşmesi eklenirse maker yükü daha da düşer.
Üçüncü madde 24 Temmuz akşamı kapandı: test paketi koşuldu ve tahsis motoru kalıcı testlere bağlandı.
Koşarken bir test kırıldı, ama sebebi kod değil testin kendisiydi; hesap planının yaprak sayısını 261
olarak sabitliyordu, oysa plana 549.90 aktüeryal kazanç/kayıp fonu gibi TFRS hesapları meşru olarak
eklenmişti ve sayı 266'ya çıkmıştı. Sabit sayı iddiası, her meşru ekleme yapıldığında testi kırıp
ortada gerçek bir hata varmış izlenimi veriyordu. Testi gerçek değişmezlere bağladım: MSUGT çekirdeği
eksiksiz olmalı, kodlar tekil olmalı, başlıklar yaprak olmamalı ve motorların kayıt attığı çekirdek
hesaplar (100, 102, 120, 153, 191, 320, 391, 600) yaprak olarak bulunmalı. Ardından FIFO motoru için
dokuz yeni test yazdım; bunlar para korunumunu, faturanın tahsil ve kalan tutarlılığını, FIFO
sırasının korunmasını, gelecek tarihli faturanın kapatılamamasını, ünvansız tahsilatın otomatik
dağıtılmamasını, tahsisin cari ve yön uyumunu, manuel kararın FIFO'yu ezmesini, motorun determinizmini
ve üretilen fişlerin dengeli olmasını doğruluyor. Böylece daha önce elle yaptığım ve her seferinde
kaybolan uç doğrulamaları kalıcı korumaya çevirdim. Şu an tüm çalışma alanında 79 test geçiyor, hiç
başarısız yok; tip kontrolü ve derleme de temiz.
Dördüncüsü, arayüzün görsel teyidini yapamadım, çünkü giriş parolasını ben giremiyorum; backend'i
uçtan uca doğruladım ama ekran teyidi sende. Beşincisi, e-Fatura oluşturma formu ve entegratör
adapter'ı henüz yazılmadı, gerçek entegratör erişimini bekliyor. Son olarak simülasyonun kanıt
olmadığını tekrar belirtmek isterim; gerçek açık bankacılık bağlandığında banka hareketi güçlü bir
dış kaynak olacak ama yine de dış teyit yerine geçmeyecek.

Sıradaki işleri şu sırayla öneriyorum: önce kalıcılık, çünkü diğer her şeyin önünde duruyor ve
kaybolan şey artık sadece kolaylık değil denetim izinin kendisi; sonra eşleşme ipuçlarının
çoğaltılması, yani IBAN'dan cariye eşleme, açıklamadaki fatura numarasını yakalama ve tutar ile
tarihin birebir eşleşmesi; en son da entegratör erişimi geldiğinde e-Fatura oluşturma formu ve
adapter. Kalıcılık yazıldığında storno zinciri için de test yazılmalı, çünkü şu an test edilen
katman tahsis motoru; deftere aktarım ve iptal davranışı hâlâ yalnız uç düzeyinde doğrulanıyor.
