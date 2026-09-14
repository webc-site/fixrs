mod de;
mod en;
mod es;
mod fr;
mod it;
mod ja;
mod ko;
mod pt;
mod ru;
mod zh;

pub use de::DE;
pub use en::EN;
pub use es::ES;
pub use fr::FR;
pub use it::IT;
pub use ja::JA;
pub use ko::KO;
pub use pt::PT;
pub use ru::RU;
pub use zh::ZH;

use crate::i18n::msg::I18n;

// 其它主流国际语言（编译期常量聚合，无句号规范）

pub const AR: I18n = I18n {
  about: "أداة لاستبدال المسارات المؤهلة في Rust بعبارات use، وإصلاح clippy::absolute_paths",
  path: "مسار الملف أو الدليل المراد معالجته (يبحث تلقائيًا عن Cargo.toml لأعلى)",
  dry_run: "وضع المعاينة دون تعديل الملفات فعليًا",
  write: "تعديل الملفات مباشرة وتشغيل rustfmt تلقائيًا (افتراضي)",
  check: "وضع التحقق: الإنهاء برمز غير صفري إذا تجاوزت المسارات الحد المسموح به",
  max_segments: "الحد الأقصى المسموح به لأجزاء المسار قبل التبسيط (افتراضي: 2)",
  keep_segments: "عدد الأجزاء النهائية المراد الاحتفاظ بها (افتراضي: 1)",
  allow_crate: "القائمة البيضاء للحزم المسموح لها بالاحتفاظ بمسارات مطلقة",
  extra_crate: "حزم معروفة إضافية محددة صراحة",
  quiet: "الوضع الصامت، كتم تفاصيل التعديلات",
  show: "وضع العرض، فرض إخراج تفاصيل التعديل",
  verbose: "إخراج سجلات المعالجة التفصيلية",
  no_cache: "تعطيل ذاكرة التخزين المؤقت وإعادة الفحص الكامل",

  found_exceeding_limit: |count| {
    format!("تم العثور على {count} ملف(ات) تتجاوز الحد المسموح به للمسارات")
  },
  can_be_simplified: |count| {
    format!("\nيمكن تبسيط {count} ملف(ات)، قم بالتشغيل بدون `--dry-run` لتطبيق التغييرات")
  },
  failed_init_runtime: "فشل في تهيئة بيئة تشغيل compio",
  error_processing_file: |file, err| format!("[warn] خطأ أثناء معالجة {file}: {err}، تم التخطي"),
};

pub const HI: I18n = I18n {
  about: "Rust में पूर्ण पथों को use कथनों से बदलने और clippy::absolute_paths को ठीक करने का CLI टूल",
  path: "जांच या संसाधित करने के लिए फ़ाइल या निर्देशिका पथ (डिफ़ॉल्ट रूप से ऊपर Cargo.toml खोजता है)",
  dry_run: "फ़ाइलों को संशोधित किए बिना केवल पूर्वावलोकन मोड",
  write: "फ़ाइलों को सीधे संशोधित करें और स्वतः rustfmt चलाएं (डिफ़ॉल्ट)",
  check: "जांच मोड: यदि पथ सीमा से अधिक हैं तो गैर-शून्य कोड के साथ बाहर निकलें",
  max_segments: "सरलीकरण से पहले अधिकतम अनुमत पथ खंड (डिफ़ॉल्ट: 2)",
  keep_segments: "रखे जाने वाले अंतिम खंडों की संख्या (डिफ़ॉल्ट: 1)",
  allow_crate: "पूर्ण पथ बनाए रखने के लिए अनुमत क्रेट्स की श्वेतसूची",
  extra_crate: "स्पष्ट रूप से निर्दिष्ट अतिरिक्त ज्ञात क्रेट्स",
  quiet: "शांत मोड, संशोधन विवरण छुपाएं",
  show: "प्रदर्शन मोड, संशोधन विवरण प्रदर्शित करने के लिए बाध्य करें",
  verbose: "विस्तृत प्रोसेसिंग लॉग दिखाएं",
  no_cache: "इंक्रीमेंटल कैश अक्षम करें और पूर्ण पुनः जांच करें",

  found_exceeding_limit: |count| format!("सीमा से अधिक पथों वाली {count} फ़ाइलें मिलीं"),
  can_be_simplified: |count| {
    format!(
      "\n{count} फ़ाइलों को सरल बनाया जा सकता है, परिवर्तनों को लागू करने के लिए बिना `--dry-run` के चलाएं"
    )
  },
  failed_init_runtime: "compio रनटाइम प्रारंभ करने में विफल",
  error_processing_file: |file, err| {
    format!("[warn] फ़ाइल {file} को संसाधित करने में त्रुटि: {err}, छोड़ दिया गया")
  },
};

