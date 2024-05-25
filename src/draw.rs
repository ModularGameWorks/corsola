use anyhow::{anyhow, Result};
use glyphon::{
    cosmic_text::Align, fontdb::Source, Attrs, Buffer, Color, FontSystem, Metrics, Resolution,
    Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Wrap,
};
use pixels::{wgpu::MultisampleState, Pixels, SurfaceTexture};
use tiny_skia::{Mask, Pixmap, PixmapPaint, Transform};
use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, TextureFormat};
use winit::{
    error::OsError,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

#[cfg(target_os = "android")]
const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
#[cfg(not(target_os = "android"))]
const FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

pub struct Renderer {
    pub pixels: Pixels,
    pub surface: Pixmap,
    text_renderers: Vec<TextRenderer>,
    num_text: usize,
    font_atlas: TextAtlas,
    glyph_cache: SwashCache,
    font_sys: Option<FontSystem>,
    fonts: Vec<Source>,
}

impl Renderer {
    pub fn new(window: &'_ Window) -> Result<Self> {
        let win_size = window.inner_size();

        let surface = Pixmap::new(win_size.width, win_size.height)
            .ok_or(anyhow!("Error initialising Pixmap surface"))?;

        let pixels = {
            let surf_tex = SurfaceTexture::new(win_size.width, win_size.height, window);

            Pixels::new(win_size.width, win_size.height, surf_tex)?
        };
        let device = pixels.device();
        let queue = pixels.queue();
        let font_atlas = TextAtlas::new(device, queue, FORMAT);
        // let text_renderer =
        //     TextRenderer::new(&mut font_atlas, device, MultisampleState::default(), None);

        Ok(Self {
            pixels,
            surface,
            text_renderers: Vec::new(),
            num_text: 0,
            font_atlas,
            // clear_colour: wgpu::Color {
            //     r: 0.0,
            //     g: 0.0,
            //     b: 0.0,
            //     a: 0.0,
            // },
            glyph_cache: SwashCache::new(),
            font_sys: None,
            fonts: Vec::new(),
        })
    }

    pub fn blit(
        &mut self,
        x: i32,
        y: i32,
        pixmap: &Pixmap,
        paint: &PixmapPaint,
        transform: Transform,
        mask: Option<&Mask>,
    ) {
        self.surface
            .draw_pixmap(x, y, pixmap.as_ref(), paint, transform, mask);
    }

    pub fn update(&mut self) -> Result<()> {
        self.pixels.frame_mut().copy_from_slice(self.surface.data());
        // self.pixels.render()?;

        self.pixels.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("text rendering"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        // store: StoreOp::Discard,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            for t_rend in &self.text_renderers[..self.num_text] {
                t_rend
                    .render(&self.font_atlas, &mut pass)
                    .map_err(|e| anyhow!("Error rendering text {:?}", e))?;
            }
            // queue.submit(Some(encoder.finish()));

            Ok(())
        })?;

        self.num_text = 0;
        // self.text_renderers.clear();

        Ok(())
    }

    pub fn load_fonts(&mut self, fonts: impl IntoIterator<Item = Source>, update: bool) {
        self.fonts.extend(fonts);
        if update {
            self.font_sys = Some(FontSystem::new_with_fonts(self.fonts.clone().into_iter()));
        }
    }

    pub fn text_ex(
        &mut self,
        txt: &str,
        x: f32,
        y: f32,
        font_size: f32,
        params: TextParams,
    ) -> Result<()> {
        if self.font_sys.is_none() {
            self.font_sys = Some(FontSystem::new_with_fonts(self.fonts.clone().into_iter()));
        }
        if let Some(fonts) = &mut self.font_sys {
            let device = self.pixels.device();
            let queue = self.pixels.queue();
            let tex = &self.pixels.context().texture;
            let width = tex.width();
            let height = tex.height();
            let line_height = match params.line_height {
                Some(lh) => lh,
                None => font_size * 1.5,
            };
            let mut buf = Buffer::new(fonts, Metrics::new(font_size, line_height));
            buf.set_wrap(fonts, params.wrap);
            let (dimx, dimy) = match params.dimensions {
                Some((x, y)) => {
                    // buf.set_size(fonts, x, y);
                    (x, y)
                }
                None => (width as f32, height as f32),
            };
            buf.set_size(fonts, dimx, dimy);
            buf.set_text(fonts, txt, params.attrs, params.shaping);

            for line in buf.lines.iter_mut() {
                line.set_align(params.align);
            }

            buf.shape_until_scroll(fonts);

            if self.text_renderers.len() <= self.num_text {
                self.text_renderers.push(TextRenderer::new(
                    &mut self.font_atlas,
                    device,
                    MultisampleState::default(),
                    None,
                ));
            }
            let (boundsx, boundsy) = buf.size();
            self.text_renderers
                .get_mut(self.num_text)
                .unwrap()
                .prepare(
                    device,
                    queue,
                    fonts,
                    &mut self.font_atlas,
                    Resolution { width, height },
                    [TextArea {
                        buffer: &buf,
                        left: x,
                        top: y,
                        scale: params.scale,
                        bounds: match params.bounds {
                            Some(bounds) => bounds,
                            None => TextBounds {
                                left: x as i32,
                                top: y as i32,
                                right: boundsx as i32,
                                bottom: boundsy as i32,
                                // right: width as i32,
                                // bottom: (y + line_height * buf.lines.len()) as i32,
                            },
                        },
                        default_color: params.colour,
                    }],
                    &mut self.glyph_cache,
                )?;
            self.num_text += 1;
        }
        Ok(())
    }

    pub fn text(&mut self, txt: &str, x: f32, y: f32, font_size: f32, colour: Color) -> Result<()> {
        self.text_ex(
            txt,
            x,
            y,
            font_size,
            TextParams {
                colour,
                ..Default::default()
            },
        )
    }
}

