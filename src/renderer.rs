use nalgebra::Vector2;
use wgpu_text::font::FontRef;
use wgpu_text::section::Section;
use winit::window::Window;

use crate::base_renderer::BaseRenderer;
use crate::sprite::{Sprite, SpriteBatch, SpriteRenderer};
use crate::world::{World, WORLD_SIZE};
use crate::worm::Worm;

const WORLD_TEXTURE_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: WORLD_SIZE,
    height: WORLD_SIZE,
    depth_or_array_layers: 1,
};

pub struct Renderer {
    sprite_renderer: SpriteRenderer,
    base: BaseRenderer,
    world_texture: wgpu::Texture,
    world_bind_group: wgpu::BindGroup,
    circle_bind_group: wgpu::BindGroup,
    text_brush: wgpu_text::TextBrush<FontRef<'static>>,
}

impl Renderer {
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.base.size
    }

    fn load_world(&mut self, world: &World) {
        self.base.queue.write_texture(
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
        worms: &Vec<Worm>,
        text_sections: &[Section],
    ) -> Result<(), wgpu::SurfaceError> {
        self.load_world(world);

        let output = self.base.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut worm_sprites = Vec::<Sprite>::new();
        for worm in worms.iter() {
            worm_sprites.push(position_to_sprite(&worm.head.0));

            for segment in worm.segments.iter() {
                worm_sprites.push(position_to_sprite(&segment.0));
            }
        }

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
                    sprites: worm_sprites,
                    texture_bind_group: &self.circle_bind_group,
                },
            ],
            &self.base.device,
            &self.base.queue,
            &view,
            [self.base.size.width as f32, self.base.size.height as f32],
        )];

        // text
        for section in text_sections.iter() {
            self.text_brush.queue(section);
            command_buffers.push(
                self.text_brush
                    .draw(&self.base.device, &view, &self.base.queue),
            );
        }

        self.base.queue.submit(command_buffers);
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.base.window
    }

    pub async fn new(window: Window) -> Self {
        let base = BaseRenderer::new(window).await;

        let world_texture = base.device.create_texture(&wgpu::TextureDescriptor {
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
        let circle_texture_view = load_pixel_png(&base.device, &base.queue);

        let pixel_art_sampler = base.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sprite_renderer = SpriteRenderer::new(
            &base.config,
            &base.device,
            base.size.width as f32,
            base.size.height as f32,
        );

        let world_bind_group = sprite_renderer.create_texture_bind_group(
            &base.device,
            &pixel_art_sampler,
            &world_texture_view,
        );

        let circle_bind_group = sprite_renderer.create_texture_bind_group(
            &base.device,
            &pixel_art_sampler,
            &circle_texture_view,
        );

        let text_brush = wgpu_text::BrushBuilder::using_font_bytes(include_bytes!(
            "../assets/FiraCode-Regular.ttf"
        ))
        .unwrap()
        .build(&base.device, &base.config);

        Self {
            text_brush,
            sprite_renderer,
            world_texture,
            circle_bind_group,
            world_bind_group,
            base,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.text_brush.resize_view(
                new_size.width as f32,
                new_size.height as f32,
                &self.base.queue,
            );
            self.base.resize(new_size);
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

fn position_to_sprite(position: &Vector2<f32>) -> Sprite {
    Sprite {
        position: [position.x, position.y],
        size: [10.0, 10.0],
    }
}