pub const VI: I18n = I18n {
  about: "Công cụ dòng lệnh thay thế đường dẫn tuyệt đối trong Rust bằng use, sửa clippy::absolute_paths",
  path: "Đường dẫn tệp hoặc thư mục cần xử lý (mặc định tự động tìm Cargo.toml lên trên)",
  dry_run: "Chế độ chạy thử, chỉ xem trước mà không sửa tệp thực tế",
  write: "Sửa đổi tệp trực tiếp và tự động chạy rustfmt (mặc định)",
  check: "Chế độ kiểm tra: thoát với mã khác 0 nếu phát hiện đường dẫn vượt quá giới hạn",
  max_segments: "Số phân đoạn đường dẫn tối đa cho phép trước khi rút gọn (mặc định: 2)",
  keep_segments: "Số phân đoạn cuối cần giữ lại (mặc định: 1; đặt 2 để giữ module::item)",
  allow_crate: "Danh sách trắng các crate được phép giữ đường dẫn tuyệt đối",
  extra_crate: "Các crate đã biết bổ sung được chỉ định rõ ràng",
  quiet: "Chế độ yên lặng, không xuất chi tiết chỉnh sửa",
  show: "Chế độ hiển thị, buộc xuất chi tiết chỉnh sửa",
  verbose: "Xuất nhật ký xử lý chi tiết",
  no_cache: "Vô hiệu hóa bộ nhớ đệm và buộc kiểm tra lại toàn bộ",

  found_exceeding_limit: |count| format!("Đã tìm thấy {count} tệp có đường dẫn vượt quá giới hạn"),
  can_be_simplified: |count| {
    format!("\nCó thể rút gọn {count} tệp, chạy không có `--dry-run` để áp dụng thay đổi")
  },
  failed_init_runtime: "Không thể khởi tạo môi trường thực thi compio",
  error_processing_file: |file, err| format!("[warn] Lỗi khi xử lý {file}: {err}, đã bỏ qua"),
};

pub const TH: I18n = I18n {
  about: "เครื่องมือ CLI สำหรับแทนที่พาธแบบเต็มใน Rust ด้วยคำสั่ง use และแก้ไข clippy::absolute_paths",
  path: "พาธของไฟล์หรือไดเรกทอรีที่ต้องการประมวลผล (ค้นหา Cargo.toml ย้อนขึ้นไปโดยค่าเริ่มต้น)",
  dry_run: "โหมดทดลองใช้เพื่อดูตัวอย่างโดยไม่แก้ไขไฟล์จริง",
  write: "แก้ไขไฟล์โดยตรงและรัน rustfmt อัตโนมัติ (ค่าเริ่มต้น)",
  check: "โหมดตรวจสอบ: ออกด้วยรหัสข้อผิดพลาดหากพบพาธที่เกินขีดจำกัด",
  max_segments: "จำนวนส่วนพาธสูงสุดที่อนุญาตก่อนย่อให้สั้นลง (ค่าเริ่มต้น: 2)",
  keep_segments: "จำนวนส่วนพาธท้ายสุดที่ต้องการคงไว้ (ค่าเริ่มต้น: 1)",
  allow_crate: "ไวท์ลิสต์ของ crate ที่อนุญาตให้ใช้พาธแบบเต็มได้",
  extra_crate: "crate เพิ่มเติมที่ระบุไว้อย่างชัดเจน",
  quiet: "โหมดเงียบ ซ่อนรายละเอียดการแก้ไข",
  show: "โหมดแสดงผล บังคับแสดงรายละเอียดการแก้ไข",
  verbose: "แสดงบันทึกการประมวลผลโดยละเอียด",
  no_cache: "ปิดใช้งานแคชและบังคับตรวจสอบใหม่ทั้งหมด",

  found_exceeding_limit: |count| format!("พบ {count} ไฟล์ที่มีพาธเกินขีดจำกัด"),
  can_be_simplified: |count| {
    format!("\nสามารถย่อพาธได้ {count} ไฟล์ รันโดยไม่ใส่ `--dry-run` เพื่อใช้การเปลี่ยนแปลง")
  },
  failed_init_runtime: "การเตรียมพร้อมการทำงานของ compio ล้มเหลว",
  error_processing_file: |file, err| format!("[warn] เกิดข้อผิดพลาดขณะประมวลผล {file}: {err} ข้ามไป"),
};

