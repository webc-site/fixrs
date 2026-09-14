use crate::i18n::msg::I18n;

pub const FR: I18n = I18n {
  about: "Outil CLI pour remplacer les chemins absolus Rust par des clauses use (corrige clippy::absolute_paths)",
  path: "Chemin du fichier ou répertoire à traiter (recherche Cargo.toml vers le haut par défaut)",
  dry_run: "Mode aperçu sans modifier les fichiers réels",
  write: "Modifier les fichiers sur place et exécuter rustfmt (par défaut)",
  check: "Mode vérification : quitter avec un code non nul si des chemins dépassent la limite (pour CI)",
  max_segments: "Nombre maximal de segments de chemin autorisés (défaut : 2)",
  keep_segments: "Nombre de segments finaux à conserver (défaut : 1 ; 2 pour conserver module::item)",
  allow_crate: "Liste blanche de crates autorisées à conserver leurs chemins absolus",
  extra_crate: "Crates connues supplémentaires spécifiées explicitement",
  quiet: "Mode silencieux, supprime les détails de modification",
  show: "Mode affichage, force la sortie des détails de modification",
  verbose: "Afficher les journaux détaillés",
  no_cache: "Désactiver le cache incrémental et forcer une nouvelle analyse",

  found_exceeding_limit: |count| {
    format!("{count} fichier(s) contenant des chemins dépassant la limite trouvés")
  },
  can_be_simplified: |count| {
    format!(
      "\n{count} fichier(s) peuvent être simplifiés, exécutez sans `--dry-run` pour appliquer"
    )
  },
  failed_init_runtime: "Échec de l'initialisation du runtime compio",
  error_processing_file: |file, err| {
    format!("[warn] Erreur lors du traitement de {file} : {err}, ignoré")
  },
};
