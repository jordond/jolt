use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum HistoryCommands {
    Summary {
        #[arg(short, long, default_value = "week")]
        period: String,
    },

    Top {
        #[arg(short, long, default_value = "week")]
        period: String,

        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    Export {
        #[arg(short, long)]
        output: Option<String>,

        #[arg(short, long, default_value = "json")]
        format: String,

        #[arg(long)]
        from: Option<String>,

        #[arg(long)]
        to: Option<String>,

        #[arg(short, long)]
        period: Option<String>,

        #[arg(long)]
        include_samples: bool,
    },

    Prune {
        #[arg(long)]
        older_than: Option<u32>,

        #[arg(short = 'y', long)]
        yes: bool,
    },
}
