<p align="center">
  <a href="https://space.pentomake.com">
    <picture>
      <img src="docs\media\readme\space.png" alt="Space">
    </picture>
  </a>
</p>

<p align="center">
  <a href="https://github.com/strongd3v/test/blob/test/LICENSE">
    <img src="https://img.shields.io/github/license/strongd3v/test?style=for-the-badge&color=red" alt="License">
  </a>
  <a href="https://github.com/strongd3v/test/releases">
    <img src="https://img.shields.io/github/v/release/strongd3v/test?style=for-the-badge&color=green" alt="Latest Release">
  </a>
  <a href="https://github.com/strongd3v/test/blob/test/Cargo.toml">
    <img src="https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fstrongd3v%2Ftest%2Ftest%2FCargo.toml&query=%24.workspace.package.edition&label=Rust&style=for-the-badge&color=blue" alt="Rust Edition">
  </a>
  <br>
  <a href="README-DE.md">Deutsch</a> | <a href="README.md">English</a> | <a href="README-ES.md">Español</a> | <a href="README-FR.md">Français</a> | <a href="README-PT.md">Português</a> | <strong>Türkçe</strong> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <a href="README-JA.md">日本語</a>
</p>

Space, kodlama ajanları için yerel bir masaüstü kontrol arayüzüdür: günlük işlerinizde birlikte kullandığınız ajan CLI'larını, araçları, projeleri ve cihazları tek bir arayüzde birleştirir. Tarayıcı sekmesinde yaşamak ya da tek bir sağlayıcıya kilitlenmek yerine Space, makinelerinizde zaten kurulu olan kodlama ajanlarını, Claude Code'dan Cursor'a, sürer; her oturum, çıktı ve ayar sizin kendi donanımınızda kalır.