pub const TR: I18n = I18n {
  about: "Rust kaynak kodundaki uzun mutlak yolları use bildirimleriyle değiştiren ve clippy::absolute_paths kuralını düzelten CLI aracı",
  path: "İşlenecek dosya veya dizin yolu (varsayılan olarak yukarı doğru Cargo.toml arar)",
  dry_run: "Dosyaları değiştirmeden önizleme yapan deneme modu",
  write: "Dosyaları yerinde değiştir ve rustfmt çalıştır (varsayılan)",
  check: "Kontrol modu: sınırı aşan yol bulunursa sıfır olmayan kodla çık (CI için)",
  max_segments: "Basitleştirmeden önce izin verilen maksimum yol segmenti (varsayılan: 2)",
  keep_segments: "Basitleştirmeden sonra tutulacak son segment sayısı (varsayılan: 1)",
  allow_crate: "Mutlak yolları korumasına izin verilen paketlerin beyaz listesi",
  extra_crate: "Açıkça belirtilen ek bilinen paketler",
  quiet: "Sessiz mod, değişiklik ayrıntılarını gizle",
  show: "Görüntüleme modu, değişiklik ayrıntılarını zorla göster",
  verbose: "Ayrıntılı işlem günlüklerini göster",
  no_cache: "Artımlı önbelleği devre dışı bırak ve tam yeniden denetim yap",

  found_exceeding_limit: |count| format!("Sınırı aşan yollara sahip {count} dosya bulundu"),
  can_be_simplified: |count| {
    format!(
      "\n{count} dosya basitleştirilebilir, değişiklikleri uygulamak için `--dry-run` olmadan çalıştırın"
    )
  },
  failed_init_runtime: "compio çalışma zamanı başlatılamadı",
  error_processing_file: |file, err| {
    format!("[warn] {file} işlenirken hata oluştu: {err}, atlandı")
  },
};

pub const PL: I18n = I18n {
  about: "Narzędzie CLI do zastępowania pełnych ścieżek Rust instrukcjami use (naprawia clippy::absolute_paths)",
  path: "Ścieżka do pliku lub katalogu (domyślnie wyszukuje Cargo.toml w górę)",
  dry_run: "Tryb próbny bez modyfikowania rzeczywistych plików",
  write: "Modyfikuj pliki w miejscu i uruchamiaj rustfmt (domyślnie)",
  check: "Tryb sprawdzania: zakończ z kodem błędu, jeśli ścieżki przekraczają limit (dla CI)",
  max_segments: "Maksymalna dozwolona liczba segmentów ścieżki (domyślnie: 2)",
  keep_segments: "Liczba końcowych segmentów do zachowania (domyślnie: 1)",
  allow_crate: "Biała lista skrzynek (crates), które mogą zachować pełne ścieżki",
  extra_crate: "Dodatkowe znane skrzynki określone jawnie",
  quiet: "Tryb cichy, ukrywa szczegóły modyfikacji",
  show: "Tryb wyświetlania, wymusza wypisanie szczegółów modyfikacji",
  verbose: "Szczegółowe dzienniki przetwarzania",
  no_cache: "Wyłącz pamięć podręczną i wymuś pełną ponowną weryfikację",

  found_exceeding_limit: |count| {
    format!("Znaleziono {count} plik(ów) ze ścieżkami przekraczającymi limit")
  },
  can_be_simplified: |count| {
    format!("\nMożna uprościć {count} plik(ów), uruchom bez `--dry-run`, aby zastosować zmiany")
  },
  failed_init_runtime: "Nie udało się zainicjować środowiska wykonawczego compio",
  error_processing_file: |file, err| {
    format!("[warn] Błąd podczas przetwarzania {file}: {err}, pomijanie")
  },
};

