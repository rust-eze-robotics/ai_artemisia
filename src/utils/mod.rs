pub use rand::Rng;
pub use giotto_tool::tools::image::{GiottoImage, GiottoImageBuilder};

pub const IMG_PATH: &str = "res/img/";

pub const IMGS: [&str; 1] = [
    "meow.png",
];

pub fn build_img(path: &str) -> GiottoImage {
    GiottoImageBuilder::new().path(path).build()
}

pub fn rand_img() {
    let mut rng = rand::thread_rng();
    let img_name = IMGS[rng.gen_range(0..IMGS.len())];
    let tmp = [IMG_PATH.to_string(), img_name.to_string()].concat();

    let img_path = tmp.as_str();

    let giotto_img : GiottoImage = build_img(img_path);
        // .resize_height(50)
    println!("{:?}", giotto_img);

}