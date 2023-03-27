use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "PDF Annotation Fixer",
    about = "Fixing messed up PDF annotions. Use at your own risk!"
)]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let recovered_annots = pdf_fixing_lib::fix_pdf_annotations(
        opt.input.clone(),
        if opt.output.is_none() { opt.input.clone() } else { opt.output.unwrap() },
    )
    .context("unable to fix annoations")?;

    println!("recovered {} annotations", recovered_annots);
    Ok(())
}