pub const NL: I18n = I18n {
  about: "CLI-tool om absolute Rust-paden te vervangen door use-instructies (herstelt clippy::absolute_paths)",
  path: "Pad naar bestand of map om te verwerken (zoekt standaard naar boven naar Cargo.toml)",
  dry_run: "Voorbeeldmodus zonder bestanden daadwerkelijk te wijzigen",
  write: "Wijzig bestanden ter plaatse en voer rustfmt uit (standaard)",
  check: "Controlemodus: sluit af met foutcode als paden de limiet overschrijden (voor CI)",
  max_segments: "Maximaal toegestane padsegmenten vóór vereenvoudiging (standaard: 2)",
  keep_segments: "Aantal te behouden eindsegmenten (standaard: 1)",
  allow_crate: "Witte lijst van crates die absolute paden mogen behouden",
  extra_crate: "Expliciet opgegeven extra bekende crates",
  quiet: "Stille modus, verbergt wijzigingsdetails",
  show: "Weergavemodus, forceert het tonen van wijzigingsdetails",
  verbose: "Gedetailleerde verwerkingslogboeken weergeven",
  no_cache: "Incrementele cache uitschakelen en volledige hercontrole forceren",

  found_exceeding_limit: |count| {
    format!("{count} bestand(en) gevonden met paden die de limiet overschrijden")
  },
  can_be_simplified: |count| {
    format!(
      "\n{count} bestand(en) kunnen worden vereenvoudigd, voer uit zonder `--dry-run` om toe te passen"
    )
  },
  failed_init_runtime: "Initialiseren van compio-runtime mislukt",
  error_processing_file: |file, err| {
    format!("[warn] Fout bij verwerken van {file}: {err}, overgeslagen")
  },
};

pub const ID: I18n = I18n {
  about: "Alat CLI untuk mengganti jalur absolut Rust dengan pernyataan use, memperbaiki clippy::absolute_paths otomatis",
  path: "Jalur berkas atau direktori yang akan diproses (mencari Cargo.toml ke atas secara default)",
  dry_run: "Mode uji coba pratinjau tanpa memodifikasi berkas sebenarnya",
  write: "Ubah berkas secara langsung dan jalankan rustfmt (default)",
  check: "Mode pemeriksaan: keluar dengan status bukan nol jika jalur melebihi batas (untuk CI)",
  max_segments: "Segmen jalur maksimum yang diizinkan sebelum penyederhanaan (default: 2)",
  keep_segments: "Jumlah segmen akhir yang dipertahankan (default: 1; gunakan 2 untuk module::item)",
  allow_crate: "Daftar putih crate yang diizinkan mempertahankan jalur absolut",
  extra_crate: "Crate tambahan yang diketahui dan ditentukan secara eksplisit",
  quiet: "Mode hening, sembunyikan rincian modifikasi",
  show: "Mode tampilan, paksa menampilkan rincian modifikasi",
  verbose: "Tampilkan log pemrosesan terperinci",
  no_cache: "Nonaktifkan cache inkremental dan paksa pemeriksaan ulang penuh",

  found_exceeding_limit: |count| {
    format!("Ditemukan {count} berkas dengan jalur yang melebihi batas")
  },
  can_be_simplified: |count| {
    format!(
      "\n{count} berkas dapat disederhanakan, jalankan tanpa `--dry-run` untuk menerapkan perubahan"
    )
  },
  failed_init_runtime: "Gagal menginisialisasi runtime compio",
  error_processing_file: |file, err| {
    format!("[warn] Terjadi kesalahan saat memproses {file}: {err}, dilewati")
  },
};

