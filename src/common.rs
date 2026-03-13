use anyhow::Result;
use anyhow::anyhow;
use rumpus::image::Binary;
use rumpus::optic::PixelCoordinate;
use rumpus::ray::GlobalFrame;
use rumpus::ray::Ray;
use rumpus::ray::SensorFrame;
use std::fs;
use std::path::Path;
use uom::si::angle::radian;
use uom::si::f64::Angle;

use rumpus::image::{Jet, RayImage};

use crate::cli::{Format, Target};

pub(crate) fn parse_format<P: AsRef<Path>>(path: P) -> Result<Format> {
    match path
        .as_ref()
        .extension()
        .ok_or(anyhow!("no extension provided"))?
        .to_str()
    {
        Some("png") => Ok(Format::Png),
        Some("dat") => Ok(Format::Dat),
        Some("bin") => Ok(Format::Bin),
        _ => Err(anyhow!("could not infer a supported format from extension")),
    }
}

pub(crate) fn write_ray_image<F: Copy, P: AsRef<Path>>(
    ray_image: RayImage<F>,
    target: Target,
    format: Format,
    output: P,
) -> Result<()> {
    match format {
        Format::Png => write_image(ray_image, target, output),
        Format::Dat => write_dat(ray_image, target, output),
        Format::Bin => write_bin(ray_image, target, output),
    }
}

pub(crate) fn write_image<F: Copy, P: AsRef<Path>>(
    ray_image: RayImage<F>,
    target: Target,
    path: P,
) -> Result<()> {
    let bytes = match target {
        Target::AopSensor | Target::AopGlobal => ray_image.aop_bytes(&Jet),
        Target::Dop => ray_image.dop_bytes(&Jet),
    };

    image::save_buffer(
        path,
        &bytes,
        ray_image.cols() as u32,
        ray_image.rows() as u32,
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(())
}

pub(crate) fn write_dat<F, P: AsRef<Path>>(
    ray_image: RayImage<F>,
    target: Target,
    path: P,
) -> Result<()> {
    todo!()

    // Write simulated output to file.
    // let mut output_file = BufWriter::new(File::create(&path)?);
    // for row in 0..rows {
    //     for col in 0..cols {
    //         let i: usize = (row * cols + col).try_into()?;
    //         write!(output_file, "{:5} ", image[i])?;
    //     }
    //     write!(output_file, "\n")?;
    // }
    //
    // Ok(())
}

pub(crate) fn write_bin<F: Copy, P: AsRef<Path>>(
    ray_image: RayImage<F>,
    target: Target,
    output: P,
) -> Result<()> {
    let bytes = match target {
        Target::AopSensor | Target::AopGlobal => ray_image.aop_bytes(&Binary),
        Target::Dop => ray_image.dop_bytes(&Binary),
    };

    fs::write(output, &bytes)?;

    Ok(())
}

/// Shifts the ray_image ignoring any tilt!
pub(crate) fn sensor_to_global(
    ray_image: &RayImage<SensorFrame>,
    origin: &PixelCoordinate,
) -> RayImage<GlobalFrame> {
    let rays: Vec<_> = ray_image
        .pixels()
        .map(|px| {
            let ray = px.ray()?;

            let px_coord = PixelCoordinate::new(px.row(), px.col());

            let shift = shift_by(px_coord, origin);
            let angle = ray.aop().into_global_frame(-shift);
            Some(Ray::<GlobalFrame>::new(angle, ray.dop()))
        })
        .collect();

    RayImage::from_rays(rays, ray_image.rows(), ray_image.cols()).unwrap()
}

#[allow(clippy::cast_precision_loss)]
fn shift_by(coord: PixelCoordinate, origin: &PixelCoordinate) -> Angle {
    let y0 = origin.row() as f64;
    let x0 = origin.col() as f64;

    let y1 = coord.row() as f64;
    let x1 = coord.col() as f64;

    let y = -y1 + y0;
    let x = x1 - x0;

    Angle::new::<radian>(y.atan2(x))
}
