use crate::cli::Format;
use crate::cli::Target;
use anyhow::Context;
use anyhow::Result;
use chrono::prelude::*;
use rumpus::image::RayImage;
use rumpus::optic::Camera;
use rumpus::optic::PinholeOptic;
use rumpus::ray::GlobalFrame;
use rumpus::simulation::Simulation;
use sguaba::Coordinate;
use sguaba::engineering::Orientation;
use sguaba::engineering::Pose;
use sguaba::math::RigidBodyTransform;
use sguaba::system;
use sguaba::systems::Wgs84;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uom::si::f64::Angle;
use uom::si::f64::Length;
use uom::si::{
    angle::degree,
    length::{meter, micron, millimeter},
};

pub fn run(
    params: Option<PathBuf>,
    target: Target,
    output: PathBuf,
    format: Option<Format>,
) -> Result<()> {
    let params = match params {
        Some(path) => Params::try_from_toml(path)?,
        None => Params::default(),
    };

    let ray_image = simulate(&params)?;

    let format = format
        .map(Ok)
        .unwrap_or_else(|| crate::common::parse_format(&output))?;

    crate::common::write_ray_image(ray_image, target, format, &output)?;

    Ok(())
}

system!(struct CameraBody using right-handed XYZ);
system!(struct CameraEnu using ENU);

fn simulate(params: &Params) -> Result<RayImage<GlobalFrame>> {
    // SAFETY: CameraBody and CameraEnu have coincident origins.
    let camera_enu_to_ecef =
        unsafe { RigidBodyTransform::ecef_to_enu_at(&params.wgs84()?) }.inverse();

    let camera_pose_enu = Pose::new(Coordinate::origin(), params.orientation());
    let camera_pose_ecef = camera_enu_to_ecef.transform(camera_pose_enu);

    let ray_image = Simulation::new(
        Camera::new(
            PinholeOptic::from_focal_length(params.focal_length()),
            params.pixel_size(),
            params.image_rows(),
            params.image_cols(),
        ),
        camera_pose_ecef,
        params.time(),
    )
    .par_ray_image();

    Ok(ray_image)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Params {
    pixel_size_um: f64,
    focal_length_mm: f64,
    image_rows: usize,
    image_cols: usize,
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
    fn try_from_toml<P: AsRef<Path>>(path: P) -> Result<Params> {
        let mut buffer = String::new();
        std::fs::File::open(path)?.read_to_string(&mut buffer)?;
        let params = toml::from_str(&buffer)?;
        Ok(params)
    }

    fn focal_length(&self) -> Length {
        Length::new::<millimeter>(self.focal_length_mm)
    }

    fn pixel_size(&self) -> Length {
        Length::new::<micron>(self.pixel_size_um)
    }

    fn image_rows(&self) -> usize {
        self.image_rows
    }

    fn image_cols(&self) -> usize {
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
