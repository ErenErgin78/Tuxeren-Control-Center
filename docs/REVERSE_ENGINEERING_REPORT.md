# Tuxeren Control Center (TxeCtrl): Tersine Mühendislik, Sürücü Adaptasyonu ve Donanım Kontrolü Geliştirme Süreci

Bu doküman; Windows'tan Linux'a geçiş sonrasında Monster dizüstü bilgisayarlarda karşılaşılan donanım denetimi eksikliğinin, üretici yazılımına uygulanan tersine mühendislik adımlarının, sürücü seviyesindeki uyumluluk engellerinin ve sıfırdan geliştirilen **Tuxeren Control Center (`txectrl`)** projesinin teknik serüvenini kayda geçirir.
---

## 1. Arka Plan ve Çıkış Noktası

Windows işletim sisteminden Linux'a (Fedora) geçiş yapıldığında, Monster dizüstü bilgisayarlar (hedef donanım: Monster Tulpar T7 V20.7 / Tongfang-Uniwill `GM7AR0R` kasası) için üretici tarafından sunulan resmi bir kontrol merkezi yazılımının bulunmadığı görüldü. Cihazın fan hızları, işlemci güç limitleri (PL1/PL2/PL4), klavye arka aydınlatması ve kasa üzerindeki fiziksel mod butonu Linux tarafında kontrolsüz durumdaydı.

Yürütülen araştırmada, Monster dizüstü bilgisayarların Tongfang ve Uniwill ODM (Original Design Manufacturer) şasilerini kullandığı, bu şasilerin Almanya merkezli Linux bilgisayar üreticisi Tuxedo Computers tarafından da kullanıldığı tespit edildi. Teorik olarak Tuxedo'nun açık kaynaklı yazılımlarının bu cihazda da çalışması bekleniyordu.

Ancak yapılan testlerde şu engellerle karşılaşıldı:
1. **Kaynak Eksikliği ve Eski Dokümantasyon:** İnternetteki yönlendirmelerin çoğu güncelliğini yitirmişti.
2. **Kullanıcı Arayüzü Hantallığı:** Tuxedo Control Center (TCC) doğrudan kaynak kodundan derlenip çalıştırıldığında arayüz oldukça hantal kaldı ve görsel olarak modern standartların gerisindeydi.
3. **Boş Telemetri ve Çalışmayan Donanım:** En kritik sorun, arayüz açılmasına rağmen hiçbir donanım verisinin (fan devirleri, sıcaklıklar, güç profilleri) görünmemesiydi; kontroller tamamen etkisizdi.

### Sorunun Kök Nedeni: DMI / Üretici Kimlik Kısıtlaması
Tuxedo sürücülerinin kaynak kodu (`tuxedo-drivers`) detaylı olarak incelendiğinde `tuxedo_compatibility_check.c` dosyasında şu kontrol tespit edildi:

```c
// Going forward we only run the drivers on in house tested devices
static const struct dmi_system_id tuxedo_dmi_string_match[] = {
    {
        .matches = {
            DMI_MATCH(DMI_SYS_VENDOR, "TUXEDO"),
        },
    },
    ...
};
```

Tuxedo, sürücünün yalnızca kendi test ettikleri cihazlarda çalışması için BIOS DMI tablosundaki `DMI_SYS_VENDOR` değerinin `"TUXEDO"` olmasını şart koşuyordu. Monster ise aynı anakart ve gövdeyi kullanmasına rağmen BIOS DMI alanlarına kendi marka kimliğini yazıyordu:
- `DMI_SYS_VENDOR`: `"MONSTER"`
- `DMI_BOARD_NAME`: `"GM7AR0R"`
- `DMI_PRODUCT_NAME`: `"TULPAR T7 V20.7"`

Bu nedenle sürücü yükleniyor görünse dahi uyumluluk testini geçemiyor, donanım düğümlerini oluşturmuyor ve arayüze veri akışı sağlayamıyordu.

---

## 2. Tersine Mühendislik Süreci: Windows Kontrol Merkezinin Deşifre Edilmesi

