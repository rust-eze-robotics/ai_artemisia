// std libs
pub use rand::Rng;

// tools
pub use giotto_tool::tools::image::{GiottoImage, GiottoImageBuilder};

// constants
pub const DEBUG: bool = true;
pub const IMG_PATH: &str = "res/img/";
pub const IMGS: [&str; 6] = [
    "agentileschi_giodittaoloferne.png",
    "fontana_concettospaziale.png",
    "giulialama_martirioeurosia.png",
    "meow.png", // this img gives problems
    "paularego_war.png",
    "remediosvaro_fenomeno.png",
];

/// This function builds a GiottoImage from a path
/// # Arguments
/// * `path` - string slice that holds the path to the image
/// # Returns
/// * a GiottoImage
/// # Example
/// ```
/// let img = build_img("res/img/meow.png");
/// ```
/// # Note
/// * The image will be resized to a height of 50 pixels
pub fn build_img(path: &str) -> GiottoImage {
    GiottoImageBuilder::new()
        .path(path)
        .resize_height(50)
        .build()
}

/// This function returns a random GiottoImage from the list of images
/// # Returns
/// * a GiottoImage
/// # Example
/// ```
/// let img = rand_img();
/// ```
/// # Note
/// * The list of images is hardcoded in the IMGS constant
/// * The function uses the IMG_PATH constant to build the path to the image
/// * The function uses the build_img function to build the GiottoImage
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

/// This function prints a debug message
/// # Arguments
/// * `s` - string slice that holds the message
/// # Example
/// ```
/// print_debug("This is a debug message");
/// ```
/// # Note
/// * The function uses the DEBUG constant to check if the message should be printed
/// * The message is printed with a prefix "ARTEMIS-IA: "
pub fn print_debug(s: &str) {
    if DEBUG {
        println!("\nARTEMIS-IA: {}\n", s);
    }
}
