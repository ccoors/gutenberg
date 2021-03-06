extern crate elasticlunr;
#[macro_use]
extern crate lazy_static;
extern crate ammonia;
#[macro_use]
extern crate errors;
extern crate content;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use elasticlunr::{Index, Language};

use content::Section;
use errors::Result;


pub const ELASTICLUNR_JS: &'static str = include_str!("elasticlunr.min.js");

lazy_static! {
    static ref AMMONIA: ammonia::Builder<'static> = {
        let mut clean_content = HashSet::new();
        clean_content.insert("script");
        clean_content.insert("style");
        let mut builder = ammonia::Builder::new();
        builder
            .tags(HashSet::new())
            .tag_attributes(HashMap::new())
            .generic_attributes(HashSet::new())
            .link_rel(None)
            .allowed_classes(HashMap::new())
            .clean_content_tags(clean_content);
        builder
    };
}


/// Returns the generated JSON index with all the documents of the site added using
/// the language given
/// Errors if the language given is not available in Elasticlunr
/// TODO: is making `in_search_index` apply to subsections of a `false` section useful?
pub fn build_index(sections: &HashMap<PathBuf, Section>, lang: &str) -> Result<String> {
    let language = match Language::from_code(lang) {
        Some(l) => l,
        None => { bail!("Tried to build search index for language {} which is not supported", lang); }
    };

    let mut index = Index::with_language(language, &["title", "body"]);

    for section in sections.values() {
        add_section_to_index(&mut index, section);
    }

    Ok(index.to_json())
}

fn add_section_to_index(index: &mut Index, section: &Section) {
    if !section.meta.in_search_index {
        return;
    }

    // Don't index redirecting sections
    if section.meta.redirect_to.is_none() {
        index.add_doc(
            &section.permalink,
            &[&section.meta.title.clone().unwrap_or(String::new()), &AMMONIA.clean(&section.content).to_string()],
        );
    }

    for page in &section.pages {
        if !page.meta.in_search_index || page.meta.draft {
            continue;
        }

        index.add_doc(
            &page.permalink,
            &[&page.meta.title.clone().unwrap_or(String::new()), &AMMONIA.clean(&page.content).to_string()],
        );
    }
}
