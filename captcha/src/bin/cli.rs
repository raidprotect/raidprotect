//! Command line interface to generate a captcha.
//!
//! This CLI is used for testing purposes while working on the RaidProtect captcha
//! generation.
//!
//! Use `cargo run --features cli --bin captcha-cli` to run it.

use argh::FromArgs;
use imageproc::window::display_image;
use raidprotect_captcha::{
    code::{random_code, random_human_code},
    generate_captcha,
};

/// Generate a captcha.
#[derive(FromArgs, Debug)]
pub struct CaptchaArgs {
    /// code of the generated captcha (random if missing)
    #[argh(option, short = 'c')]
    code: Option<String>,
    /// length of the captcha code
    #[argh(option, default = "6", short = 'l')]
    length: usize,
    /// generated image output path (the image will be opened in a new window if missing)
    #[argh(option, short = 'o')]
    output: Option<String>,
    /// whether the generated code should be easy to read for a human
    #[argh(switch, short = 'h')]
    human: bool,
}

fn main() {
    let args: CaptchaArgs = argh::from_env();
    let code = args.code.unwrap_or_else(|| {
        if args.human {
            random_human_code(args.length)
        } else {
            random_code(args.length)
        }
    });

    let image = generate_captcha(&code);
    let (width, height) = image.dimensions();

    if let Some(output) = args.output {
        if let Err(error) = image.save(output) {
            eprintln!("failed to save image: {error}");
        }
    } else {
        display_image("captcha.png", &image, width, height)
    }
}
