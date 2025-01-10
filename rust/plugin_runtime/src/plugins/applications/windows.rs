use std::{mem, ptr};
use std::io::Cursor;
use std::mem::MaybeUninit;
use crate::plugins::applications::{resize_icon, DesktopApplication, DesktopPathAction};
use deno_core::op2;
use std::path::PathBuf;
use anyhow::{anyhow, Context};
use image::RgbaImage;
use tokio::task::spawn_blocking;
use windows::core::{GUID, HSTRING, PWSTR};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Graphics::Gdi;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Storage::FileSystem;
use windows::Win32::UI::{Controls, Shell, WindowsAndMessaging};
use windows::Win32::UI::Controls::HIMAGELIST;

deno_core::extension!(
    gauntlet_internal_windows,
    ops = [
        windows_application_dirs,
        windows_open_application,
        windows_app_from_path,
    ],
    esm_entry_point = "ext:gauntlet/internal-windows/bootstrap.js",
    esm = [
        "ext:gauntlet/internal-windows/bootstrap.js" =  "../../js/bridge_build/dist/bridge-internal-windows-bootstrap.js",
        "ext:gauntlet/internal-windows.js" =  "../../js/core/dist/internal-windows.js",
    ]
);


#[op2]
#[serde]
fn windows_application_dirs() -> Vec<String> {
    vec![
        known_folder(&Shell::FOLDERID_Desktop),
        known_folder(&Shell::FOLDERID_PublicDesktop),
        known_folder(&Shell::FOLDERID_StartMenu),
        known_folder(&Shell::FOLDERID_CommonStartMenu),
    ]
        .into_iter()
        .flatten()
        .collect()
}

#[op2]
#[serde]
fn windows_open_application(#[string] file_path: String) -> anyhow::Result<()> {
    open::that_detached(file_path)?;

    Ok(())
}

#[op2(async)]
#[serde]
async fn windows_app_from_path(#[string] file_path: String) -> anyhow::Result<Option<DesktopPathAction>> {
    Ok(spawn_blocking(|| windows_app_from_path_blocking(file_path)).await?)
}

fn windows_app_from_path_blocking(file_path: String) -> anyhow::Result<Option<DesktopPathAction>> {
    if PathBuf::from(&file_path).exists() {
        let name = extract_name(&file_path)?;
        let icon = extract_icon(&file_path)
            .inspect_err(|err| tracing::error!("Unable to extract icon for {}: {:?}", file_path, err))
            .ok();

        Ok(Some(DesktopPathAction::Add {
            id: file_path.clone(),
            data: DesktopApplication {
                name,
                path: file_path,
                icon,
            },
        }))
    } else {
        Ok(Some(DesktopPathAction::Remove {
            id: file_path,
        }))
    }
}

fn extract_icon(file_path: &str) -> anyhow::Result<Vec<u8>> {
    unsafe {
        let mut shfileinfow = Shell::SHFILEINFOW::default();

        let hresult = Shell::SHGetFileInfoW(
            &HSTRING::from(file_path),
            FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfileinfow),
            size_of::<Shell::SHFILEINFOW>() as u32,
            Shell::SHGFI_SYSICONINDEX,
        );

        if hresult == 0 || shfileinfow.iIcon == 0 {
            return Err(anyhow!("SHGetFileInfoW failed: {}", hresult));
        }

        let image_list: Controls::IImageList = Shell::SHGetImageList(Shell::SHIL_JUMBO as i32)?;
        let icon = image_list.GetIcon(shfileinfow.iIcon, Controls::ILD_TRANSPARENT.0)?;

        let mut icon_info = WindowsAndMessaging::ICONINFO::default();
        WindowsAndMessaging::GetIconInfo(icon, &mut icon_info)
            .context("Unable to GetIconInfo")?;

        let _ = Gdi::DeleteObject(icon_info.hbmMask);

        let mut bitmap = Gdi::BITMAP::default();
        let result = Gdi::GetObjectW(
            icon_info.hbmColor,
            size_of::<Gdi::BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        );
        if result != size_of::<Gdi::BITMAP>() as i32 {
            Err(anyhow!("Error running GetObjectW: {}", result))?;
        }

        let size = (bitmap.bmWidthBytes * bitmap.bmHeight) as usize;
        let mut bits: Vec<u32> = vec![0; size];

        let dc = Gdi::GetDC(HWND(ptr::null_mut()));
        if dc == HDC(ptr::null_mut()) {
            Err(anyhow!("Error running GetDC"))?;
        }

        let mut bitmap_info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER {
                biSize: size_of::<Gdi::BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: Gdi::BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };

        let result = Gdi::GetDIBits(
            dc,
            icon_info.hbmColor,
            0,
            bitmap.bmHeight as u32,
            Some(bits.as_mut_ptr() as *mut _),
            &mut bitmap_info,
            Gdi::DIB_RGB_COLORS,
        );
        if result != bitmap.bmHeight {
            Err(anyhow!("Error running GetDIBits: {}", result))?;
        }

        let result = Gdi::ReleaseDC(HWND(ptr::null_mut()), dc);
        if result != 1 {
            Err(anyhow!("Error running ReleaseDC: {}", result))?;
        }

        let _ = Gdi::DeleteObject(icon_info.hbmColor);

        let image_buffer = RgbaImage::from_fn(bitmap.bmWidth as u32, bitmap.bmHeight as u32, |x, y| {
            let idx= y as usize * bitmap.bmWidth as usize + x as usize;
            let [b, g, r, a] = bits[idx].to_le_bytes();
            [r, g, b, a].into()
        });

        WindowsAndMessaging::DestroyIcon(icon)?;

        let rgba_image = image::DynamicImage::ImageRgba8(image_buffer);

        let mut result = Cursor::new(vec![]);

        rgba_image.write_to(&mut result, image::ImageFormat::Png)
            .expect("should be able to convert to png");

        let data = result.into_inner();

        let data = resize_icon(data)?;

        Ok(data)
    }
}


fn extract_name(file_path: &str) -> anyhow::Result<String> {
    let mut file_info = Shell::SHFILEINFOW::default();

    unsafe {
        Shell::SHGetFileInfoW(
            &HSTRING::from(file_path),
            FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut file_info),
            size_of::<Shell::SHFILEINFOW>() as u32,
            Shell::SHGFI_DISPLAYNAME,
        );
    }

    let strlen = file_info
        .szDisplayName
        .iter()
        .position(|x| *x == 0)
        .unwrap();

    Ok(String::from_utf16(&file_info.szDisplayName[..strlen])?)
}

pub fn known_folder(folder_id: &GUID) -> anyhow::Result<String> {
    let result: PWSTR = unsafe { Shell::SHGetKnownFolderPath(folder_id, Shell::KF_FLAG_CREATE, HANDLE::default()) }?;
    let result_str = unsafe { result.to_string() }?;
    Ok(result_str)
}