pub const UK: I18n = I18n {
  about: "Утиліта CLI для заміни абсолютних шляхів Rust на use, автоматичне виправлення clippy::absolute_paths",
  path: "Шлях до файлу або каталогу (за замовчуванням шукає Cargo.toml вгору)",
  dry_run: "Режим попереднього перегляду без фактичної зміни файлів",
  write: "Змінювати файли на місці та запускати rustfmt (за замовчуванням)",
  check: "Режим перевірки: вихід із ненульовим кодом у разі перевищення ліміту шляхів (для CI)",
  max_segments: "Максимальна дозволена кількість сегментів шляху (за замовчуванням: 2)",
  keep_segments: "Кількість кінцевих сегментів для збереження (за замовчуванням: 1)",
  allow_crate: "Білий список крейтів, яким дозволено зберігати абсолютні шляхи",
  extra_crate: "Явно вказані додаткові відомі крейти",
  quiet: "Тихий режим, приховує деталі змін",
  show: "Режим показу, примусово виводить деталі змін",
  verbose: "Докладні журнали обробки",
  no_cache: "Вимкнути інкрементний кеш і примусово перевірити все наново",

  found_exceeding_limit: |count| {
    format!("Знайдено {count} файл(ів) із перевищенням довжини шляхів")
  },
  can_be_simplified: |count| {
    format!("\nМожна спростити {count} файл(ів), запустіть без `--dry-run` для застосування змін")
  },
  failed_init_runtime: "Не вдалося ініціалізувати середовище compio",
  error_processing_file: |file, err| format!("[warn] Помилка обробки {file}: {err}, пропущено"),
};

pub const CS: I18n = I18n {
  about: "Nástroj CLI pro nahrazení absolutních cest v Rustu příkazy use (opravuje clippy::absolute_paths)",
  path: "Cesta k souboru nebo adresáři (ve výchozím nastavení hledá Cargo.toml směrem nahoru)",
  dry_run: "Režim náhledu bez skutečné úpravy souborů",
  write: "Upravit soubory na místě a spustit rustfmt (výchozí)",
  check: "Režim kontroly: ukončit s nenulovým kódem, pokud cesty překročí limit (pro CI)",
  max_segments: "Maximální povolený počet segmentů cesty před zjednodušením (výchozí: 2)",
  keep_segments: "Počet koncových segmentů k zachování (výchozí: 1)",
  allow_crate: "Seznam povolených beden (crates), které si mohou ponechat absolutní cesty",
  extra_crate: "Explicitně zadané další známé crates",
  quiet: "Tichý režim, potlačí podrobnosti o úpravách",
  show: "Režim zobrazení, vynutí výpis podrobností o úpravách",
  verbose: "Zobrazit podrobné protokoly zpracování",
  no_cache: "Zakázat přírůstkovou mezipaměť a vynutit úplnou novou kontrolu",

  found_exceeding_limit: |count| format!("Nalezeno {count} souborů s cestami překračujícími limit"),
  can_be_simplified: |count| {
    format!("\nLze zjednodušit {count} souborů, spusťte bez `--dry-run` pro aplikaci změn")
  },
  failed_init_runtime: "Inicializace modulu runtime compio se nezdařila",
  error_processing_file: |file, err| {
    format!("[warn] Chyba při zpracování {file}: {err}, přeskočeno")
  },
};

pub const SV: I18n = I18n {
  about: "CLI-verktyg för att ersätta absoluta Rust-sökvägar med use-satser (åtgärdar clippy::absolute_paths)",
  path: "Sökväg till fil eller katalog som ska bearbetas (söker uppåt efter Cargo.toml som standard)",
  dry_run: "Förhandsgranskningsläge utan att faktiskt ändra filer",
  write: "Ändra filer på plats och kör rustfmt automatiskt (standard)",
  check: "Kontrolläge: avsluta med felkod om sökvägar överskrider gränsen (för CI)",
  max_segments: "Maximalt antal tillåtna sökvägssegment före förenkling (standard: 2)",
  keep_segments: "Antal avslutande segment att behålla (standard: 1)",
  allow_crate: "Vitlista över crates som tillåts behålla absoluta sökvägar",
  extra_crate: "Ytterligare kända crates som specificerats uttryckligen",
  quiet: "Tyst läge, döljer ändringsdetaljer",
  show: "Visningsläge, tvingar utskrift av ändringsdetaljer",
  verbose: "Visa detaljerade bearbetningsloggar",
  no_cache: "Inaktivera inkrementell cache och tvinga fullständig omkontroll",

  found_exceeding_limit: |count| {
    format!("Hittade {count} fil(er) med sökvägar som överskrider gränsen")
  },
  can_be_simplified: |count| {
    format!("\n{count} fil(er) kan förenklas, kör utan `--dry-run` för att tillämpa ändringar")
  },
  failed_init_runtime: "Kunde inte initiera compio-körtid",
  error_processing_file: |file, err| {
    format!("[warn] Fel vid bearbetning av {file}: {err}, hoppar över")
  },
};

