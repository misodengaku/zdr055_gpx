use clap::Parser;

#[derive(Parser)]
pub(crate) struct Cli {
    path: std::path::PathBuf,

    #[clap(short, long, default_value = "./")]
    output_path: Option<std::path::PathBuf>,

    #[clap(short, long, default_value = "8")]
    parallel: usize,

    #[clap(short, long, default_value = "false")]
    merge: bool,

    #[clap(long, default_value = "6h")]
    merge_threshold: humantime::Duration,
}

impl Cli {
    pub(crate) fn get_output_path(&self) -> std::path::PathBuf {
        match &self.output_path {
            Some(path) => path.clone(),
            None => "./".into(),
        }
    }

    pub(crate) fn get_parallel_count(&self) -> usize {
        if self.parallel > 0 {
            self.parallel
        } else {
            1
        }
    }

    pub(crate) fn get_input_path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub(crate) fn get_merge_enabled(&self) -> bool {
        self.merge
    }

    pub(crate) fn get_merge_threshold(&self) -> humantime::Duration {
        self.merge_threshold
    }

    pub(crate) fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
