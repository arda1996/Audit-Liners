# Sistem Görselleri

Programın dört ana parçasını ve aralarındaki bağı anlatır. SVG'dir — ölçeklenir, metni seçilebilir.

| Görsel | Ne anlatır |
|---|---|
| [1 · Muhasebe akışı](1-muhasebe-akisi.svg) | Belge → fiş → yevmiye → kebir → mizan → beyanname. İptal≠iade, kayıt silinmez, KDV oran kırılımı, ÖTV→KDV zinciri |
| [2 · Banka API hattı](2-banka-api.svg) | Açık bankacılık → ekstre → tahsilat olayı → tahsis. Banka/kasa defteri, tevsik zorunluluğu, havada bakiye |
| [3 · Fatura sistemi](3-fatura-sistemi.svg) | e-Belge durum makinesi, net borç hesabı, KDV muavinleri, entegrasyon yolları |
| [4 · Tahsis motoru](4-tahsis-motoru.svg) | Donmuş → manuel → FIFO önceliği, dört sonuç durumu, güvenlik katmanı (K/D kodları) |

Görsellerdeki her kural koddan ve dokümandan alınmıştır, temsilî değildir:
`docs/learnings/iptal-iade-muhasebesi.md` · `crates/api/src/kontrol.rs` · `crates/api/src/simulasyon.rs`