pub const EL: I18n = I18n {
  about: "Εργαλείο CLI για την αντικατάσταση απόλυτων διαδρομών Rust με δηλώσεις use (διορθώνει clippy::absolute_paths)",
  path: "Διαδρομή αρχείου ή καταλόγου προς επεξεργασία (αναζητά Cargo.toml προς τα πάνω)",
  dry_run: "Λειτουργία προεπισκόπησης χωρίς τροποποίηση των πραγματικών αρχείων",
  write: "Τροποποίηση αρχείων επί τόπου και αυτόματη εκτέλεση rustfmt (προεπιλογή)",
  check: "Λειτουργία ελέγχου: έξοδος με μη μηδενικό κωδικό εάν οι διαδρομές υπερβαίνουν το όριο",
  max_segments: "Μέγιστα επιτρεπόμενα τμήματα διαδρομής πριν από την απλοποίηση (προεπιλογή: 2)",
  keep_segments: "Αριθμός τελικών τμημάτων προς διατήρηση (προεπιλογή: 1)",
  allow_crate: "Λίστα επιτρεπόμενων crates για διατήρηση απόλυτων διαδρομών",
  extra_crate: "Επιπλέον γνωστά crates που καθορίζονται ρητά",
  quiet: "Αθόρυβη λειτουργία, απόκρυψη λεπτομερειών τροποποίησης",
  show: "Λειτουργία εμφάνισης, επιβολή εμφάνισης λεπτομερειών τροποποίησης",
  verbose: "Εμφάνιση λεπτομερών καταγραφών επεξεργασίας",
  no_cache: "Απενεργοποίηση σταδιακής προσωρινής μνήμης και πλήρης επανέλεγχος",

  found_exceeding_limit: |count| {
    format!("Βρέθηκαν {count} αρχεία με διαδρομές που υπερβαίνουν το όριο")
  },
  can_be_simplified: |count| {
    format!("\n{count} αρχεία μπορούν να απλοποιηθούν, εκτελέστε χωρίς `--dry-run` για εφαρμογή")
  },
  failed_init_runtime: "Αποτυχία προετοιμασίας του περιβάλλοντος εκτέλεσης compio",
  error_processing_file: |file, err| {
    format!("[warn] Σφάλμα κατά την επεξεργασία του {file}: {err}, παράβλεψη")
  },
};

pub const HE: I18n = I18n {
  about: "כלי CLI להחלפת נתיבים מלאים ב-Rust בהצהרות use, ותיקון clippy::absolute_paths",
  path: "נתיב לקובץ או לתיקייה לעיבוד (מחפש Cargo.toml כלפי מעלה כברירת מחדל)",
  dry_run: "מצב תצוגה מקדימה ללא שינוי קבצים בפועל",
  write: "עריכת קבצים במקום והפעלת rustfmt אוטומטית (ברירת מחדל)",
  check: "מצב בדיקה: יציאה עם קוד שגיאה אם יש נתיבים החורגים מהמגבלה",
  max_segments: "מספר מקטעי נתיב מרבי מותר לפני פישוט (ברירת מחדל: 2)",
  keep_segments: "מספר מקטעים סופיים שיש לשמור (ברירת מחדל: 1)",
  allow_crate: "רשימת crates מורשים לשמירה על נתיבים מלאים",
  extra_crate: "crates נוספים שצוינו במפורש",
  quiet: "מצב שקט, הסתרת פרטי שינויים",
  show: "מצב תצוגה, הצגת פרטי שינויים באופן מאולץ",
  verbose: "הצגת יומני עיבוד מפורטים",
  no_cache: "השבתת מטמון ובדיקה מלאה מחדש",

  found_exceeding_limit: |count| format!("נמצאו {count} קבצים עם נתיבים החורגים מהמגבלה"),
  can_be_simplified: |count| {
    format!("\nניתן לפשט {count} קבצים, הפעל ללא `--dry-run` כדי להחיל שינויים")
  },
  failed_init_runtime: "אתחול סביבת compio נכשל",
  error_processing_file: |file, err| format!("[warn] שגיאה בעיבוד {file}: {err}, מדלג"),
};

