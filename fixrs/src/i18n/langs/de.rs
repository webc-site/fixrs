use crate::i18n::msg::I18n;

pub const DE: I18n = I18n {
  about: "CLI-Tool zum Ersetzen absoluter Rust-Pfade durch use-Anweisungen (behebt clippy::absolute_paths)",
  path: "Pfad zur zu verarbeitenden Datei oder zum Verzeichnis (sucht standardmäßig aufwärts nach Cargo.toml)",
  dry_run: "Testlauf-Modus: Vorschau ohne tatsächliche Dateiänderungen",
  write: "Dateien direkt ändern und rustfmt ausführen (Standard)",
  check: "Prüfmodus: Beendet mit Fehlercode, falls Pfade das Limit überschreiten (für CI)",
  max_segments: "Maximal zulässige Pfadsegmente vor Vereinfachung (Standard: 2)",
  keep_segments: "Anzahl der beizubehaltenden Endsegmente (Standard: 1; 2 für module::item)",
  allow_crate: "Whitelist von Crates, die absolute Pfade behalten dürfen",
  extra_crate: "Explizit angegebene zusätzliche bekannte Crates",
  quiet: "Stiller Modus, unterdrückt Änderungsdetails",
  show: "Anzeigemodus, erzwingt die Ausgabe von Änderungsdetails",
  verbose: "Ausführliche Verarbeitungsprotokolle ausgeben",
  no_cache: "Inkrementellen Cache deaktivieren und vollständige Neuprüfung erzwingen",

  found_exceeding_limit: |count| format!("{count} Datei(en) mit unzulässig langen Pfaden gefunden"),
  can_be_simplified: |count| {
    format!(
      "\n{count} Datei(en) können vereinfacht werden, ohne `--dry-run` ausführen zum Anwenden"
    )
  },
  failed_init_runtime: "Fehler beim Initialisieren der compio-Laufzeitumgebung",
  error_processing_file: |file, err| {
    format!("[warn] Fehler beim Verarbeiten von {file}: {err}, übersprungen")
  },
};