Sorunun çözümü için başkalarının sürücülerine bağımlı kalmak yerine, donanımın kendi dilini çözme kararı alındı. Monster'ın resmi destek sayfasından cihaza ait Windows sürümü olan `ControlCenter_5.8.49.19_Monster.exe` indirildi ve tersine mühendislik sürecine başlandı.

### 2.1 İkili Dosya Ayrıştırma ve Mimari Katmanlar
Yükleyici dosyasının bir **Inno Setup 6.3.0** kapsayıcısı olduğu saptandı ve içeriği ayıklandı. Çıkarılan dosyalar arasında 158 MB boyutundaki `GamingCenter3_Cross.pdb` sembol dosyasının yer alması, tüm fonksiyon ve yapı isimlerinin doğrudan deşifre edilmesini sağladı.

İnceleme sonucunda Windows yazılımının üç katmanlı mimarisi ortaya çıkarıldı:
- **Kullanıcı Arayüzü:** UWP C++/CX tabanlı `GamingCenter3_Cross.dll` ve sistem tepsisini yöneten `SystrayComponent.exe`.
- **Servis ve İletişim:** Arka planda `NT AUTHORITY\SYSTEM` olarak koşan ve yerel bir MQTT broker (`127.0.0.1:1883`) barındıran .NET Framework 4.7.2 servisi (`GCUService.exe`).
- **Donanım Köprüsü:** `ACPIDriverDll.dll` üzerinden çağrılan KMDF çekirdek sürücüsü `UWACPIDriver.sys` ve onun ACPI seviyesindeki hedefi: `ACPI\INOU0000`.

---

## 3. IOCTL Dispatch Tablosu ve ACPI Köprüsünün Çözülmesi

Windows servisinin donanıma nasıl eriştiğini anlamak amacıyla `ACPIDriverDll.dll` ve `UWACPIDriver.sys` ikilileri disasm edildi. Sürücünün `\\.\ACPIDriver` sembolik linki üzerinden Windows `CTL_CODE` standardında (`DeviceType = 0x9C40`) IOCTL çağrıları karşıladığı belirlendi:

$$\text{IOCTL} = (\text{DeviceType} \ll 16) \mid (\text{RequiredAccess} \ll 14) \mid (\text{Function} \ll 2) \mid \text{TransferType}$$

| Fonksiyon Adı | Export No | IOCTL Kodu | Girdi / Çıktı | Açıklama |
| :--- | :--- | :--- | :--- | :--- |
| `ReadCMOS` | 1 | `0x9C40A480` | `u32` ofset $\rightarrow$ `u8` değer | CMOS RTC okuma |
| `WriteCMOS` | 14 | `0x9C40A484` | `[u32; 2]` `[reg, val]` | CMOS RTC yazma |
| `ReadEC` | 2 | `0x9C40A488` | `u32` ofset $\rightarrow$ `u8` değer | EC RAM'den 1 bayt okuma |
| `WriteEC` | 15 | `0x9C40A48C` | `[u32; 2]` `[reg, val]` | EC RAM'e 1 bayt yazma |
| `ReadIO` | 3 | `0x9C40A4C0` | `u16` port $\rightarrow$ `u8` değer | x86 I/O port okuma (`inb`) |
| `WriteIO` | 16 | `0x9C40A4C4` | `[u16; 1], [u8; 1]` | x86 I/O port yazma (`outb`) |
| `SMAPCTable` | 7 | `0x9C40A500` | 128 bayt girdi/çıktı | Smart APC güç senkronizasyonu |

### Kritik Keşif: Doğrudan ACPI Metot Çağrısı
`UWACPIDriver.sys` disasm edildiğinde, `ReadEC` fonksiyonunun ACPI değerlendirme arabelleği ile `\_SB.INOU.ECRR` metodunu, `WriteEC` fonksiyonunun ise `\_SB.INOU.ECRW` metodunu çalıştırdığı görüldü.

