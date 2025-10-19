use crate::cli::SimulationFormat;
use anyhow::Context;
use anyhow::Result;
use chrono::prelude::*;
use rayon::prelude::*;
use rumpus::prelude::*;
use sguaba::Coordinate;
use sguaba::engineering::Orientation;
use sguaba::systems::Wgs84;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
};
use uom::si::f64::Angle;
use uom::si::f64::Length;
use uom::si::{
    angle::degree,
    length::{meter, micron, millimeter},
};

#[derive(serde::Serialize, serde::Deserialize)]
struct Params {
    pixel_size_um: f64,
    focal_length_mm: f64,
    image_rows: u16,
    image_cols: u16,
    yaw_deg: f64,
    pitch_deg: f64,
    roll_deg: f64,
    lat_deg: f64,
    lon_deg: f64,
    time: DateTime<Utc>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            pixel_size_um: 3.45 * 2.,
            focal_length_mm: 8.,
            image_rows: 1024,
            image_cols: 1224,
            yaw_deg: 0.,
            pitch_deg: 0.,
            roll_deg: 0.,
            lat_deg: 44.2187,
            lon_deg: -76.4747,
            time: "2025-06-13T16:26:47+00:00".parse().unwrap(),
        }
    }
}

impl Params {
    fn focal_length(&self) -> Length {
        Length::new::<millimeter>(self.focal_length_mm)
    }

    fn pixel_size(&self) -> Length {
        Length::new::<micron>(self.pixel_size_um)
    }

    fn image_rows(&self) -> u16 {
        self.image_rows
    }

    fn image_cols(&self) -> u16 {
        self.image_cols
    }

    fn wgs84(&self) -> Result<Wgs84> {
        Ok(Wgs84::builder()
            .longitude(Angle::new::<degree>(self.lon_deg))
            .latitude(Angle::new::<degree>(self.lat_deg))
            .context("latitude between -90 and 90 degrees")?
            // Altitude is not used in the sky model.
            .altitude(Length::new::<meter>(0.0))
            .build())
    }

    fn time(&self) -> DateTime<Utc> {
        self.time
    }

    fn orientation(&self) -> Orientation<CameraEnu> {
        Orientation::<CameraEnu>::tait_bryan_builder()
            .yaw(Angle::new::<degree>(self.yaw_deg))
            .pitch(Angle::new::<degree>(self.pitch_deg))
            .roll(Angle::new::<degree>(self.roll_deg))
            .build()
    }
}

pub fn run(
    params: &Option<PathBuf>,
    output: &PathBuf,
    format: &Option<SimulationFormat>,
) -> Result<()> {
    let params = match params {
        Some(path) => parse_params(&path)?,
        None => Params::default(),
    };

    let ray_image = simulate(&params)?;

    match format.or_else(|| {
        match output
            .as_path()
            .extension()
            .map(|os_str: &OsStr| os_str.to_str())
        {
            Some(Some("png")) => Some(SimulationFormat::Png),
            Some(Some("dat")) => Some(SimulationFormat::Dat),
            _ => None,
        }
    }) {
        Some(format) => match format {
            SimulationFormat::Png => {
                write_image(ray_image, params.image_rows(), params.image_cols(), output)
            }
            SimulationFormat::Dat => {
                write_dat(ray_image, params.image_rows(), params.image_cols(), output)
            }
        },
        None => anyhow::bail!("unsupported output format"),
    }
}

fn simulate(params: &Params) -> Result<RayImage<GlobalFrame>> {
    let lens = Lens::from_focal_length(params.focal_length()).expect("positive focal length");
    let image_sensor = ImageSensor::new(
        params.pixel_size(),
        params.pixel_size(),
        params.image_rows(),
        params.image_cols(),
    );
    let coords: Vec<Coordinate<CameraFrd>> = (0..params.image_rows())
        .flat_map(|row| (0..params.image_cols()).map(move |col| (row, col)))
        .map(|(row, col)| image_sensor.at_pixel(row, col).unwrap())
        .collect();

    let sky_model = SkyModel::from_wgs84_and_time(params.wgs84()?, params.time());
    let cam_orientation = params.orientation();

    let camera = Camera::new(lens.clone(), cam_orientation);
    let rays: Vec<Ray<_>> = coords
        .par_iter()
        .filter_map(|coord| {
            let bearing_cam_enu = camera
                .trace_from_sensor(*coord)
                .expect("coord on sensor plane");
            let aop = sky_model.aop(bearing_cam_enu)?;

            Some(Ray::new(*coord, aop, Dop::new(0.0)))
        })
        .collect();

    Ok(RayImage::from_rays_with_sensor(rays, &image_sensor).expect("no ray hits the same pixel"))
}

