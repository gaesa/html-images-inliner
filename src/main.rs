#![allow(clippy::needless_return)]

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD as B64_STANDARD};
use clap::Parser;
use kuchikiki::{ElementData, NodeDataRef, parse_html};
use kuchikiki::traits::TendrilSink;
use mime::Mime;
use rayon::prelude::*;

#[derive(Debug, Parser)]
#[clap(about = "A tool to inline images in HTML files using base64", long_about = None)]
struct Cli {
    #[arg(required = true, num_args = 1..)]
    files: Vec<PathBuf>,
}

fn get_modification<'a, I: Iterator<Item = &'a NodeDataRef<ElementData>>>(
    img_elements: I,
) -> Vec<Option<String>> {
    fn validate_string(s: String) -> Option<String> {
        #[allow(clippy::manual_strip)]
        return if s.starts_with("file://") {
            Some(s["file://".len()..].to_owned())
        } else if s.starts_with("./") || s.starts_with("../") || s.starts_with('/') {
            Some(s)
        } else {
            None
        };
    }

    fn validate_path<S: AsRef<str>>(s: Option<S>) -> Option<(PathBuf, Mime)> {
        return match s {
            Some(s) => {
                let path = PathBuf::from(s.as_ref());
                let path_ref: &Path = path.as_ref();
                if path_ref.is_file() {
                    Some((path.to_owned(), mime_guess::from_path(path_ref).first()?))
                } else {
                    None
                }
            }
            None => None,
        };
    }

    fn path_to_bytes(t: Option<(PathBuf, Mime)>) -> Option<(Vec<u8>, Mime)> {
        return t.map(|opt| {
            let mut buffer = Vec::new();
            File::open(&opt.0)
                .unwrap() // after `Path::is_file`
                .read_to_end(&mut buffer)
                .unwrap();
            (buffer, opt.1)
        });
    }

    fn bytes_to_base64(b: Option<(Vec<u8>, Mime)>) -> Option<String> {
        b.map(|opt| format!("data:{};base64,{}", opt.1, B64_STANDARD.encode(opt.0)))
    }

    let src_attrs = img_elements
        .map(|ele| ele.attributes.borrow())
        .map(|attrs| attrs.get("src").unwrap().to_owned())
        .collect::<Vec<_>>(); // because `NodeDataRef<ElementData>` is not thread-safe

    return src_attrs
        .into_par_iter() // positive impact on performance can be negligible
        .map(validate_string)
        .map(validate_path)
        .map(path_to_bytes)
        .map(bytes_to_base64)
        .collect::<Vec<_>>();
}

fn apply_modification<'a, I: Iterator<Item = &'a NodeDataRef<ElementData>>>(
    orig_data: I,
    modification: Vec<Option<String>>,
) {
    for (img, modified_src) in orig_data.zip(modification) {
        if let Some(modified_src) = modified_src {
            *img.attributes.borrow_mut().get_mut("src").unwrap() = modified_src;
        }
    }
}

fn main() {
    let args = Cli::parse();

    args.files
        .into_par_iter()
        .map(|path| {
            let path_ref: &Path = path.as_ref();
            let mut file =
                File::open(path_ref).unwrap_or_else(|_| panic!("Fail to open {path_ref:?}"));
            let mut html = String::new();
            file.read_to_string(&mut html)
                .unwrap_or_else(|_| panic!("Fail to read {:?}", &file));
            let document = parse_html().one(html);

            let img_elements = document
                .select("img")
                .unwrap_or_else(|_| panic!("{}", format!("No 'img' tag found in {path_ref:?}")))
                .collect::<Vec<_>>();

            apply_modification(img_elements.iter(), get_modification(img_elements.iter()));

            let output_path = path.with_extension("inlined.html");
            return (document.to_string(), output_path); // serialization is expansive especially in our case
        })
        .for_each(|(content, path)| {
            let path_ref: &Path = path.as_ref();
            fs::write(path_ref, content)
                .unwrap_or_else(|_| panic!("Fail to write to {path_ref:?}"));
        });
}