Arayüzün altında Space, hızlı bir masaüstü uygulamasını gerçek bir motorla eşleştirir: oturumları, transkriptleri, terminalleri ve çalışma alanı durumunu yöneten, uygulamadan ya da bir MCP istemcisinden erişilebilen yerel bir servis. Linux, macOS ve Windows derlemeleri [Releases](https://github.com/strongd3v/test/releases) sayfasında yayımlanır.

---

### Neden Space

* **Ajanınızı siz seçin.** Antigravity, Claude Code, Codex, Cursor, Devin, fx, Grok, Hermes, Muse Code, OpenCode ve Pi'nin tamamı, kendi doğal protokolleriyle veya ACP üzerinden tek bir arayüzün arkasında sürülür. Hangi harness ve modeli kullanacağınıza konuşma başına siz karar verirsiniz; Space, arkasında hangi ajan olursa olsun çalışma akışını tutarlı tutar.

* **Native çalışır.** Space, Rust ile yazıldı ve GPUI ile çizilir; GPUI, Zed'in arkasındaki arayüz çerçevesidir ve bu projeye vendor edilerek genişletilmiştir. Electron yok, tarayıcı sarmalayıcısı yok: akan transkriptler, sözdizimi renklendirmeli diff'ler ve canlı terminal panelleri uzun oturumlarda bile akıcılığını korur.

* **Yerel öncelikli.** Oturumlar, ayarlar ve çalışma alanı durumu, kendi veri dizininizin altındaki diskte yaşar ve motor, hesap gerektirmeden işlem içinde başlar. Ağı çıkarmak işinizi durdurmaz; senkron, katılmayı seçtiğiniz bir şeydir, bağımlılık değil.

* **Gerçek işe dayanacak şekilde inşa edildi.** Koşular, dayanıklı bir komut kuyruğu ve çökme kurtarma günlüğü üzerinde ilerler; böylece yarıda kesilen bir konuşma, yeniden başlatmanın ilerlemeyi kaybettirmesi yerine kaldığı yerden devam eder.

* **Bir sohbet kutusu değil, tam bir çalışma alanı.** Konuşmanın yanında Space; dosya gezgini, gömülü bir PTY terminali, git değişiklikleri ve diff görünümleri, bir pull request paneli, gömülü bir tarayıcı ile eylemleri, konuşmaları, çalışma alanı dosyalarını ve dosya içeriğini arayan tek bir komut paleti sunar.

* **İstediğinizde sesli.** Sözlü yazım ve tam pencere ses sahnesi, yerel bir konuşma tanıma modeliyle çalışır; böylece sesiniz makineden asla çıkma zorunda kalmaz.

---

### Desteklenen Ajanlar

Space, her ajan CLI ile onun gerçekte konuştuğu protokol üzerinden konuşur:

* **Antigravity** (Google): ACP üzerinden sürülür.
* **Claude Code** (Anthropic): doğal akış protokolü üzerinden sürülür.
* **Codex** (OpenAI): doğal uygulama sunucusu protokolü üzerinden sürülür.
* **Cursor**: Cursor CLI, Agent Client Protocol (ACP) üzerinden sürülür.
* **Devin** (Cognition): ACP üzerinden sürülür.
* **fx** (Vercel): kendi yerel ACP sunucusu üzerinden sürülür.
* **Grok** (xAI): Grok Build ajanı, ACP üzerinden sürülür.
* **Hermes** (Nous Research): ACP üzerinden sürülür.
* **Muse Code** (Meta): Muse Session Protocol (MSP) üzerinden yerel olarak sürülür.
* **OpenCode** (SST): HTTP/SSE sunucu protokolü üzerinden yerel olarak sürülür.
* **Pi**: doğal JSONL RPC protokolü üzerinden sürülür.

Her harness bağımsız bir kurulum olarak kalır: Space, güncelleme bildirimlerini yan barda gösterir ve arkanızdan hiçbir şeyi değiştirmez; bestecinin harness ve model seçicisi, konuşma başına ajan değiştirmenizi sağlar. Böylece planlama bir modelle, uygulama başka bir modelle yürütülebilir.

---

### Konuşmalar

Space her oturumu yerel olarak saklar; böylece geliştirme, tek bir pencere içinde ölüp bitmek yerine günler boyunca sürebilir. Geçmiş aranabilir ve devam ettirilebilir, konuşmalar yeniden adlandırılabilir ya da orijinal akışı kaybetmeden bir alternatifi araştırmak için yan sohbetlere çatallanabilir; taslaklar yeniden başlatmalara dayanır.

Transkriptin kendisi sanallaştırılmış ve artımlı olarak ayrıştırılır: markdown, sözdizimi renklendirme, LaTeX matematiği, katlanmış araç grupları ve alta yapışkan akış, binlerce iletiyle bile akıcı kalır. Bestecinin Send/Steer/Stop akışı, çalışan bir turun arkasına ileti kuyruğa almanızı, ajanı yolda bir düzeltmeyle yeniden yönlendirmenizi veya doğrudan kesintiye uğratmanızı sağlar. Bir ajan size soru sorduğunda yanıt satır içinde verilir; görseller veya kaydedilen ekran görüntüleri, konuşmanın bulunduğu yere doğrudan eklenir.

---

### Çalışma Alanı

Space, projeyi ayrı bir program yerine konuşmanın bir parçası olarak ele alır:

* **Dosyalar paneli**: git durumunu, yeniden adlandırmaları ve hızlı düzenlemeleri gösteren gezgin.
* **Terminal**: sınırlı geri oynatma özelliğiyle gömülü PTY sekmeleri; yerel olarak çizilir.
* **Değişiklikler**: sohbetin yanında yan yana diff'ler ve git değişiklikleri görünümü.
* **Pull request'ler**: inceleme, yorum ve todo takibi için ayrılmış bir panel.
* **Tarayıcı**: işin yanında gerçek, sekmeli bir webview; önizlemeler ve dokümantasyon için.
* **Appshots**: masaüstünüzdeki herhangi bir pencereyi yakalar ve uygulamanın erişilebilirlik bağlamıyla ekler; ajan sizin gördüğünüzü görür.
* **Komut paleti**: tek bir tuş vuruşuyla eylemler, konuşmalar, çalışma alanı dosyaları ve dosya içeriğini arar.

Sağ panel, konuşmayı görünümden çıkarmadan dosyalar, diff'ler, terminaller, yan sohbetler, alt ajan transkriptleri ve tarayıcı arasında geçiş yapar.

---

### Ses

Sözlü yazım ve tam pencere ses sahnesi, yerel bir konuşma tanıma modeliyle çalışır: siz konuşursunuz, Space yazar; altyazılar ve canlı transkripsiyon aynı arayüzde çizilir. Sesiniz makineden asla çıkmaz.

---

### MCP

`space mcp`, motorun yerel API'sini vekil eden bir Model Context Protocol stdio sunucusu çalıştırır; böylece dış araçlar ve diğer ajanlar, Space oturumlarınızı standartlaştırılmış bir arayüz üzerinden sürebilir.

---

### Güvenlik

Bir güvenlik açığı bulduysanız, lütfen genel bir konu açmak yerine özel olarak bildirin: bildirim süreci iletişim ayrıntıları [SECURITY.md](./SECURITY.md) dosyasında yer alır.

---

### Katkıda Bulunma

Space açık olarak geliştirilir ve katkılar memnuniyetle karşılanır. Yeni ajan CLI'ları için harness sürücüleri, transkript ve terminaldeki düzeltmeler, paketleme ve dokümantasyonun tamamı yararlıdır. Küçük ve odaklı pull request'ler, incelemesi en kolay olanlardır.

Büyük bir değişikliğe başlamadan önce [katkı belgelerini](./CONTRIBUTING.md) okuyun ve işin, projenin yerel öncelikli ve yerel masaüstü odağıyla tutarlı kalması için bir konu açarak yönü tartışın. Katılan herkes, [Davranış Kuralları](./CODE_OF_CONDUCT.md)'na uymayı kabul eder.

---

### Sponsor

Space, küçük bir ekip tarafından geliştirilir ve sürdürülür; sponsorluk bunu böyle tutar: yerel motoru ve daha fazla ajan harness'ını finanse eder. Space süreçlerinize zaman kazandırıyorsa, sponsor olmak için [space@pentomake.com](mailto:space@pentomake.com) üzerinden ekibimize ulaşın.

---

### Lisans

Space, [GPL-3.0 lisansı](./LICENSE.md) ile lisanslanmıştır.
