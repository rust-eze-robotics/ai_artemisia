// std libs
pub use rand::Rng;

// tools
pub use giotto_tool::tools::image::{GiottoImage, GiottoImageBuilder};

pub const DEBUG: bool = true;
pub const IMG_PATH: &str = "res/img/";

pub const IMGS: [&str; 5] = [
    "agentileschi_giodittaoloferne.png",
    "fontana_concettospaziale.png",
    "giulialama_martirioeurosia.png",
    // "meow.png", // this img gives problems
    "paularego_war.png",
    "remediosvaro_fenomeno.png",
];

pub fn build_img(path: &str) -> GiottoImage {
    GiottoImageBuilder::new()
        .path(path)
        .resize_height(50)
        .build()
}

pub fn rand_img() -> GiottoImage {
    let mut rng = rand::thread_rng();
    let img_name = IMGS[rng.gen_range(0..IMGS.len())];
    let tmp = [IMG_PATH.to_string(), img_name.to_string()].concat();

    print_debug(format!("Painting's name: {}", tmp).as_str());

    let img_path = tmp.as_str();

    let giotto_img: GiottoImage = build_img(img_path);
    print_debug(format!("{:?}", giotto_img).as_str());

    giotto_img
}

pub fn print_debug(s: &str) {
    if DEBUG {
        println!("\nARTEMIS-IA: {}\n", s);
    }
}
