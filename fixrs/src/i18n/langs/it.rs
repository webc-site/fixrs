use crate::i18n::msg::I18n;

pub const IT: I18n = I18n {
  about: "Strumento CLI per sostituire i percorsi assoluti Rust con use (corregge clippy::absolute_paths)",
  path: "Percorso del file o della directory da elaborare (cerca Cargo.toml verso l'alto per impostazione predefinita)",
  dry_run: "Modalità di prova senza modificare i file effettivi",
  write: "Modifica i file sul posto ed esegue rustfmt (predefinito)",
  check: "Modalità di controllo: esce con stato diverso da zero se i percorsi superano il limite (per CI)",
  max_segments: "Segmenti di percorso massimi consentiti prima della semplificazione (predefinito: 2)",
  keep_segments: "Numero di segmenti finali da mantenere (predefinito: 1; 2 per mantenere module::item)",
  allow_crate: "Whitelist di crate autorizzati a mantenere percorsi assoluti",
  extra_crate: "Ulteriori crate noti specificati esplicitamente",
  quiet: "Modalità silenziosa, sopprime i dettagli delle modifiche",
  show: "Modalità di visualizzazione, forza l'output dei dettagli modificati",
  verbose: "Visualizza log di elaborazione dettagliati",
  no_cache: "Disattiva la cache incrementale e forza un controllo completo",

  found_exceeding_limit: |count| {
    format!("Trovati {count} file con percorsi che superano il limite")
  },
  can_be_simplified: |count| {
    format!("\n{count} file possono essere semplificati, eseguire senza `--dry-run` per applicare")
  },
  failed_init_runtime: "Inizializzazione del runtime compio non riuscita",
  error_processing_file: |file, err| {
    format!("[warn] Errore durante l'elaborazione di {file}: {err}, ignorato")
  },
};
