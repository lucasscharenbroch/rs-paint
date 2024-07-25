use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
#[command(name = crate::PROGRAM_NAME)]
#[command(version = crate::SEMANTIC_VERSION)]
#[command(about = crate::PROGRAM_DESCRIPTION)]
pub struct CliSettings {
    pub image_file: Option<String>,
}
