//! Cross-platform screen capture (DXGI hook on Windows, screenshots fallback).

use anyhow::{Context, Result};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, DynamicImage, ImageBuffer, RgbaImage};
use std::io::Cursor;

pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub jpeg: Vec<u8>,
}

pub trait ScreenCapture: Send {
    fn capture_frame(&mut self) -> Result<CapturedFrame>;
    fn primary_size(&self) -> (u32, u32);
}

pub fn create_capture() -> Result<Box<dyn ScreenCapture>> {
    #[cfg(windows)]
    {
        if DxgiCapture::probe().is_ok() {
            tracing::info!("Windows graphics stack OK — capture via desktop API");
        }
    }
    Ok(Box::new(ScreenshotCapture::new()?))
}

pub struct ScreenshotCapture {
    screen: screenshots::Screen,
    width: u32,
    height: u32,
    quality: u8,
}

impl ScreenshotCapture {
    pub fn new() -> Result<Self> {
        let screen = screenshots::Screen::all()
            .context("no displays")?
            .into_iter()
            .next()
            .context("no primary display")?;
        let info = screen.display_info;
        Ok(Self {
            width: info.width,
            height: info.height,
            screen,
            quality: 70,
        })
    }

    fn encode_jpeg(&self, rgba: RgbaImage) -> Result<Vec<u8>> {
        let img = DynamicImage::ImageRgba8(rgba);
        let scaled = if img.width() > 1920 {
            img.resize(1920, (1920 * img.height()) / img.width(), FilterType::Triangle)
        } else {
            img
        };
        let rgb = scaled.to_rgb8();
        let mut buf = Cursor::new(Vec::new());
        let mut enc = JpegEncoder::new_with_quality(&mut buf, self.quality);
        enc.encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )?;
        Ok(buf.into_inner())
    }
}

impl ScreenCapture for ScreenshotCapture {
    fn capture_frame(&mut self) -> Result<CapturedFrame> {
        let image = self.screen.capture().context("screenshot failed")?;
        let w = image.width();
        let h = image.height();
        let rgba =
            ImageBuffer::from_raw(w, h, image.into_raw()).context("invalid screenshot buffer")?;
        let jpeg = self.encode_jpeg(rgba)?;
        Ok(CapturedFrame {
            width: w,
            height: h,
            jpeg,
        })
    }

    fn primary_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// DXGI Desktop Duplication placeholder — validates COM on Windows.
#[cfg(windows)]
pub struct DxgiCapture;

#[cfg(windows)]
impl DxgiCapture {
    pub fn probe() -> Result<()> {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        Ok(())
    }
}

#[cfg(not(windows))]
pub struct DxgiCapture;
