/// Placeholder for `net.minecraft.world.level.levelgen.DensityFunction`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DensityFunction {
    pub id: String,
}

impl DensityFunction {
    /// Stub for `DensityFunction#mapAll`, used by `NoiseRouter::map_all`.
    pub fn map_all<F: FnMut(&DensityFunction) -> DensityFunction>(&self, visitor: &mut F) -> Self {
        visitor(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoiseRouter {
    pub barrier_noise: DensityFunction,
    pub fluid_level_floodedness_noise: DensityFunction,
    pub fluid_level_spread_noise: DensityFunction,
    pub lava_noise: DensityFunction,
    pub temperature: DensityFunction,
    pub vegetation: DensityFunction,
    pub continents: DensityFunction,
    pub erosion: DensityFunction,
    pub depth: DensityFunction,
    pub ridges: DensityFunction,
    pub preliminary_surface_level: DensityFunction,
    pub final_density: DensityFunction,
    pub vein_toggle: DensityFunction,
    pub vein_ridged: DensityFunction,
    pub vein_gap: DensityFunction,
}

impl NoiseRouter {
    // JAVA: net.minecraft.world.level.levelgen.NoiseRouter#<init>
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        barrier_noise: DensityFunction,
        fluid_level_floodedness_noise: DensityFunction,
        fluid_level_spread_noise: DensityFunction,
        lava_noise: DensityFunction,
        temperature: DensityFunction,
        vegetation: DensityFunction,
        continents: DensityFunction,
        erosion: DensityFunction,
        depth: DensityFunction,
        ridges: DensityFunction,
        preliminary_surface_level: DensityFunction,
        final_density: DensityFunction,
        vein_toggle: DensityFunction,
        vein_ridged: DensityFunction,
        vein_gap: DensityFunction,
    ) -> Self {
        Self {
            barrier_noise,
            fluid_level_floodedness_noise,
            fluid_level_spread_noise,
            lava_noise,
            temperature,
            vegetation,
            continents,
            erosion,
            depth,
            ridges,
            preliminary_surface_level,
            final_density,
            vein_toggle,
            vein_ridged,
            vein_gap,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseRouter#mapAll
    pub fn map_all<F: FnMut(&DensityFunction) -> DensityFunction>(&self, visitor: &mut F) -> Self {
        Self {
            barrier_noise: self.barrier_noise.map_all(visitor),
            fluid_level_floodedness_noise: self.fluid_level_floodedness_noise.map_all(visitor),
            fluid_level_spread_noise: self.fluid_level_spread_noise.map_all(visitor),
            lava_noise: self.lava_noise.map_all(visitor),
            temperature: self.temperature.map_all(visitor),
            vegetation: self.vegetation.map_all(visitor),
            continents: self.continents.map_all(visitor),
            erosion: self.erosion.map_all(visitor),
            depth: self.depth.map_all(visitor),
            ridges: self.ridges.map_all(visitor),
            preliminary_surface_level: self.preliminary_surface_level.map_all(visitor),
            final_density: self.final_density.map_all(visitor),
            vein_toggle: self.vein_toggle.map_all(visitor),
            vein_ridged: self.vein_ridged.map_all(visitor),
            vein_gap: self.vein_gap.map_all(visitor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn df(id: &str) -> DensityFunction {
        DensityFunction {
            id: id.to_string(),
        }
    }

    fn sample_router() -> NoiseRouter {
        NoiseRouter::new(
            df("barrier"),
            df("floodedness"),
            df("spread"),
            df("lava"),
            df("temperature"),
            df("vegetation"),
            df("continents"),
            df("erosion"),
            df("depth"),
            df("ridges"),
            df("surface"),
            df("final"),
            df("vein_toggle"),
            df("vein_ridged"),
            df("vein_gap"),
        )
    }

    #[test]
    fn new_stores_fields() {
        let r = sample_router();
        assert_eq!(r.barrier_noise, df("barrier"));
        assert_eq!(r.fluid_level_floodedness_noise, df("floodedness"));
        assert_eq!(r.preliminary_surface_level, df("surface"));
        assert_eq!(r.final_density, df("final"));
        assert_eq!(r.vein_gap, df("vein_gap"));
    }

    #[test]
    fn map_all_transforms_all_fields() {
        let r = sample_router();
        let mut visitor = |d: &DensityFunction| DensityFunction {
            id: format!("{}!", d.id),
        };
        let m = r.map_all(&mut visitor);

        assert_eq!(m.barrier_noise, df("barrier!"));
        assert_eq!(m.fluid_level_floodedness_noise, df("floodedness!"));
        assert_eq!(m.fluid_level_spread_noise, df("spread!"));
        assert_eq!(m.lava_noise, df("lava!"));
        assert_eq!(m.temperature, df("temperature!"));
        assert_eq!(m.vegetation, df("vegetation!"));
        assert_eq!(m.continents, df("continents!"));
        assert_eq!(m.erosion, df("erosion!"));
        assert_eq!(m.depth, df("depth!"));
        assert_eq!(m.ridges, df("ridges!"));
        assert_eq!(m.preliminary_surface_level, df("surface!"));
        assert_eq!(m.final_density, df("final!"));
        assert_eq!(m.vein_toggle, df("vein_toggle!"));
        assert_eq!(m.vein_ridged, df("vein_ridged!"));
        assert_eq!(m.vein_gap, df("vein_gap!"));
    }

    #[test]
    fn map_all_visits_each_field_once() {
        let r = sample_router();
        let mut count: i32 = 0;
        let mut visitor = |d: &DensityFunction| {
            count = count.wrapping_add(1);
            d.clone()
        };
        let m = r.map_all(&mut visitor);
        assert_eq!(count, 15);
        assert_eq!(m, r);
    }

    #[test]
    fn map_all_preserves_field_order() {
        let r = sample_router();
        let mut idx: i32 = 0;
        let mut visitor = |_d: &DensityFunction| {
            let out = DensityFunction {
                id: idx.to_string(),
            };
            idx = idx.wrapping_add(1);
            out
        };
        let m = r.map_all(&mut visitor);
        assert_eq!(m.barrier_noise.id, "0");
        assert_eq!(m.fluid_level_floodedness_noise.id, "1");
        assert_eq!(m.fluid_level_spread_noise.id, "2");
        assert_eq!(m.lava_noise.id, "3");
        assert_eq!(m.temperature.id, "4");
        assert_eq!(m.vegetation.id, "5");
        assert_eq!(m.continents.id, "6");
        assert_eq!(m.erosion.id, "7");
        assert_eq!(m.depth.id, "8");
        assert_eq!(m.ridges.id, "9");
        assert_eq!(m.preliminary_surface_level.id, "10");
        assert_eq!(m.final_density.id, "11");
        assert_eq!(m.vein_toggle.id, "12");
        assert_eq!(m.vein_ridged.id, "13");
        assert_eq!(m.vein_gap.id, "14");
    }
}