Bu bulgu, Linux üzerinde donanımla konuşmak için tescilli bir çekirdek sürücüsü yazma zorunluluğunu ortadan kaldırdı; çekirdeğe sağlanan `acpi_call` modülü (`/proc/acpi/call`) üzerinden aynı metotlar doğrudan yürütülebiliyordu.

---

## 4. Embedded Controller (EC) Register Haritasının Çıkarılması

`GCUService.exe` decompile edilerek `Define.ECSpec`, `Define.RamFan1p5_ECSpec` ve `Define.RamFan2_ECSpec` sınıfları çözümlendi. Bu işlem sonucunda anakart üzerindeki Embedded Controller'ın tam bellek haritası elde edildi:

### 4.1 Fan Denetimi ve Telemetri Ayrımı
- `0x075B` & `0x075C`: CPU ve GPU anlık fan çalışma yüzdeleri (`%0` - `%100` PWM).
- `0x0751`: **Ana Fan Mod Denetleyicisi:**
  - `0x00`: Otomatik BIOS kontrolü
  - `0x10`: Turbo modu
  - `0x40`: CoolerBoost (%100 tam devir kilidi)
  - `0x80`: Özel fan eğrisi (RamFan aktif)

> [!NOTE]
> **`0x046B` Adresindeki Anlamsal Ayrım ve Fan Telemetrisi:**  
> Windows PDB hata ayıklama sembollerinde (`Define.ECSpec`), `0x0464` CPU Fan RPM ve `0x046B` GPU Fan RPM olarak adlandırılmıştır. Ancak canlı Tongfang/Uniwill (`GM7AR0R`) donanımında yapılan telemetri analizinde, `0x046A` adresinin **Canlı PL1**, `0x046B` adresinin ise **Canlı PL2** güç limitleri (Watt) olduğu saptanmıştır. Canlı sistemde bu iki adres güç limitlerine tahsis edildiği için, projemizin Rust mimarisinde fan devir telemetrisi doğrudan EC RAM baytlarını okumak yerine çekirdeğin standart `/sys/class/hwmon` arayüzünden (`fan1_input`, `fan2_input`) okunacak şekilde tasarlanmıştır.

### 4.2 RamFan 11 Noktalı Fan Eğrisi Tabloları (`0x0F00` - `0x0F5F`)
Sistem, fan eğrilerini yüksek EC RAM bölgesinde tutuyordu:
- `0x0F00`: CPU sıcaklık artış tetikleyicileri (`UpT`, 11 bayt)
- `0x0F10`: CPU sıcaklık düşüş tetikleyicileri (`DownT` histerezis, 11 bayt)
- `0x0F20`: CPU fan PWM duty yüzdeleri (11 bayt)
- `0x0F30` - `0x0F50`: GPU için karşılık gelen 11'er baytlık tablolar
- `0x0F5F`: Tabloyu EC'ye uygulayan tetikleyici bayt

### 4.3 İşlemci ve Ekran Kartı Güç Limitleri
- `0x07AB`: Aktif güç modu indeksi (1: Oyun, 2: Ofis, 3: Pil Tasarrufu, 4: Turbo).
- `0x0783` (PL1), `0x0784` (PL2), `0x0785` (PL4): İşlemci statik BIOS profil limitleri (Watt).
- `0x0738` (Tau): PL1 süre penceresi (saniye).
- `0x0786` (TCC Offset): İşlemci termal kısılma ofseti (TjMax - ofset).
- `0x0743` - `0x0746`: Ekran kartı Configurable TGP ve Dynamic Boost güç hedefleri.

### 4.4 Klavye Arka Aydınlatması
- `0x0769`, `0x076A`, `0x076B`: Kırmızı, Yeşil, Mavi parlaklık seviyeleri (`0..50`).
- `0x078C`: Aydınlatma güç anahtarı ve parlaklık kademesi.
- `0x0767`: Yeni değerleri donanıma işleten mandallama baytı (Bit `0x20`).

---

## 5. İlk Canlı Doğrulama: `acpi_call` ve Python Prototipi

