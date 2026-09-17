//! 中 / 英输入模式指示器：Win11 托盘品牌图标左边的模式图标。按微软 IME 的做法经 `GUID_LBI_INPUTMODE`
//! 语言栏按钮把图标交给系统（转换模式 compartment 不走这条通道，光写它不显示）。

use std::rc::Rc;

use windows::Win32::Foundation::{COLORREF, E_NOINTERFACE, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    ANTIALIASED_QUALITY, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLIP_DEFAULT_PRECIS, CreateBitmap,
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CENTER,
    DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW, FF_DONTCARE, FW_NORMAL, GdiFlush,
    OUT_TT_PRECIS, SelectObject, SetBkMode, SetTextColor, TRANSPARENT, VARIABLE_PITCH,
};
use windows::Win32::UI::TextServices::{
    GUID_LBI_INPUTMODE, ITfLangBarItem_Impl, ITfLangBarItemButton, ITfLangBarItemButton_Impl,
    ITfLangBarItemSink, ITfMenu, ITfSource, ITfSource_Impl, TF_LANGBARITEMINFO,
    TF_LBI_STYLE_BTN_BUTTON, TfLBIClick,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, GetSystemMetrics, HICON, ICONINFO, SM_CXSMICON,
};
use windows::core::{BOOL, BSTR, GUID, IUnknown, Interface, Ref, Result, implement, w};

use super::ModeState;
use crate::com::CLSID_QINGJIAN;

/// `GUID_LBI_INPUTMODE` 语言栏按钮：图标随 [`ModeState`] 显示中 / 英，点它切模式。
#[implement(ITfLangBarItemButton, ITfSource)]
pub(crate) struct ModeButton {
    state: Rc<ModeState>,
}

impl ModeButton {
    pub(crate) fn create(state: Rc<ModeState>) -> ITfLangBarItemButton {
        Self { state }.into()
    }
}

impl ITfLangBarItem_Impl for ModeButton_Impl {
    fn GetInfo(&self, pinfo: *mut TF_LANGBARITEMINFO) -> Result<()> {
        let info = unsafe { &mut *pinfo };
        info.clsidService = CLSID_QINGJIAN;
        info.guidItem = GUID_LBI_INPUTMODE;
        info.dwStyle = TF_LBI_STYLE_BTN_BUTTON;
        info.ulSort = 0;
        let desc: Vec<u16> = "青简中英模式".encode_utf16().collect();
        let n = desc.len().min(info.szDescription.len());
        info.szDescription[..n].copy_from_slice(&desc[..n]);
        Ok(())
    }

    fn GetStatus(&self) -> Result<u32> {
        Ok(0)
    }

    fn Show(&self, _fshow: BOOL) -> Result<()> {
        Ok(())
    }

    fn GetTooltipString(&self) -> Result<BSTR> {
        Ok(BSTR::from("中 / 英（单击 Shift 切换）"))
    }
}

impl ITfLangBarItemButton_Impl for ModeButton_Impl {
    fn OnClick(&self, _click: TfLBIClick, _pt: &POINT, _prcarea: *const RECT) -> Result<()> {
        crate::com::service::toggle_mode();
        Ok(())
    }

    fn InitMenu(&self, _pmenu: Ref<ITfMenu>) -> Result<()> {
        Ok(())
    }

    fn OnMenuSelect(&self, _wid: u32) -> Result<()> {
        Ok(())
    }

    fn GetIcon(&self) -> Result<HICON> {
        make_mode_icon(if self.state.english() { '英' } else { '中' })
    }

    fn GetText(&self) -> Result<BSTR> {
        Ok(BSTR::from(if self.state.english() { "英" } else { "中" }))
    }
}

impl ITfSource_Impl for ModeButton_Impl {
    fn AdviseSink(&self, riid: *const GUID, punk: Ref<IUnknown>) -> Result<u32> {
        if unsafe { *riid } != ITfLangBarItemSink::IID {
            return Err(E_NOINTERFACE.into());
        }
        let sink: ITfLangBarItemSink = punk.ok()?.cast()?;
        *self.state.sink.borrow_mut() = Some(sink);
        Ok(1) // 只支持一个回调，cookie 固定
    }

    fn UnadviseSink(&self, _dwcookie: u32) -> Result<()> {
        *self.state.sink.borrow_mut() = None;
        Ok(())
    }
}

/// 托盘图标的边长：按系统的小图标尺寸原样画（画大了再让系统缩只会糊），取不到按 16 算。
fn icon_size() -> i32 {
    unsafe { GetSystemMetrics(SM_CXSMICON) }.clamp(16, 64)
}

/// 字的颜色：跟着**任务栏**的深浅走（`SystemUsesLightTheme`，与应用深浅是两个设置）——
/// 浅色任务栏画近黑，深色任务栏画白。读不到按 Win11 缺省的深色任务栏算。
fn glyph_color() -> u32 {
    let light_taskbar = windows_registry::CURRENT_USER
        .open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_u32("SystemUsesLightTheme"))
        .is_ok_and(|value| value == 1);
    if light_taskbar {
        0x0019_1919
    } else {
        0x00FF_FFFF
    }
}

/// 透明底的「中」/「英」图标。灰度抗锯齿的覆盖率进 alpha 通道：ClearType 的彩色子像素在透明底上
/// 只能按「非零即不透明」当成实心像素，托盘里就是一团彩色毛边。像素是非预乘 alpha（与 ICO 一致）。
/// 系统取走 HICON 后负责销毁。
fn make_mode_icon(ch: char) -> Result<HICON> {
    let size = icon_size();
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
    unsafe {
        let color = CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
        let memdc = CreateCompatibleDC(None);
        let old_bmp = SelectObject(memdc, color.into());
        let font = CreateFontW(
            -size, // 负字高 = 精确字符高度；汉字的墨迹约 0.85 字高，居中画进来不会切边
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_TT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Microsoft YaHei UI"),
        );
        let old_font = SelectObject(memdc, font.into());
        let _ = SetBkMode(memdc, TRANSPARENT);
        // 一律先画白字：底是全 0，灰度就是这一像素的覆盖率，下面再换成真正的颜色。
        SetTextColor(memdc, COLORREF(0x00FF_FFFF));
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: size,
            bottom: size,
        };
        let mut text: Vec<u16> = ch.to_string().encode_utf16().collect();
        DrawTextW(
            memdc,
            &mut text,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        let _ = GdiFlush();
        // GDI 画字不写 alpha：把覆盖率搬进 alpha，颜色换成任务栏配色。
        let rgb = glyph_color();
        let pixels = std::slice::from_raw_parts_mut(bits.cast::<u32>(), (size * size) as usize);
        for p in pixels.iter_mut() {
            let drawn = *p;
            let coverage = (drawn & 0xFF)
                .max((drawn >> 8) & 0xFF)
                .max((drawn >> 16) & 0xFF);
            *p = if coverage == 0 {
                0
            } else {
                (coverage << 24) | rgb
            };
        }
        SelectObject(memdc, old_font);
        let _ = DeleteObject(font.into());
        SelectObject(memdc, old_bmp);
        let _ = DeleteDC(memdc);
        // 掩码全 0，透明靠 32bpp 的 alpha。
        let mask = CreateBitmap(size, size, 1, 1, None);
        let info = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&info);
        let _ = DeleteObject(mask.into());
        let _ = DeleteObject(color.into());
        icon
    }
}
