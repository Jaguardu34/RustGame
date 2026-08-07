use noise::{NoiseFn, Perlin};

pub struct MapGenerator {
    perlin: Perlin,
    scale: f64,
    lacunarity: f64,
    persistance: f64,
    octaves: u32,
}

impl MapGenerator {
    pub fn new(seed: u32, scale: f64, lacunarity: f64, persistance: f64, octaves: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            scale,
            lacunarity,
            persistance,
            octaves,
        }
    }

    pub fn regenerate(
        &mut self,
        seed: u32,
        scale: f64,
        lacunarity: f64,
        persistance: f64,
        octaves: u32,
    ) {
        self.perlin = Perlin::new(seed);
        self.scale = scale;
        self.lacunarity = lacunarity;
        self.persistance = persistance;
        self.octaves = octaves;
    }

    pub fn get_height(&self, x: i64, y: i64) -> f64 {
        let mut total = 0.0;
        let mut max_amplitude = 0.0;
        let mut scale = self.scale;
        let mut amp = 1.0;

        for _ in 0..self.octaves {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            total += self.perlin.get([nx, ny]) * amp;

            max_amplitude += amp;
            scale *= self.lacunarity;
            amp *= self.persistance;
        }

        total / max_amplitude // normalise le résultat entre -1 et 1
    }
}