pub const RO: I18n = I18n {
  about: "Instrument CLI pentru înlocuirea căilor absolute din Rust cu instrucțiuni use (repară clippy::absolute_paths)",
  path: "Calea către fișierul sau directorul de procesat (caută Cargo.toml în sus în mod implicit)",
  dry_run: "Mod previzualizare fără modificarea fișierelor",
  write: "Modifică fișierele pe loc și rulează automat rustfmt (implicit)",
  check: "Mod verificare: ieșire cu cod diferit de zero dacă se depășește limita (pentru CI)",
  max_segments: "Segmente maxime de cale permise înainte de simplificare (implicit: 2)",
  keep_segments: "Număr de segmente finale de păstrat (implicit: 1)",
  allow_crate: "Lista albă a pachetelor cărora li se permite păstrarea căilor absolute",
  extra_crate: "Pachete suplimentare specificate explicit",
  quiet: "Mod silențios, ascunde detaliile modificărilor",
  show: "Mod afișare, forțează afișarea detaliilor modificărilor",
  verbose: "Afișează jurnale detaliate de procesare",
  no_cache: "Dezactivează memoria cache incrementală și forțează reverificarea completă",

  found_exceeding_limit: |count| {
    format!("Au fost găsite {count} fișiere cu căi ce depășesc limita")
  },
  can_be_simplified: |count| {
    format!("\n{count} fișiere pot fi simplificate, rulați fără `--dry-run` pentru a aplica")
  },
  failed_init_runtime: "Inițializarea runtime-ului compio a eșuat",
  error_processing_file: |file, err| format!("[warn] Eroare la procesarea {file}: {err}, se omite"),
};

pub const HU: I18n = I18n {
  about: "CLI eszköz a Rust abszolút útvonalak use utasításokra cserélésére (javítja a clippy::absolute_paths hibát)",
  path: "A feldolgozandó fájl vagy könyvtár útvonala (alapértelmezés szerint felfelé keresi a Cargo.toml fájlt)",
  dry_run: "Előnézeti mód a fájlok tényleges módosítása nélkül",
  write: "Fájlok helyben módosítása és a rustfmt automatikus futtatása (alapértelmezett)",
  check: "Ellenőrző mód: kilépés hibakóddal, ha az útvonalak túllépik a korlátot (CI-hez)",
  max_segments: "A simplification előtt engedélyezett maximális útvonalszegmensek száma (alapértelmezett: 2)",
  keep_segments: "A megőrizendő záró szegmensek száma (alapértelmezett: 1)",
  allow_crate: "Azon csomagok fehérlistája, amelyek megtarthatják az abszolút útvonalakat",
  extra_crate: "Kifejezetten megadott további ismert csomagok",
  quiet: "Csendes mód, módosítási részletek elrejtése",
  show: "Megjelenítési mód, módosítási részletek kényszerített kiírása",
  verbose: "Részletes feldolgozási naplók megjelenítése",
  no_cache: "Növekményes gyorsítótár letiltása és teljes újravizsgálat kényszerítése",

  found_exceeding_limit: |count| {
    format!("{count} korlátot meghaladó útvonalat tartalmazó fájl található")
  },
  can_be_simplified: |count| {
    format!(
      "\n{count} fájl egyszerűsíthető, futtassa a `--dry-run` nélkül a módosítások alkalmazásához"
    )
  },
  failed_init_runtime: "A compio futtatókörnyezet inicializálása sikertelen",
  error_processing_file: |file, err| {
    format!("[warn] Hiba a(z) {file} feldolgozásakor: {err}, kihagyva")
  },
};

pub const DA: I18n = I18n {
  about: "CLI-værktøj til at erstatte absolutte Rust-stier med use-sætninger (retter clippy::absolute_paths)",
  path: "Sti til fil eller mappe, der skal behandles (søger opad efter Cargo.toml som standard)",
  dry_run: "Forhåndsvisningstilstand uden at ændre filer",
  write: "Rediger filer direkte og kør rustfmt automatisk (standard)",
  check: "Kontroltilstand: afslut med fejlkode, hvis stier overskrider grænsen (til CI)",
  max_segments: "Maksimalt tilladte stisegmenter før forenkling (standard: 2)",
  keep_segments: "Antal afsluttende segmenter, der skal bevares (standard: 1)",
  allow_crate: "Hvidliste over crates, der må beholde absolutte stier",
  extra_crate: "Eksplicit angivne ekstra kendte crates",
  quiet: "Stille tilstand, skjul ændringsdetaljer",
  show: "Visningstilstand, gennemtving udskrift af ændringsdetaljer",
  verbose: "Vis detaljerede behandlingslogger",
  no_cache: "Deaktiver trinvis cache og gennemtving fuld genkontrol",

  found_exceeding_limit: |count| {
    format!("Fandt {count} fil(er) med stier, der overskrider grænsen")
  },
  can_be_simplified: |count| {
    format!("\n{count} fil(er) kan forenkles, kør uden `--dry-run` for at anvende ændringer")
  },
  failed_init_runtime: "Kunne ikke initialisere compio-køretid",
  error_processing_file: |file, err| {
    format!("[warn] Fejl under behandling af {file}: {err}, springer over")
  },
};

