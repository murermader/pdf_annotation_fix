use std::{
    collections::HashSet,
    iter::once, path::PathBuf, fs::{OpenOptions, File},
};

use anyhow::{bail, Context};
use lopdf::{Document, Object};

pub fn fix_pdf_annotations(input: PathBuf, output: PathBuf) -> anyhow::Result<usize> {
    let mut doc = Document::load_from(
        File::open(input.clone()).context("unable to open input pdf")?
    ).context("unable to parse pdf document")?;

    let pages = doc.get_pages();
    let reference_objects = doc
        .objects
        .values()
        .flat_map(Object::as_array)
        .filter(|o| o.iter().all(|o| o.as_reference().is_ok()))
        .map(|r| {
            r.iter()
                .flat_map(Object::as_reference)
                .collect::<HashSet<_>>()
        })
        .collect::<Vec<_>>();
    let mut recovered_annotations = 0;
    for page_id in pages.values() {
        let page = doc
            .get_object_mut(*page_id)
            .context("unable to get page object")?;
        let dict = page
            .as_dict_mut()
            .context("page object is not a dictionary")?;

        if let Ok(annots) = dict.get(b"Annots") {
            let annots = match annots {
                lopdf::Object::Array(a) => a
                    .iter()
                    .flat_map(Object::as_reference)
                    .collect::<HashSet<_>>(),
                Object::Reference(r) => once(r).cloned().collect::<HashSet<_>>(),
                _ => bail!("annotations are neither an array nor a single reference"),
            };
            if let Some(replacement_annotations) = reference_objects
                .iter()
                .find(|r| annots.len() != r.len() && annots.is_subset(r))
            {
                dict.set(
                    b"Annots".to_vec(),
                    Object::Array(
                        replacement_annotations
                            .iter()
                            .copied()
                            .map(Object::Reference)
                            .collect::<Vec<_>>(),
                    ),
                );
                recovered_annotations += replacement_annotations.len() - annots.len()
            }
        }
    }

    let input_override = input.as_path() == output.as_path();

    if recovered_annotations == 0 {
        println!("No annotations recovered for [{}]", input.display());
        return Ok(recovered_annotations)
    }

    if input_override {
        println!("Removing {} annotations. Override file [{}]", recovered_annotations, input.display())
    } else {
        println!("Removing {} annotations. Create new file [{}]", recovered_annotations, input.display())
    }

    let mut output_options = OpenOptions::new()
            .create_new(!input_override)
            .write(true)
            .truncate(input_override)
            .open(output)
            .context("unable to open output file. does it already exist?")?;
   
    doc.save_to(&mut output_options)
        .context("unable to save pdf document")?;
    Ok(recovered_annotations)
}
