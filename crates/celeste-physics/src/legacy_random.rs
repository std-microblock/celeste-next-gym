//! `System.Random` as `Calc.Random` uses it (`Monocle/Calc.cs:19`).
//!
//! `FloatySpaceBlock`'s constructor draws its `sineWave` from `Calc.Random` unless the map set
//! `disableSpawnOffset` (`FloatySpaceBlock.cs:50-57`), and `Level.LoadLevel` re-seeds that
//! generator to the level's `LoadSeed` for the whole of the room build (`Level.cs:386-1386`).
//! Reproducing the draw therefore needs the exact generator the game ran, which on Celeste's
//! .NET Framework 4.8 FNA build is the *legacy subtractive* `System.Random`, not the xoshiro
//! generator .NET 6+ substituted. The algorithm below is that generator: a 56-entry seed array,
//! a 55-element subtractive recurrence, and `Sample() = InternalSample() * (1.0 / int.MaxValue)`.
//!
//! Only what a map build needs is implemented: the seeded constructor, `InternalSample` and
//! `NextDouble` (which is `Sample`). `Next`/`Next(maxValue)` are not modelled - no
//! `FloatySpaceBlock` code path uses them.

/// `System.Random`'s legacy subtractive generator.
pub(crate) struct LegacyRandom {
    /// `SeedArray`, 1-based like the BCL field (index 0 is unused).
    seed_array: [i32; 56],
    inext: i32,
    inextp: i32,
}

/// `System.Random.MBIG` / `MSEED`.
const MBIG: i32 = i32::MAX;
const MSEED: i32 = 161803398;

impl LegacyRandom {
    /// `Random(int Seed)` from the .NET Framework reference source.
    pub(crate) fn new(seed: i32) -> Self {
        let subtraction = if seed == i32::MIN { i32::MAX } else { seed.abs() };
        let mut seed_array = [0i32; 56];
        let mut mj = MSEED - subtraction;
        seed_array[55] = mj;
        let mut mk = 1;
        for i in 1..55 {
            let ii = 21 * i % 55;
            seed_array[ii] = mk;
            mk = mj - mk;
            if mk < 0 {
                mk += MBIG;
            }
            mj = seed_array[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                seed_array[i] -= seed_array[1 + (i + 30) % 55];
                if seed_array[i] < 0 {
                    seed_array[i] += MBIG;
                }
            }
        }
        Self {
            seed_array,
            inext: 0,
            inextp: 21,
        }
    }

    /// `System.Random.InternalSample`.
    fn internal_sample(&mut self) -> i32 {
        let mut inext = self.inext + 1;
        if inext >= 56 {
            inext = 1;
        }
        let mut inextp = self.inextp + 1;
        if inextp >= 56 {
            inextp = 1;
        }
        let mut sample = self.seed_array[inext as usize] - self.seed_array[inextp as usize];
        if sample == MBIG {
            sample -= 1;
        }
        if sample < 0 {
            sample += MBIG;
        }
        self.seed_array[inext as usize] = sample;
        self.inext = inext;
        self.inextp = inextp;
        sample
    }

    /// `System.Random.NextDouble` (`Sample`): `InternalSample() * (1.0 / MBIG)`.
    pub(crate) fn next_double(&mut self) -> f64 {
        self.internal_sample() as f64 * (1.0 / MBIG as f64)
    }

    /// `Calc.NextFloat(this Random, float max)` (`Calc.cs:415-418`): `(float)NextDouble() * max`.
    pub(crate) fn next_float_max(&mut self, max: f32) -> f32 {
        self.next_double() as f32 * max
    }
}

/// `LevelData.LoadSeed` (`LevelData.cs:99-111`): the sum of the level name's char codes, where
/// the name is the map's `name` attribute with its first four characters (`lvl_`) stripped
/// (`LevelData.cs:120-122`).
pub fn load_seed(level_name: &str) -> i32 {
    level_name.chars().skip(4).map(|c| c as i32).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first `NextDouble()` of `new Random(0)` on .NET Framework is the published anchor for
    /// the legacy algorithm; the following values pin the recurrence itself.
    #[test]
    fn legacy_random_matches_system_random() {
        let mut random = LegacyRandom::new(0);
        assert_eq!(random.next_double(), 0.7262432699679598);
        assert_eq!(random.next_double(), 0.81732535959096875);
        assert_eq!(random.next_double(), 0.76802268939466343);
        // `LoadSeed` of `e-00z` (`LevelData.cs:99-111`).
        let mut seeded = LegacyRandom::new(364);
        assert_eq!(seeded.next_double(), 0.88905763481234085);
    }

    #[test]
    fn load_seed_sums_the_room_name() {
        // `lvl_e-00z` -> `e-00z` -> 101 + 45 + 48 + 48 + 122.
        assert_eq!(load_seed("lvl_e-00z"), 364);
    }
}