pub const FI: I18n = I18n {
  about: "CLI-työkalu Rustin absoluuttisten polkujen korvaamiseen use-lauseilla (korjaa clippy::absolute_paths)",
  path: "Käsiteltävän tiedoston tai hakemiston polku (etsii oletuksena Cargo.toml-tiedostoa ylöspäin)",
  dry_run: "Esikatselutila muuttamatta tiedostoja todellisuudessa",
  write: "Muokkaa tiedostoja suoraan ja suorita rustfmt automaattisesti (oletus)",
  check: "Tarkistustila: poistu virhekoodilla, jos polut ylittävät rajan (CI-käyttöön)",
  max_segments: "Suurin sallittu polkusegmenttien määrä ennen yksinkertaistamista (oletus: 2)",
  keep_segments: "Säilytettävien loppusegmenttien määrä (oletus: 1)",
  allow_crate: "Sallittujen pakettien luettelo, jotka voivat säilyttää absoluuttiset polut",
  extra_crate: "Erikseen määritellyt tunnetut lisäpaketit",
  quiet: "Hiljainen tila, piilota muutosten tiedot",
  show: "Näyttötila, pakota muutosten tietojen tulostaminen",
  verbose: "Näytä yksityiskohtaiset käsittelylokit",
  no_cache: "Poista välimuisti käytöstä ja suorita täysi uudelleentarkistus",

  found_exceeding_limit: |count| {
    format!("Löytyi {count} tiedosto(a), joiden polut ylittävät rajan")
  },
  can_be_simplified: |count| {
    format!(
      "\n{count} tiedosto(a) voidaan yksinkertaistaa, suorita ilman `--dry-run`-valitsinta muutosten ottamiseksi käyttöön"
    )
  },
  failed_init_runtime: "compio-suoritusympäristön alustaminen epäonnistui",
  error_processing_file: |file, err| {
    format!("[warn] Virhe käsiteltäessä tiedostoa {file}: {err}, ohitetaan")
  },
};

pub const NO: I18n = I18n {
  about: "CLI-verktøy for å erstatte absolutte Rust-stier med use-setninger (fikser clippy::absolute_paths)",
  path: "Bane til fil eller mappe som skal behandles (søker oppover etter Cargo.toml som standard)",
  dry_run: "Forhåndsvisningsmodus uten å endre filer",
  write: "Endre filer direkte og kjør rustfmt automatisk (standard)",
  check: "Kontrollmodus: avslutt med feilkode hvis stier overskrider grensen (for CI)",
  max_segments: "Maksimalt tillatte stisegmenter før forenkling (standard: 2)",
  keep_segments: "Antall avsluttende segmenter som skal beholdes (standard: 1)",
  allow_crate: "Hviteliste over crates som tillates å beholde absolutte stier",
  extra_crate: "Ekstra kjente crates spesifisert eksplisitt",
  quiet: "Stille modus, skjul endringsdetaljer",
  show: "Visningsmodus, tving utskrift av endringsdetaljer",
  verbose: "Vis detaljerte behandlingslogger",
  no_cache: "Deaktiver trinnvis hurtigbuffer og tving full ny sjekk",

  found_exceeding_limit: |count| format!("Fant {count} fil(er) med stier som overskrider grensen"),
  can_be_simplified: |count| {
    format!("\n{count} fil(er) kan forenkles, kjør uten `--dry-run` for å ta i bruk endringer")
  },
  failed_init_runtime: "Kunne ikke initialisere compio-kjøretid",
  error_processing_file: |file, err| {
    format!("[warn] Feil under behandling av {file}: {err}, hopper over")
  },
};
