use cgmath::Angle;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotLightUniform {
    pub position: [f32; 3],
    cut_off: f32,
    pub direction: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    // padding for 16 bytes align
    _padding: u32,
}
impl Default for SpotLightUniform {
    fn default() -> Self {
        Self { 
            position: Default::default(), 
            cut_off: cgmath::Deg(12.5).cos(),
            direction: Default::default(), 
            intensity: Default::default(), 
            color: Default::default(), 
            _padding: 0 
        }
    }
}

impl SpotLightUniform {
    pub fn new(position: [f32; 3], direction: [f32; 3], color: [f32; 3], intensity: f32, cut_off: f32) -> SpotLightUniform {
        SpotLightUniform {
            position,
            cut_off,
            direction,
            intensity,
            color,
            ..Default::default()
        }
    }
}