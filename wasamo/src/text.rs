use windows::{
    core::Interface,
    Foundation::Size,
    Graphics::DirectX::{DirectXAlphaMode, DirectXPixelFormat},
    Win32::Graphics::{
        Direct2D::{
            D2D1CreateFactory, ID2D1DeviceContext, ID2D1Factory1, ID2D1SolidColorBrush,
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
        },
        Direct2D::Common::{D2D1_COLOR_F, D2D_POINT_2F},
        Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Direct3D11::{D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION},
        DirectWrite::{
            DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout,
            DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_TEXT_METRICS,
        },
        Dxgi::IDXGIDevice,
    },
    Win32::System::WinRT::Composition::ICompositionDrawingSurfaceInterop,
    UI::{
        Color,
        Composition::{CompositionDrawingSurface, CompositionGraphicsDevice, Compositor},
    },
};

/// Semantic typography style mapping to Windows type ramp constants (Segoe UI Variable).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TypographyStyle {
    Caption,   // 12sp regular
    Body,      // 14sp regular
    Subtitle,  // 20sp semi-bold
    Title,     // 28sp semi-bold
}

impl TypographyStyle {
    pub fn size_sp(self) -> f32 {
        match self {
            Self::Caption  => 12.0,
            Self::Body     => 14.0,
            Self::Subtitle => 20.0,
            Self::Title    => 28.0,
        }
    }

    pub fn is_semi_bold(self) -> bool {
        matches!(self, Self::Subtitle | Self::Title)
    }
}

/// Shared D2D/DWrite device resources created once for the process.
pub struct TextRenderer {
    dwrite_factory: IDWriteFactory,
    _d2d_factory: ID2D1Factory1,
    gfx_device: CompositionGraphicsDevice,
}

impl TextRenderer {
    pub fn new(compositor: &Compositor) -> windows::core::Result<Self> {
        // D3D11 device with BGRA support (required for D2D interop).
        let mut d3d_device: Option<ID3D11Device> = None;
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )?;
        }
        let d3d_device = d3d_device.unwrap();

        // D2D factory and device.
        let d2d_factory: ID2D1Factory1 =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        let dxgi_device: IDXGIDevice = d3d_device.cast()?;
        let d2d_device = unsafe { d2d_factory.CreateDevice(&dxgi_device)? };

        // CompositionGraphicsDevice wraps the D2D device for Visual Layer surface allocation.
        use windows::Win32::System::WinRT::Composition::ICompositorInterop;
        let compositor_interop: ICompositorInterop = compositor.cast()?;
        let gfx_device: CompositionGraphicsDevice =
            unsafe { compositor_interop.CreateGraphicsDevice(&d2d_device)? };

        // DWrite factory (shared — reused across all text layouts).
        let dwrite_factory: IDWriteFactory =
            unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };

        Ok(Self { dwrite_factory, _d2d_factory: d2d_factory, gfx_device })
    }

    /// Measure the natural (unconstrained) pixel size of a text string.
    pub fn measure(
        &self,
        text: &str,
        style: TypographyStyle,
    ) -> windows::core::Result<(f32, f32)> {
        let layout = self.create_text_layout(text, style, f32::MAX, f32::MAX)?;
        let mut metrics = DWRITE_TEXT_METRICS::default();
        unsafe { layout.GetMetrics(&mut metrics)? };
        Ok((metrics.widthIncludingTrailingWhitespace, metrics.height))
    }

    /// Allocate a `CompositionDrawingSurface` and draw `text` onto it.
    pub fn draw_text(
        &self,
        text: &str,
        style: TypographyStyle,
        width: f32,
        height: f32,
        color: Color,
    ) -> windows::core::Result<CompositionDrawingSurface> {
        let surface = self.gfx_device.CreateDrawingSurface(
            Size { Width: width.max(1.0), Height: height.max(1.0) },
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )?;

        let interop: ICompositionDrawingSurfaceInterop = surface.cast()?;
        let mut offset = windows::Win32::Foundation::POINT::default();
        let dc: ID2D1DeviceContext =
            unsafe { interop.BeginDraw(None, &mut offset)? };

        unsafe {
            dc.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
        }

        let color_f = D2D1_COLOR_F {
            r: color.R as f32 / 255.0,
            g: color.G as f32 / 255.0,
            b: color.B as f32 / 255.0,
            a: color.A as f32 / 255.0,
        };
        let brush: ID2D1SolidColorBrush =
            unsafe { dc.CreateSolidColorBrush(&color_f, None)? };

        let layout = self.create_text_layout(text, style, width, height)?;
        let origin = D2D_POINT_2F {
            x: offset.x as f32,
            y: offset.y as f32,
        };
        unsafe {
            dc.DrawTextLayout(origin, &layout, &brush, Default::default());
            interop.EndDraw()?;
        }

        Ok(surface)
    }

    fn create_text_layout(
        &self,
        text: &str,
        style: TypographyStyle,
        max_w: f32,
        max_h: f32,
    ) -> windows::core::Result<IDWriteTextLayout> {
        let font_name: Vec<u16> = "Segoe UI Variable\0".encode_utf16().collect();
        let locale: Vec<u16> = "en-us\0".encode_utf16().collect();
        let weight = if style.is_semi_bold() {
            DWRITE_FONT_WEIGHT_SEMI_BOLD
        } else {
            DWRITE_FONT_WEIGHT_NORMAL
        };
        let format: IDWriteTextFormat = unsafe {
            self.dwrite_factory.CreateTextFormat(
                windows::core::PCWSTR(font_name.as_ptr()),
                None,
                weight,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                style.size_sp(),
                windows::core::PCWSTR(locale.as_ptr()),
            )?
        };
        let text_w: Vec<u16> = text.encode_utf16().collect();
        let layout: IDWriteTextLayout = unsafe {
            self.dwrite_factory.CreateTextLayout(
                &text_w,
                &format,
                max_w,
                max_h,
            )?
        };
        Ok(layout)
    }
}