pub struct Surface {
    pub window: Window,
    pub renderer: Renderer,
}

pub fn new_window<T>(
    event_loop: &'_ EventLoop<T>,
    title: &str,
) -> std::result::Result<Window, OsError> {
    WindowBuilder::new()
        .with_title(title)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(event_loop)
}

pub fn new_window_ex<T>(
    event_loop: &'_ EventLoop<T>,
    title: &str,
    win_func: impl FnOnce(WindowBuilder) -> WindowBuilder,
) -> std::result::Result<Window, OsError> {
    let mut builder = WindowBuilder::new()
        .with_title(title)
        .with_fullscreen(Some(Fullscreen::Borderless(None)));

    builder = (win_func)(builder);

    builder.build(event_loop)
}

pub fn new_surface<T>(event_loop: &'_ EventLoop<T>, title: &str) -> Result<Surface> {
    let window = new_window(event_loop, title)?;

    Ok(Surface {
        window,
        renderer: Renderer::new(&window)?,
    })
}

pub fn new_surface_ex<T>(
    event_loop: &'_ EventLoop<T>,
    title: &str,
    win_func: impl FnOnce(WindowBuilder) -> WindowBuilder,
) -> Result<Surface> {
    let window = new_window_ex(event_loop, title, win_func)?;

    Ok(Surface {
        window,
        renderer: Renderer::new(&window)?,
    })
}

pub struct TextParams<'a> {
    pub attrs: Attrs<'a>,
    pub shaping: Shaping,
    pub align: Option<Align>,
    pub line_height: Option<f32>,
    pub wrap: Wrap,
    pub dimensions: Option<(f32, f32)>,
    pub scale: f32,
    pub bounds: Option<TextBounds>,
    pub colour: Color,
}

impl<'a> Default for TextParams<'a> {
    fn default() -> Self {
        Self {
            attrs: Attrs::new(),
            shaping: Shaping::Advanced,
            align: None,
            line_height: None,
            wrap: Wrap::Word,
            dimensions: None,
            scale: 1.0,
            bounds: None,
            colour: Color::rgb(255, 255, 255),
        }
    }
}
