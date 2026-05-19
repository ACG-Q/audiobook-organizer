#[macro_export]
macro_rules! run_cli {
    ($cli_type:ty, $translate_fn:expr, $handler:expr) => {{
        use clap::{CommandFactory, FromArgMatches};
        use $crate::i18n::detect_lang;
        let lang = detect_lang();
        let mut cmd = <$cli_type as clap::CommandFactory>::command();
        cmd = $translate_fn(cmd, &lang);
        let matches = cmd
            .try_get_matches_from_mut(std::env::args())
            .unwrap_or_else(|e| e.exit());
        let cli = <$cli_type as clap::FromArgMatches>::from_arg_matches(&matches)
            .unwrap_or_else(|e| e.exit());
        $handler(cli)
    }};
}
