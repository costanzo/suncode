use crate::{ComputerError, ComputerResult};
use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayGeometry {
    pub generation: u64,
    pub input_x: i32,
    pub input_y: i32,
    pub input_width: u32,
    pub input_height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PixelPoint {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PixelRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerFrame {
    pub display: DisplayGeometry,
    pub rgba: Vec<u8>,
}

impl ComputerFrame {
    pub fn new(display: DisplayGeometry, rgba: Vec<u8>) -> ComputerResult<Self> {
        if display.input_width == 0
            || display.input_height == 0
            || display.pixel_width == 0
            || display.pixel_height == 0
        {
            return Err(ComputerError::InvalidFrame(
                "display dimensions must be positive",
            ));
        }
        let expected = usize::try_from(display.pixel_width)
            .ok()
            .and_then(|width| {
                usize::try_from(display.pixel_height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(ComputerError::InvalidFrame(
                "display dimensions are too large",
            ))?;
        if rgba.len() != expected {
            return Err(ComputerError::InvalidFrame(
                "RGBA payload length does not match the display dimensions",
            ));
        }
        Ok(Self { display, rgba })
    }

    pub fn input_point(&self, point: PixelPoint) -> ComputerResult<(i32, i32)> {
        if point.x >= self.display.pixel_width || point.y >= self.display.pixel_height {
            return Err(ComputerError::CoordinateOutOfBounds);
        }
        Ok((
            map_axis(
                point.x,
                self.display.pixel_width,
                self.display.input_x,
                self.display.input_width,
            )?,
            map_axis(
                point.y,
                self.display.pixel_height,
                self.display.input_y,
                self.display.input_height,
            )?,
        ))
    }

    pub fn crop(&self, region: PixelRegion) -> ComputerResult<Self> {
        validate_region(region, self.display.pixel_width, self.display.pixel_height)?;
        let source_stride = usize::try_from(self.display.pixel_width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(ComputerError::InvalidFrame(
                "display dimensions are too large",
            ))?;
        let crop_stride = usize::try_from(region.width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(ComputerError::InvalidAction("zoom region is too large"))?;
        let mut rgba = Vec::with_capacity(
            crop_stride
                .checked_mul(
                    usize::try_from(region.height)
                        .map_err(|_| ComputerError::InvalidAction("zoom region is too large"))?,
                )
                .ok_or(ComputerError::InvalidAction("zoom region is too large"))?,
        );
        let x_offset = usize::try_from(region.x)
            .ok()
            .and_then(|x| x.checked_mul(4))
            .ok_or(ComputerError::InvalidAction("zoom region is too large"))?;
        for row in region.y..region.y + region.height {
            let start = usize::try_from(row)
                .ok()
                .and_then(|row| row.checked_mul(source_stride))
                .and_then(|offset| offset.checked_add(x_offset))
                .ok_or(ComputerError::InvalidAction("zoom region is too large"))?;
            rgba.extend_from_slice(&self.rgba[start..start + crop_stride]);
        }
        let top_left = self.input_point(PixelPoint {
            x: region.x,
            y: region.y,
        })?;
        let bottom_right = self.input_point(PixelPoint {
            x: region.x + region.width - 1,
            y: region.y + region.height - 1,
        })?;
        Self::new(
            DisplayGeometry {
                generation: self.display.generation,
                input_x: top_left.0,
                input_y: top_left.1,
                input_width: u32::try_from(bottom_right.0 - top_left.0 + 1)
                    .map_err(|_| ComputerError::CoordinateOutOfBounds)?,
                input_height: u32::try_from(bottom_right.1 - top_left.1 + 1)
                    .map_err(|_| ComputerError::CoordinateOutOfBounds)?,
                pixel_width: region.width,
                pixel_height: region.height,
            },
            rgba,
        )
    }

    pub fn png(&self) -> ComputerResult<Vec<u8>> {
        let image = RgbaImage::from_raw(
            self.display.pixel_width,
            self.display.pixel_height,
            self.rgba.clone(),
        )
        .ok_or(ComputerError::InvalidFrame(
            "RGBA payload length does not match the display dimensions",
        ))?;
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, ImageFormat::Png)
            .map_err(|error| ComputerError::Backend(format!("PNG encoding failed: {error}")))?;
        Ok(bytes.into_inner())
    }
}

fn map_axis(
    pixel: u32,
    pixel_extent: u32,
    input_origin: i32,
    input_extent: u32,
) -> ComputerResult<i32> {
    if pixel_extent == 1 || input_extent == 1 {
        return Ok(input_origin);
    }
    let offset = u64::from(pixel) * u64::from(input_extent - 1) / u64::from(pixel_extent - 1);
    input_origin
        .checked_add(i32::try_from(offset).map_err(|_| ComputerError::CoordinateOutOfBounds)?)
        .ok_or(ComputerError::CoordinateOutOfBounds)
}

fn validate_region(region: PixelRegion, width: u32, height: u32) -> ComputerResult<()> {
    if region.width == 0
        || region.height == 0
        || region
            .x
            .checked_add(region.width)
            .is_none_or(|right| right > width)
        || region
            .y
            .checked_add(region.height)
            .is_none_or(|bottom| bottom > height)
    {
        return Err(ComputerError::InvalidAction(
            "zoom region must be inside the screenshot",
        ));
    }
    Ok(())
}
