use crate::{
    get_error,
    gpu::{Device as GpuDevice, Texture as GpuTexture},
    libc::c_int,
    pixels::Color,
    render::TextureCreator,
    ttf::{
        sys::{
            TTF_CreateGPUTextEngine, TTF_CreateRendererTextEngine, TTF_CreateText,
            TTF_DestroyGPUTextEngine, TTF_DestroyRendererTextEngine, TTF_DestroyText,
            TTF_DrawRendererText, TTF_GPUAtlasDrawSequence, TTF_GetGPUTextDrawData,
            TTF_GetTextSize, TTF_ImageType, TTF_SetTextColor, TTF_SetTextFont, TTF_SetTextString,
            TTF_SetTextWrapWidth, TTF_Text, TTF_TextEngine, TTF_UpdateText,
        },
        Font,
    },
    Error,
};
use std::ffi::CString;
use std::marker::PhantomData;

/// Distinguishes which native `TTF_TextEngine` (Render API or GPU) backend a
/// [`TextEngine`]/[`Text`] was created with.
pub trait TextEngineBackend {
    #[doc(hidden)]
    unsafe fn destroy_engine(raw: *mut TTF_TextEngine);
}

/// Marker for a [`TextEngine`] that draws through an `SDL_Renderer`.
pub struct RendererBackend;

/// Marker for a [`TextEngine`] that draws through the SDL GPU API.
pub struct GpuBackend;

impl TextEngineBackend for RendererBackend {
    unsafe fn destroy_engine(raw: *mut TTF_TextEngine) {
        unsafe { TTF_DestroyRendererTextEngine(raw) };
    }
}

impl TextEngineBackend for GpuBackend {
    unsafe fn destroy_engine(raw: *mut TTF_TextEngine) {
        unsafe { TTF_DestroyGPUTextEngine(raw) };
    }
}

pub struct TextEngine<B: TextEngineBackend> {
    raw: *mut TTF_TextEngine,
    _marker: PhantomData<B>,
}

pub type GpuTextEngine = TextEngine<GpuBackend>;
pub type RendererTextEngine = TextEngine<RendererBackend>;

impl TextEngine<RendererBackend> {
    #[doc(alias = "TTF_CreateRendererTextEngine")]
    pub fn new<T>(creator: &TextureCreator<T>) -> Result<Self, Error> {
        let raw = unsafe { TTF_CreateRendererTextEngine(creator.raw()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Self {
                raw,
                _marker: PhantomData,
            })
        }
    }
}

impl TextEngine<GpuBackend> {
    #[doc(alias = "TTF_CreateGPUTextEngine")]
    pub fn new(device: &GpuDevice) -> Result<Self, Error> {
        let raw = unsafe { TTF_CreateGPUTextEngine(device.raw()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Self {
                raw,
                _marker: PhantomData,
            })
        }
    }
}

impl<B: TextEngineBackend> TextEngine<B> {
    pub fn raw(&self) -> *mut TTF_TextEngine {
        self.raw
    }

    #[doc(alias = "TTF_CreateText")]
    pub fn create_text(&self, font: &Font, text: &str) -> Result<Text<B>, Error> {
        let ctext = CString::new(text).unwrap();
        let raw =
            unsafe { TTF_CreateText(self.raw, font.raw(), ctext.as_ptr(), ctext.count_bytes()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Text {
                raw,
                _marker: PhantomData,
            })
        }
    }
}

impl<B: TextEngineBackend> Drop for TextEngine<B> {
    fn drop(&mut self) {
        unsafe { B::destroy_engine(self.raw) };
    }
}

pub struct Text<B: TextEngineBackend> {
    raw: *mut TTF_Text,
    _marker: PhantomData<B>,
}

impl<B: TextEngineBackend> Text<B> {
    pub fn raw(&self) -> *mut TTF_Text {
        self.raw
    }

