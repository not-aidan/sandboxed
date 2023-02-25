use wgpu_text::font::FontRef;
use wgpu_text::section::Section;
use winit::window::Window;

use crate::sprite::{Sprite, SpriteBatch, SpriteRenderer};
use crate::world::{World, WORLD_SIZE};

const WORLD_TEXTURE_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: WORLD_SIZE,
    height: WORLD_SIZE,
    depth_or_array_layers: 1,
};

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    sprite_renderer: SpriteRenderer,
    world_texture: wgpu::Texture,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    world_bind_group: wgpu::BindGroup,
    circle_bind_group: wgpu::BindGroup,
    text_brush: wgpu_text::TextBrush<FontRef<'static>>,
}

impl Renderer {
    fn load_world(&mut self, world: &World) {
        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.world_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            world.pixels().as_slice(),
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * WORLD_SIZE),
                rows_per_image: std::num::NonZeroU32::new(WORLD_SIZE),
            },
            WORLD_TEXTURE_SIZE,
        );
    }

    pub fn render(
        &mut self,
        world: &World,
        text_sections: &[Section],
    ) -> Result<(), wgpu::SurfaceError> {
        self.load_world(world);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_buffers = vec![self.sprite_renderer.draw(
            &vec![
                SpriteBatch {
                    sprites: vec![Sprite {
                        position: [0.0, 0.0],
                        size: [200.0, 200.0],
                    }],
                    texture_bind_group: &self.world_bind_group,
                },
                SpriteBatch {
                    sprites: vec![Sprite {
                        position: [0.0, 0.0],
                        size: [10.0, 10.0],
                    }],
                    texture_bind_group: &self.circle_bind_group,
                },
            ],
            &self.device,
            &self.queue,
            &view,
            [self.size.width as f32, self.size.height as f32],
        )];

        // text
        for section in text_sections.iter() {
            self.text_brush.queue(section);
            command_buffers.push(self.text_brush.draw(&self.device, &view, &self.queue));
        }

        self.queue.submit(command_buffers);
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // Renderer owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.describe().srgb)
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let world_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: WORLD_TEXTURE_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB so we need to reflect that here.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("world_texture"),
            // This is the same as with the SurfaceConfig. It
            // specifies what texture formats can be used to
            // create TextureViews for this texture. The base
            // texture format (Rgba8UnormSrgb in this case) is
            // always supported. Note that using a different
            // texture format is not supported on the WebGL2
            // backend.
            view_formats: &[],
        });

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let world_texture_view = world_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let circle_texture_view = load_pixel_png(&device, &queue);

        let pixel_art_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sprite_renderer =
            SpriteRenderer::new(&config, &device, size.width as f32, size.height as f32);

        let world_bind_group = sprite_renderer.create_texture_bind_group(
            &device,
            &pixel_art_sampler,
            &world_texture_view,
        );

        let circle_bind_group = sprite_renderer.create_texture_bind_group(
            &device,
            &pixel_art_sampler,
            &circle_texture_view,
        );

        let text_brush = wgpu_text::BrushBuilder::using_font_bytes(include_bytes!(
            "../assets/FiraCode-Regular.ttf"
        ))
        .unwrap()
        .build(&device, &config);

        Self {
            text_brush,
            sprite_renderer,
            window,
            world_texture,
            circle_bind_group,
            world_bind_group,
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.text_brush
                .resize_view(new_size.width as f32, new_size.height as f32, &self.queue);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

fn load_pixel_png(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let diffuse_bytes = include_bytes!("../assets/circle.png");
    let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
    let diffuse_rgba = diffuse_image.to_rgba8();

    use image::GenericImageView;
    let dimensions = diffuse_image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        // All textures are stored as 3D, we represent our 2D texture
        // by setting depth to 1.
        size: texture_size,
        mip_level_count: 1, // We'll talk about this a little later
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Most images are stored using sRGB so we need to reflect that here.
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
        // COPY_DST means that we want to copy data to this texture
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("circle_texture"),
        // This is the same as with the SurfaceConfig. It
        // specifies what texture formats can be used to
        // create TextureViews for this texture. The base
        // texture format (Rgba8UnormSrgb in this case) is
        // always supported. Note that using a different
        // texture format is not supported on the WebGL2
        // backend.
        view_formats: &[],
    });

    queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::ImageCopyTexture {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &diffuse_rgba,
        // The layout of the texture
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
            rows_per_image: std::num::NonZeroU32::new(dimensions.1),
        },
        texture_size,
    );

    diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
