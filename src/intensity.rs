use clap::Parser;
use image::ImageReader;
use rumpus::image::IntensityImage;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input image.
    image: PathBuf,

    #[arg(long, default_value_t = 0.5)]
    dop_max: f64,
}

fn main() {
    let args = Args::parse();

    let image = ImageReader::open(args.image)
        .unwrap()
        .decode()
        .unwrap()
        .into_luma8();

    let (width, height) = image.dimensions();
    let stokes_image = IntensityImage::from_bytes(width, height, &image.into_raw())
        .unwrap()
        .into_stokes_image()
        .par_transform_frame(StokesReferenceFrame::Pixel);

    let mms = stokes_image.into_measurements();
    let (width, height) = stokes_image.dimensions();
    let aop_image = AopImage::from_sparse_mms(&mms, width, height).into_raw();
    let dop_image = DopImage::from_sparse_mms(&mms, width, height).into_raw();

    let _ = image::save_buffer(
        "aop.png",
        &aop_image,
        width,
        height,
        image::ExtendedColorType::Rgb8,
    );

    let _ = image::save_buffer(
        "dop.png",
        &dop_image,
        width,
        height,
        image::ExtendedColorType::Rgb8,
    );
}