    #[doc(alias = "TTF_UpdateText")]
    pub fn update(&mut self) -> Result<(), Error> {
        let ok = unsafe { TTF_UpdateText(self.raw) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_GetTextSize")]
    pub fn size(&self) -> (u32, u32) {
        let mut w: c_int = 0;
        let mut h: c_int = 0;
        unsafe { TTF_GetTextSize(self.raw, &mut w, &mut h) };
        (w as u32, h as u32)
    }

    #[doc(alias = "TTF_SetTextFont")]
    pub fn set_font(&mut self, font: &Font) -> Result<(), Error> {
        let ok = unsafe { TTF_SetTextFont(self.raw, font.raw()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_SetTextString")]
    pub fn set_text(&mut self, text: &str) -> Result<(), Error> {
        let ctext = CString::new(text).unwrap();
        let ok = unsafe { TTF_SetTextString(self.raw, ctext.as_ptr(), ctext.count_bytes()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_SetTextColor")]
    pub fn set_color<T>(&mut self, color: T) -> Result<(), Error>
    where
        T: Into<Color>,
    {
        let color: Color = color.into().into();
        let ok = unsafe { TTF_SetTextColor(self.raw, color.r, color.g, color.b, color.a) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_SetTextWrapWidth")]
    pub fn set_wrap_width(&self, pixels: i32) -> Result<(), Error> {
        let ok = unsafe { TTF_SetTextWrapWidth(self.raw, pixels) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

impl Text<RendererBackend> {
    #[doc(alias = "TTF_DrawRendererText")]
    pub fn draw(&self, x: f32, y: f32) -> Result<(), Error> {
        let ok = unsafe { TTF_DrawRendererText(self.raw, x, y) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

impl Text<GpuBackend> {
    #[doc(alias = "TTF_GetGPUTextDrawData")]
    pub fn draw_data(&self) -> Result<GpuAtlasDrawSequence<'_>, Error> {
        let raw = unsafe { TTF_GetGPUTextDrawData(self.raw) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(GpuAtlasDrawSequence {
                raw,
                _marker: PhantomData,
            })
        }
    }
}

impl<B: TextEngineBackend> Drop for Text<B> {
    fn drop(&mut self) {
        unsafe { TTF_DestroyText(self.raw) };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ImageType {
    Alpha = TTF_ImageType::ALPHA.0,
    Color = TTF_ImageType::COLOR.0,
    Sdf = TTF_ImageType::SDF.0,
}

impl ImageType {
    pub fn from_ll(raw: TTF_ImageType) -> Option<ImageType> {
        Some(match raw {
            TTF_ImageType::ALPHA => ImageType::Alpha,
            TTF_ImageType::COLOR => ImageType::Color,
            TTF_ImageType::SDF => ImageType::Sdf,
            _ => return None,
        })
    }
}

/// Geometry from [`Text::draw_data`], and a cursor over the rest of
/// the linked list SDL_ttf returns.
#[derive(Clone, Copy)]
pub struct GpuAtlasDrawSequence<'a> {
    raw: *mut TTF_GPUAtlasDrawSequence,
    _marker: PhantomData<&'a ()>,
}

impl<'a> GpuAtlasDrawSequence<'a> {
    fn node(&self) -> &TTF_GPUAtlasDrawSequence {
        unsafe { &*self.raw }
    }

    pub fn atlas_texture(&self) -> GpuTexture<'static> {
        GpuTexture::new_sdl_managed(self.node().atlas_texture, 0, 0)
    }

    pub fn image_type(&self) -> Option<ImageType> {
        ImageType::from_ll(self.node().image_type)
    }

    pub fn xy(&self) -> &'a [sys::rect::SDL_FPoint] {
        let node = self.node();
        unsafe { std::slice::from_raw_parts(node.xy, node.num_vertices as usize) }
    }

    pub fn uv(&self) -> &'a [sys::rect::SDL_FPoint] {
        let node = self.node();
        unsafe { std::slice::from_raw_parts(node.uv, node.num_vertices as usize) }
    }

    pub fn indices(&self) -> &'a [c_int] {
        let node = self.node();
        unsafe { std::slice::from_raw_parts(node.indices, node.num_indices as usize) }
    }

    /// Iterates over this sequence and every sequence after it, starting
    /// with `self`.
    pub fn iter(&self) -> GpuAtlasDrawSequenceIter<'a> {
        GpuAtlasDrawSequenceIter(Some(*self))
    }
}

pub struct GpuAtlasDrawSequenceIter<'a>(Option<GpuAtlasDrawSequence<'a>>);

impl<'a> Iterator for GpuAtlasDrawSequenceIter<'a> {
    type Item = GpuAtlasDrawSequence<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.0?;

        let next = unsafe { (*cur.raw).next };
        self.0 = (!next.is_null()).then(|| GpuAtlasDrawSequence {
            raw: next,
            _marker: PhantomData,
        });

        Some(cur)
    }
}
