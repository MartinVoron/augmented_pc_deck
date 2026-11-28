//! Extraction de l'icône d'un fichier, telle que l'Explorateur Windows l'affiche.
//!
//! Sous Windows, on demande l'image au shell (`IShellItemImageFactory`), ce qui marche pour
//! les programmes, les raccourcis `.lnk`, les dossiers et tous les types de fichiers.
//! L'image est convertie en PNG et renvoyée sous forme de `data:` URL, prête pour une `<img>`.

use std::path::Path;

use base64::Engine;

use crate::error::{AppError, AppResult};

/// Taille de l'icône demandée, en pixels (assez grand pour un écran à 200 %).
pub const ICON_SIZE: i32 = 64;

/// Image en pixels RGBA, ligne par ligne, de haut en bas.
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Convertit des pixels BGRA à alpha prémultiplié (format de Windows) en RGBA classique.
/// Une image sans aucune transparence renseignée est rendue opaque.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn bgra_premultiplied_to_rgba(mut pixels: Vec<u8>) -> Vec<u8> {
    let has_alpha = pixels.chunks_exact(4).any(|p| p[3] != 0);
    for p in pixels.chunks_exact_mut(4) {
        p.swap(0, 2);
        if !has_alpha {
            p[3] = 255;
        } else if p[3] != 0 && p[3] != 255 {
            let alpha = u32::from(p[3]);
            for c in &mut p[..3] {
                *c = (u32::from(*c) * 255 / alpha).min(255) as u8;
            }
        }
    }
    pixels
}

/// Encode une image RGBA en PNG, puis en `data:` URL.
pub fn to_png_data_url(image: &Rgba) -> AppResult<String> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let png_error = |e: png::EncodingError| AppError::Other(format!("icône illisible : {e}"));
        let mut writer = encoder.write_header().map_err(png_error)?;
        writer.write_image_data(&image.pixels).map_err(png_error)?;
    }
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{encoded}"))
}

/// Renvoie l'icône de `path` en `data:` URL, ou `None` si Windows n'en donne pas.
pub fn extract_data_url(path: &Path) -> AppResult<Option<String>> {
    match platform::extract(path, ICON_SIZE) {
        Some(image) => to_png_data_url(&image).map(Some),
        None => Ok(None),
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::path::Path;

    use windows::core::HSTRING;
    use windows::Win32::Foundation::SIZE;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
    };
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::{
        IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
    };

    use super::{bgra_premultiplied_to_rgba, Rgba};

    pub fn extract(path: &Path, size: i32) -> Option<Rgba> {
        // SAFETY : chaque appel COM ou GDI est suivi de la libération de ce qu'il a créé.
        unsafe {
            let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let image = shell_image(path, size);
            if com.is_ok() {
                CoUninitialize();
            }
            image
        }
    }

    unsafe fn shell_image(path: &Path, size: i32) -> Option<Rgba> {
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None)
                .inspect_err(|e| log::warn!("icône de {} : {e}", path.display()))
                .ok()?;
        let bitmap = factory
            .GetImage(
                SIZE { cx: size, cy: size },
                SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
            )
            .inspect_err(|e| log::warn!("icône de {} : {e}", path.display()))
            .ok()?;
        let image = read_bitmap(bitmap);
        let _ = DeleteObject(bitmap.into());
        image
    }

    unsafe fn read_bitmap(bitmap: HBITMAP) -> Option<Rgba> {
        let mut info = BITMAP::default();
        let read = GetObjectW(
            bitmap.into(),
            size_of::<BITMAP>() as i32,
            Some(&mut info as *mut BITMAP as *mut c_void),
        );
        if read == 0 || info.bmWidth <= 0 || info.bmHeight <= 0 {
            return None;
        }
        let (width, height) = (info.bmWidth, info.bmHeight);
        let mut header = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Hauteur négative : lignes de haut en bas.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let dc = CreateCompatibleDC(None);
        let lines = GetDIBits(
            dc,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut header,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(dc);
        (lines == height).then(|| Rgba {
            width: width as u32,
            height: height as u32,
            pixels: bgra_premultiplied_to_rgba(pixels),
        })
    }
}

#[cfg(not(windows))]
mod platform {
    use std::path::Path;

    use super::Rgba;

    /// Hors de Windows, pas d'icône pour un fichier quelconque : la tuile affiche l'icône
    /// du module. (Les fichiers `.desktop` ont la leur, voir `desktop.rs`.)
    pub fn extract(_path: &Path, _size: i32) -> Option<Rgba> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_bgra_and_removes_premultiplied_alpha() {
        // Un pixel bleu opaque, un pixel rouge à moitié transparent (prémultiplié).
        let pixels = bgra_premultiplied_to_rgba(vec![255, 0, 0, 255, 0, 0, 100, 128]);
        assert_eq!(pixels, vec![0, 0, 255, 255, 199, 0, 0, 128]);
    }

    #[test]
    fn image_without_alpha_becomes_opaque() {
        let pixels = bgra_premultiplied_to_rgba(vec![10, 20, 30, 0]);
        assert_eq!(pixels, vec![30, 20, 10, 255]);
    }

    #[test]
    fn encodes_a_png_data_url() {
        let image = Rgba {
            width: 1,
            height: 1,
            pixels: vec![1, 2, 3, 255],
        };
        let url = to_png_data_url(&image).unwrap();
        assert!(url.starts_with("data:image/png;base64,iVBORw0KGgo"));
    }
}
