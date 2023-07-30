use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Debug, ValueEnum)]
pub enum FileEvent {
    Access,
    Create,
    Modify,
    Remove,
    Any,
}
