#[derive(Debug, Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
}

#[derive(Debug)]
pub struct Sphere {
    pub position: glam::Vec3,
    pub radius: f32,

    pub albedo: glam::Vec3,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            radius: 0.5,
            albedo: glam::Vec3::ONE,
        }
    }
}
