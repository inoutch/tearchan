pub mod material_billboard;

pub struct Material<T> {
    provider: T,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl<'a, T> Material<T>
where
    T: MaterialProvider<'a>,
{
    pub fn new(device: &wgpu::Device, params: &T::Params, provider: T) -> Material<T> {
        let bind_group_layout = provider.create_bind_group_layout(device, params);
        let pipeline_layout = provider.create_pipeline_layout(device, params, &bind_group_layout);
        let pipeline = provider.create_pipeline(device, params, &pipeline_layout);
        let bind_group = provider.create_bind_group(device, params, &bind_group_layout);
        Material {
            provider,
            bind_group,
            pipeline,
        }
    }

    pub fn bind<'b>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
    }

    pub fn provider(&self) -> &T {
        &self.provider
    }
}

pub trait MaterialProvider<'a> {
    type Params: 'a;

    fn create_bind_group_layout(
        &self,
        device: &wgpu::Device,
        params: &Self::Params,
    ) -> wgpu::BindGroupLayout;

    fn create_pipeline_layout(
        &self,
        device: &wgpu::Device,
        params: &Self::Params,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout;

    fn create_pipeline(
        &self,
        device: &wgpu::Device,
        params: &Self::Params,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline;

    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        params: &Self::Params,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup;
}
