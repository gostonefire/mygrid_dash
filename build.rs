// build.rs
use std::{fs, path::Path};

fn main() {
    let index_template_path = Path::new("static_src/index.html");
    let index_out_path = Path::new("static/index.html");
    let index_js_path = Path::new("static/mygrid_dash_essential_v2.js");

    let index_full_template_path = Path::new("static_src/index_full.html");
    let index_full_out_path = Path::new("static/index_full.html");
    let index_full_js_path = Path::new("static/mygrid_dash_v2.js");

    hash_js_and_build_html(index_js_path, index_template_path, index_out_path);
    hash_js_and_build_html(index_full_js_path, index_full_template_path, index_full_out_path);

    println!("cargo:rerun-if-changed={}", index_template_path.display());
    println!("cargo:rerun-if-changed={}", index_js_path.display());
    println!("cargo:rerun-if-changed={}", index_full_template_path.display());
    println!("cargo:rerun-if-changed={}", index_full_js_path.display());
}

/// Hashes the js file and replaces the hash in the HTML template,
/// then writes the result to the output path
///
/// # Arguments
///
/// * 'js_path' - the path to the js file
/// * 'template_path' - the path to the HTML template file
/// * 'out_path' - the path to write the output HTML file
fn hash_js_and_build_html(js_path: &Path, template_path: &Path, out_path: &Path) {
    let html = fs::read_to_string(template_path).unwrap();
    let js = fs::read(js_path).unwrap();

    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    js.hash(&mut h);
    let js_hash = format!("{:x}", h.finish());

    let html = html.replace("{{JS_HASH}}", &js_hash);

    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    fs::write(out_path, html).unwrap();
}