Çıkarılan adreslerin doğruluğunu test etmek amacıyla Linux tarafında `/proc/acpi/call` köprüsü kullanılarak çalışma dizininde yer alan `py_deneme` prototipi kapsamında minimal bir Python test betiği hazırlandı.

Betiğin çalışma adımları:
```python
# 1. Renk register'larına yaz:
write_ec(0x0769, red)    # Kırmızı PWM (0..50)
write_ec(0x076A, green)  # Yeşil PWM (0..50)
write_ec(0x076B, blue)   # Mavi PWM (0..50)

# 2. Parlaklık seviyesini belirle:
write_ec(0x078C, brightness)

# 3. Mandal bitini tetikle:
write_ec(0x0767, 0x20)

# 4. Donanımın rengi anında yenilemesi için re-arm darbesi:
write_ec(0x078C, 0x00)
time.sleep(0.04)
write_ec(0x078C, 0x01)
```

Bu test sonucunda klavye donanımının anında tepki verdiği, renklerin başarıyla değiştiği gözlendi. Ayrıca insan gözünün algısal duyarlılığına göre fotopik kalibrasyon çarpanları eklenerek ara renklerin (turuncu, camgöbeği, mor) doğru tonlarda yanması sağlandı.

### Test Sırasında Çürütülen Varsayımlar:
1. **USB HID Yanılgısı:** Windows kodlarındaki ITE IT829x USB HID kütüphanelerine rağmen, `lsusb` çıktısında aygıtın yer almadığı; Tulpar T7 V20.7'de klavyenin doğrudan EC'ye bağlı **Single-Zone PWM** bir LED grubu olduğu kanıtlandı.
2. **Fedora Kernel Uyumsuzluğu Efsanesi:** Fedora'da hazır araçların çalışmama sebebinin çekirdeğin donanımı desteklememesi değil; güvenlik nedeniyle `ec_sys` debugfs modülünün derlenmemiş olması olduğu anlaşıldı. ACPI ağacında `/sys/bus/acpi/devices/INOU0000:00/` düğümü tamamen faaldi.

---

## 6. Stratejik Karar: Sıfırdan Modül mü, Tuxedo Sürücüsünü Miras Almak mı?

Klavye ve EC erişimi başarıyla doğrulandıktan sonra önümüzde iki teknik yol vardı:
1. **Seçenek A:** Tüm EC mantığını ve ACPI yönetimini sıfırdan Rust tabanlı bir çekirdek sürücüsü veya userspace katmanı olarak sıfırdan yazmak.
2. **Seçenek B:** Tuxedo'nun 5 yılı aşkın süredir Uniwill ve Tongfang kasalar için geliştirdiği, yüzlerce kenar durumu (edge case), ACPI WMI olaylarını ve uyku/uyanma (suspend/resume) davranışlarını bizzat test ettiği açık kaynaklı `tuxedo-drivers` çekirdek modülünü alıp Monster için yamalamak.

### Karar ve Uygulanan Çözüm
Yeniden tekerlek icat etmek yerine, Tuxedo'nun 5 yıllık saha deneyimini miras almak tercih edildi. Sürücü kaynak kodundaki DMI filtreleme mekanizması çözümlenerek `tuxedo_compatibility_check.c` ve `clevo_leds.h` içerisine Monster donanım imzaları eklendi:

```c
// driver/tuxedo-drivers-monster-4.25.0/src/tuxedo_compatibility_check/tuxedo_compatibility_check.c
static const struct dmi_system_id tuxedo_dmi_string_match[] = {
    { .matches = { DMI_MATCH(DMI_SYS_VENDOR, "TUXEDO"), }, },
    { .matches = { DMI_MATCH(DMI_SYS_VENDOR, "MONSTER"), }, },
    { .matches = { DMI_BOARD_NAME, "GM7AR0R"), }, },
    { .matches = { DMI_BOARD_NAME, "TULPAR T7 V20.7"), }, },
    { .matches = { DMI_PRODUCT_SKU, "T7V207GM7AR0R"), }, },
    ...
};
```

