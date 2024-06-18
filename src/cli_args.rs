use clap::Parser as ClapParser;

#[derive(ClapParser)]
pub struct CliArgs {
    pub source: Option<String>,
}