fn parse_params(path: &PathBuf) -> Result<Params> {
    let mut buffer = String::new();
    std::fs::File::open(path)?.read_to_string(&mut buffer)?;
    let params = toml::from_str(&buffer)?;

    Ok(params)
}

fn write_image(
    ray_image: RayImage<GlobalFrame>,
    image_rows: u16,
    image_cols: u16,
    path: &PathBuf,
) -> Result<()> {
    // Map the AoP values in the RayImage to RGB colours.
    // Draw missing pixels as white.
    let aop_image: Vec<u8> = ray_image
        .ray_pixels()
        .flat_map(|pixel| match pixel {
            Some(ray) => to_rgb(ray.aop().angle().get::<degree>(), -90.0, 90.0)
                .expect("aop in between -90 and 90"),
            None => [255, 255, 255],
        })
        .collect();

    // Save the buffer of RGB pixels as a PNG.
    image::save_buffer(
        &path,
        &aop_image,
        image_cols.into(),
        image_rows.into(),
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(())
}

fn write_dat(ray_image: RayImage<GlobalFrame>, rows: u16, cols: u16, path: &PathBuf) -> Result<()> {
    // Map the AoP values in the RayImage to RGB colours.
    // Draw missing pixels as white.
    let aop_image: Vec<f64> = ray_image
        .ray_pixels()
        .map(|pixel| match pixel {
            Some(ray) => ray.aop().angle().get::<degree>(),
            None => f64::NAN,
        })
        .collect();

    // Write simulated output to file.
    let mut output_file = BufWriter::new(File::create(&path)?);
    for row in 0..rows {
        for col in 0..cols {
            let i: usize = (row * cols + col).try_into()?;
            write!(output_file, "{:5} ", aop_image[i])?;
        }
        write!(output_file, "\n")?;
    }

    Ok(())
}

// Map an f64 on the interval [x_min, x_max] to an RGB color.
pub fn to_rgb(x: f64, x_min: f64, x_max: f64) -> Option<[u8; 3]> {
    if x < x_min || x > x_max {
        return None;
    }

    let interval_width = x_max - x_min;
    let x_norm = ((x - x_min) / interval_width * 255.).floor() as u8;

    let r = vec![
        255,
        x_norm
            .checked_sub(96)
            .unwrap_or(u8::MIN)
            .checked_mul(4)
            .unwrap_or(u8::MAX),
        255 - x_norm
            .checked_sub(224)
            .unwrap_or(u8::MIN)
            .checked_mul(4)
            .unwrap_or(u8::MAX),
    ]
    .into_iter()
    .min()
    .unwrap();

    let g = vec![
        255,
        x_norm
            .checked_sub(32)
            .unwrap_or(u8::MIN)
            .checked_mul(4)
            .unwrap_or(u8::MAX),
        255 - x_norm
            .checked_sub(160)
            .unwrap_or(u8::MIN)
            .checked_mul(4)
            .unwrap_or(u8::MAX),
    ]
    .into_iter()
    .min()
    .unwrap();

    let b = vec![
        255,
        x_norm
            .checked_add(127)
            .unwrap_or(u8::MIN)
            .checked_mul(4)
            .unwrap_or(u8::MAX),
        255 - x_norm
            .checked_sub(96)
            .unwrap_or(u8::MIN)
            .checked_mul(4)
            .unwrap_or(u8::MAX),
    ]
    .into_iter()
    .min()
    .unwrap();

    Some([r, g, b])
}