Bu yama sayesinde `tuxedo_keyboard`, `uniwill_wmi` ve `tuxedo_io` çekirdek modülleri Monster Tulpar cihazında da standart Linux `sysfs` LED ve WMI arayüzlerini (`/sys/devices/platform/tuxedo_keyboard/`) sorunsuz şekilde ayağa kaldırdı.

---

## 7. Şasi Mod Butonu ve Dinamik Telemetri Düğümünün Çözülmesi

Donanımın canlı çalışması sırasında karşılaşılan en zorlu problem, kasa üzerindeki fiziksel mod butonu ve buton üzerindeki gösterge LED'iydi:

### 7.1 Otonom Donanım Durum Makinesi
Statik Windows kodları butona basıldığında işletim sistemine `0x00B0` ACPI scancode geleceğini belirtiyordu. Ancak Linux altında `acpi_listen` ve `/dev/input/event*` dinlemelerinde hiçbir scancode yakalanamadı. Buton basışının doğrudan EC firmware tarafından işletilen otonom bir donanım durum makinesi olduğu ve işletim sistemini beklemeden iç durumunu ilerlettiği keşfedildi.

### 7.2 Sönmeyen LED Problemi ve `0x0751` Adresi
Kasa butonuna basıldığında durumun `0x07AB` üzerinden değil, doğrudan `0x0751` (`ADDR_MAFAN_CONTROL_BYTE`) üzerinden değiştiği saptandı.

Önceki açık kaynaklı uygulamalar sessiz moda geçerken bu adrese `0x00` (Otomatik BIOS) yazıyordu. Fakat bu kasada `0x00` yazmak fanı otomatiğe alsa da buton LED'ini kapatmıyor, orta parlaklıkta (`0x20`) asılı bırakıyordu.

Yapılan canlı telemetri deneyleriyle LED'i tamamen söndürmek için bu adrese açıkça **`0xA0`** yazılması gerektiği kanıtlandı:

| Çalışma Modu | Buton LED Durumu | `0x0751` Değeri | Canlı PL1 (`0x046A`) | Canlı PL2 (`0x046B`) | Karakteristik |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Ofis / Sessiz** | **Tamamen Kapalı** | `0xA0` | `45 W` (`0x2D`) | `45 W` (`0x2D`) | Akustik öncelikli, LED sönük |
| **Oyun / Dengeli** | **Yarı Parlak (Tek LED)** | `0x20` | `75 W` (`0x4B`) | `75 W` (`0x4B`) | Dengeli termal zarf, tek LED |
| **Turbo / Canavar** | **Tam Parlak (Çift LED)**| `0x30` | `90 W` (`0x5A`) | `90 W` (`0x5A`) | Maksimum TDP, çift LED açık |

### 7.3 Dinamik Limitler (`0x046A` / `0x046B`) ve Hwmon Fan Okuması
Mod butonuna basıldığında EC'nin BIOS varsayılanlarını doğrudan `0x046A` (PL1) ve `0x046B` (PL2) canlı adreslerine kopyaladığı tespit edildi. 

Statik sembollerdeki `0x046B` GPU RPM tanımının aksine, canlı donanımda `0x046A` ve `0x046B` adresleri 8-bitlik anlık işlemci güç tavanını (45W, 75W, 90W) temsil eder. Nitekim Rust çekirdeğimizde (`performance.rs`) mod değişiminde doğrudan bu adreslere yazılmakta; fan RPM değerleri ise EC port çakışmalarını önlemek adına çekirdeğin `/sys/class/hwmon` arayüzünden (`find_hwmon_fans`) okunmaktadır.

---

## 8. Kendi Kodumuzu Yazmak: Tuxeren Markalaşması ve Modern Linux Mimarisi

Çalışma sadece tek bir Monster modeliyle sınırlı kalmayıp aynı ODM altyapısını paylaşan tüm dizüstü bilgisayarları kapsayacak şekilde genişletildi. Proje bu aşamada **Tuxeren Control Center (`txectrl`)** olarak markalaştırıldı.

