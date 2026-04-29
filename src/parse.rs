use crate::cli::Format;
use crate::cli::Target;
use anyhow::Result;
use rumpus::image::IntensityImage;
use rumpus::image::RayImage;
use rumpus::optic::PixelCoordinate;
use rumpus::ray::SensorFrame;
use std::path::Path;
use std::path::PathBuf;

#[allow(clippy::similar_names)]
pub fn run(
    file: PathBuf,
    output: PathBuf,
    min_dop: f64,
    target: Target,
    format: Option<Format>,
) -> anyhow::Result<()> {
    if min_dop > 0. {
        println!("dop filtering not supported... yet!");
    }

    let ray_image = ray_image_from_path(file)?;
    let format = format
        .map(Ok)
        .unwrap_or_else(|| crate::common::parse_format(&output))?;

    match target {
        Target::AopSensor | Target::Dop | Target::S0 => {
            crate::common::write_ray_image(ray_image, target, format, &output)?;
        }
        Target::AopGlobal => {
            let pixel_coord = PixelCoordinate::new(ray_image.rows() / 2, ray_image.cols() / 2);
            let ray_image = crate::common::sensor_to_global(&ray_image, &pixel_coord);
            crate::common::write_ray_image(ray_image, target, format, &output)?;
        }
    }

    Ok(())
}

fn ray_image_from_path<P: AsRef<Path>>(path: P) -> Result<RayImage<SensorFrame>> {
    let image = image::ImageReader::open(&path)
        .unwrap()
        .decode()
        .unwrap()
        .into_luma8();

    let (width, height) = image.dimensions();
    let intensity_image =
        IntensityImage::from_bytes(width as usize, height as usize, &image.into_raw())
            .expect("image dimensions are even");

    Ok(RayImage::from_metapixels(
        intensity_image.metapixels(),
        intensity_image.rows(),
        intensity_image.cols(),
    )?)
}
