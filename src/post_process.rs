use bevy::{
    core_pipeline::{Core3dSystems, FullscreenShader, schedule::Core3d},
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
    },
};

const SHADER_ASSET_PATH: &str = "shaders/post_processing.wgsl";

pub struct PostProcessPluginCustom;

impl Plugin for PostProcessPluginCustom {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // The settings will be a component that lives in the main world but will
            // be extracted to the render world every frame.
            // This makes it possible to control the effect from the main world.
            // This plugin will take care of extracting it automatically.
            // It's important to derive [`ExtractComponent`] on [`PostProcessSettings`]
            // for this plugin to work correctly.
            ExtractComponentPlugin::<CustomPostProcessSettings>::default(),
            // The settings will also be the data used in the shader.
            // This plugin will prepare the component for the GPU by creating a uniform buffer
            // and writing the data to that buffer every frame.
            UniformComponentPlugin::<CustomPostProcessSettings>::default(),
        ))
        .add_systems(Update, update_settings);

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_post_process_pipeline);
        render_app.add_systems(
            Core3d,
            post_process_system.in_set(Core3dSystems::PostProcess),
        );
    }
}

#[derive(Default)]
struct PostProcessBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

fn post_process_system(
    view: ViewQuery<(
        &ViewTarget,
        &CustomPostProcessSettings,
        &DynamicUniformIndex<CustomPostProcessSettings>,
    )>,
    post_process_pipeline: Option<Res<PostProcessPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<CustomPostProcessSettings>>,
    mut cache: Local<PostProcessBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(post_process_pipeline) = post_process_pipeline else {
        return;
    };

    let (view_target, _post_process_settings, settings_index) = view.into_inner();

    let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
    else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    // This will start a new "post process write", obtaining two texture
    // views from the view target - a `source` and a `destination`.
    // `source` is the "current" main texture and you _must_ write into
    // `destination` because calling `post_process_write()` on the
    // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
    // texture to the `destination` texture. Failing to do so will cause
    // the current main texture information to be lost.
    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            // The bind_group gets created each frame.
            //
            // Normally, you would create a bind_group in the Queue set,
            // but this doesn't work with the post_process_write().
            // The reason it doesn't work is because each post_process_write will alternate the source/destination.
            // The only way to have the correct source/destination for the bind_group
            // is to make sure you get it during the node execution.
            let bind_group = ctx.render_device().create_bind_group(
                "post_process_bind_group",
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                &BindGroupEntries::sequential((
                    // Make sure to use the source view
                    post_process.source,
                    // Use the sampler created for the pipeline
                    &post_process_pipeline.sampler,
                    // Set the settings binding
                    settings_binding.clone(),
                )),
            );

            let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
            bind_group
        }
    };

    let mut render_pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

    render_pass.set_pipeline(pipeline);
    // By passing in the index of the post process settings on this view, we ensure
    // that in the event that multiple settings were sent to the GPU (as would be the
    // case with multiple cameras), we use the correct one.
    render_pass.set_bind_group(0, bind_group, &[settings_index.index()]);
    render_pass.draw(0..3, 0..1);
}

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_post_process_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    // We need to define the bind group layout used for our pipeline
    let layout = BindGroupLayoutDescriptor::new(
        "post_process_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            // The layout entries will only be visible in the fragment stage
            ShaderStages::FRAGMENT,
            (
                // The screen texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                // The sampler that will be used to sample the screen texture
                sampler(SamplerBindingType::Filtering),
                // The settings uniform that will control the effect
                uniform_buffer::<CustomPostProcessSettings>(true),
            ),
        ),
    );
    // We can create the sampler here since it won't change at runtime and doesn't depend on the view
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    // Get the shader handle
    let shader = asset_server.load(SHADER_ASSET_PATH);
    // This will setup a fullscreen triangle for the vertex state.
    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache
        // This will add the pipeline to the cache and queue its creation
        .queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("post_process_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: vertex_state,
            fragment: Some(FragmentState {
                shader,
                // Make sure this matches the entry point of your shader.
                // It can be anything as long as it matches here and in the shader.
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        });
    commands.insert_resource(PostProcessPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct CustomPostProcessSettings {
    pub intensity: f32,
    // WebGL2 structs must be 16 byte aligned.
    // #[cfg(feature = "webgl2")]
    // _webgl2_padding: Vec3,
}

fn update_settings(mut settings: Query<&mut CustomPostProcessSettings>, time: Res<Time>) {
    for mut setting in &mut settings {
        let mut intensity = ops::sin(time.elapsed_secs());
        // Make it loop periodically
        intensity = ops::sin(intensity);
        // Remap it to 0..1 because the intensity can't be negative
        intensity = intensity * 0.5 + 0.5;
        // Scale it to a more reasonable level
        intensity *= 0.015;

        // Set the intensity.
        // This will then be extracted to the render world and uploaded to the GPU automatically by the [`UniformComponentPlugin`]
        setting.intensity = intensity;
    }
}