```
[Kasa Fiziksel Butonu / UI Etkileşimi]
                 │
                 ▼
     [Rust Hardware Controller]
                 │
   ┌─────────────┴─────────────┐
   ▼                           ▼
[EC RAM & Sürücü Senkronu]   [Linux Çekirdek Entegrasyonu]
- 0x0751 (Fan & Mod LED)     - Intel RAPL powercap (/sys/class/powercap/intel-rapl)
- 0x046A / 0x046B (PL1/PL2)  - CPU cpufreq EPP (power / balance / performance)
- sysfs /dev/tuxedo_io       - D-Bus power-profiles-daemon senkronizasyonu
```

### 8.1 Mimari Bileşenler
1. **Rust Çekirdeği ve Donanım Yöneticisi:**
   - 200 ms aralıklarla `0x0751` adresini dinleyen ve CPU kullanımı %0.01'in altında olan donanım dinleyicisi.
   - Mod değiştiğinde Linux'un `intel-rapl` (`powercap`) arayüzüne mikrowatt seviyesinde sınır koyan ve `cpufreq/energy_performance_preference` politikasını güncelleyen senkronizasyon motoru.
2. **Tauri v2 Tabanlı Yerel Kullanıcı Arayüzü:**
   - Eski Tuxedo yazılımının hantal yapısı yerine, Rust backend üzerinde koşan hafif ve modern bir Tauri v2 arayüzü inşa edildi.
   - 11 renk önayarı, 2D HSV renk seçici, canlı telemetri paneli ve Türkçe/İngilizce çift dil desteği sağlandı.

### 8.2 Çekirdek Modülü Yaşam Döngüsü ve Otomatik Derleme Ayrımı
Projenin donanım katmanında yer alan iki farklı çekirdek modülü için yaşam döngüsü rolleri net olarak ayrıştırılmıştır:

- **Erken Prototipleme ve Ham EC Araştırması (`acpi_call`):**  
  İlk donanım keşiflerinde ve `/proc/acpi/call` üzerinden yapılan doğrudan EC RAM doğrulama testlerinde kullanılmıştır. Fedora kernel güncellemelerinde (`vermagic` uyumsuzluğunda) modül eksikliği algılandığında, yerel çekirdek başlıkları (`/lib/modules/$(uname -r)/build`) üzerinden `acpi_call.ko` ikilisinin doğrudan derlenmesi stratejisi uygulanmıştır.

- **Nihai Üretim Mimarisi (`tuxedo-drivers` & DKMS):**  
  Sistemin ana omurgasını oluşturan sürücüler (`tuxedo_keyboard`, `uniwill_wmi`, `tuxedo_io`) `deploy_driver.sh` betiği ve GUI arayüzündeki tek tıkla `pkexec` yetkilendirmesi üzerinden **DKMS (Dynamic Kernel Module Support)** altına kaydedilmektedir (`/usr/src/tuxedo-drivers-monster-4.25.0`). Çekirdek güncellendiğinde sistem DKMS mekanizması aracılığıyla modülleri otomatik olarak yeni çekirdeğe derlemekte; aynı betik `99-tuxeren-control-center.rules` udev kural dosyasını yazarak `/sys/devices/platform/tuxedo_keyboard` ve `/dev/tuxedo_io` için root gerektirmeyen kullanıcı izinlerini tesis etmektedir.

---

## 9. Sonuç ve Kazanımlar

Bu süreç sonucunda:
- Kapalı kaynaklı Windows kontrol merkezinin IOCTL ve EC seviyesindeki protokolleri tamamen çözümlendi.
- Tuxedo sürücülerinin Monster donanımlarında çalışmasını engelleyen DMI uyumluluk kısıtlaması yamalandı.
- Donanımın çalışma mekanizmasına uygun, Linux çekirdeği ile tam senkron çalışan, hafif ve modern bir donanım kontrol merkezi ([Tuxeren Control Center](README.md)) başarıyla hayata geçirildi.
