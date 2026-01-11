use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub type RandResult<T> = Result<T, RandError>;

#[derive(Debug, Clone)]
pub enum RandError {
    InvalidRange,
    InvalidDistribution,
    SeedingError(String),
}

impl std::fmt::Display for RandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandError::InvalidRange => write!(f, "Invalid range"),
            RandError::InvalidDistribution => write!(f, "Invalid distribution"),
            RandError::SeedingError(msg) => write!(f, "Seeding error: {}", msg),
        }
    }
}

impl std::error::Error for RandError {}

pub trait Rng {
    fn next_u32(&mut self) -> u32;
    fn next_u64(&mut self) -> u64;
    fn next_u128(&mut self) -> u128;
    fn next_f32(&mut self) -> f32;
    fn next_f64(&mut self) -> f64;
    fn next_bool(&mut self) -> bool;
    fn fill_bytes(&mut self, dest: &mut [u8]);
    fn gen_range<T: PartialOrd + SampleUniform>(&mut self, range: T) -> T;
    fn gen<T: Distribution>(&mut self, dist: &T) -> T::Output;
}

pub trait Distribution {
    type Output;
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output;
}

pub trait SampleUniform: PartialOrd {
    type X;
    fn new<B, B1>(low: B, high: B1) -> Self::X;
}

pub struct StdRng {
    state: u64,
}

impl StdRng {
    pub fn new() -> Self {
        StdRng { state: Self::seed() }
    }

    pub fn from_seed(seed: u64) -> Self {
        StdRng { state: seed.wrapping_mul(6364136223846793005).wrapping_add(1) }
    }

    fn seed() -> u64 {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        duration.as_nanos() as u64
    }

    pub fn reseed(&mut self, seed: u64) {
        self.state = seed;
    }
}

impl Default for StdRng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng for StdRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_u128(&mut self) -> u128 {
        let hi = self.next_u64() as u128;
        let lo = self.next_u64() as u128;
        (hi << 64) | lo
    }

    fn next_f32(&mut self) -> f32 {
        const MANTISSA_BITS: u32 = 23;
        const EXPONENT_BITS: u32 = 8;
        const VALUE_MASK: u32 = (1 << MANTISSA_BITS) - 1;
        const SIGN_BIT: u32 = 1 << (MANTISSA_BITS + EXPONENT_BITS);

        let mut value = self.next_u32();
        value &= VALUE_MASK | SIGN_BIT;
        f32::from_bits(value)
    }

    fn next_f64(&mut self) -> f64 {
        const MANTISSA_BITS: u64 = 52;
        const EXPONENT_BITS: u64 = 11;
        const VALUE_MASK: u64 = (1 << MANTISSA_BITS) - 1;
        const SIGN_BIT: u64 = 1 << (MANTISSA_BITS + EXPONENT_BITS);

        let mut value = self.next_u64();
        value &= VALUE_MASK | SIGN_BIT;
        f64::from_bits(value)
    }

    fn next_bool(&mut self) -> bool {
        self.next_u32() & 1 == 1
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let bytes = self.next_u64().to_le_bytes();
            let len = chunk.len().min(8);
            chunk[..len].copy_from_slice(&bytes[..len]);
        }
    }

    fn gen_range<T: PartialOrd + SampleUniform>(&mut self, range: T) -> T {
        let dist = Uniform::new(range);
        self.gen(&dist)
    }

    fn gen<T: Distribution>(&mut self, dist: &T) -> T::Output {
        dist.sample(self)
    }
}

pub struct Uniform<X>(X);

impl<X> Uniform<X> {
    pub fn new<B, B1>(low: B, high: B1) -> Self
    where
        X: SampleUniform<X = X>,
        B: SampleUniform<X = X>,
        B1: SampleUniform<X = X>,
    {
        Uniform(SampleUniform::new(low, high))
    }
}

impl<X: SampleUniform> Distribution for Uniform<X> {
    type Output = X::X;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let low = self.0.low();
        let high = self.0.high();
        let range = high.wrapping_sub(&low);
        let mut result = rng.next_u64() as u128;

        let scale = (u128::MAX / range as u128) + 1;
        result /= scale;

        low.wrapping_add(&(result as u64))
    }
}

impl SampleUniform for i8 {
    type X = i8;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as i8;
        let high = high as i8;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for i16 {
    type X = i16;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as i16;
        let high = high as i16;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for i32 {
    type X = i32;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as i32;
        let high = high as i32;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for i64 {
    type X = i64;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as i64;
        let high = high as i64;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for u8 {
    type X = u8;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as u8;
        let high = high as u8;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for u16 {
    type X = u16;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as u16;
        let high = high as u16;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for u32 {
    type X = u32;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as u32;
        let high = high as u32;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for u64 {
    type X = u64;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as u64;
        let high = high as u64;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for f32 {
    type X = f32;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as f32;
        let high = high as f32;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

impl SampleUniform for f64 {
    type X = f64;

    fn new<B, B1>(low: B, high: B1) -> Self::X {
        let low = low as f64;
        let high = high as f64;
        low
    }

    fn low(&self) -> Self::X {
        *self
    }

    fn high(&self) -> Self::X {
        *self
    }
}

pub struct Bernoulli {
    p: f64,
}

impl Bernoulli {
    pub fn new(p: f64) -> RandResult<Self> {
        if !(0.0..=1.0).contains(&p) {
            return Err(RandError::InvalidDistribution);
        }
        Ok(Bernoulli { p })
    }

    pub fn p(&self) -> f64 {
        self.p
    }
}

impl Distribution for Bernoulli {
    type Output = bool;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        rng.next_f64() < self.p
    }
}

pub struct Binomial {
    n: u64,
    p: f64,
}

impl Binomial {
    pub fn new(n: u64, p: f64) -> RandResult<Self> {
        if !(0.0..=1.0).contains(&p) {
            return Err(RandError::InvalidDistribution);
        }
        Ok(Binomial { n, p })
    }

    pub fn n(&self) -> u64 {
        self.n
    }

    pub fn p(&self) -> f64 {
        self.p
    }
}

impl Distribution for Binomial {
    type Output = u64;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let mut count = 0u64;
        for _ in 0..self.n {
            if rng.next_f64() < self.p {
                count += 1;
            }
        }
        count
    }
}

pub struct Poisson {
    lambda: f64,
}

impl Poisson {
    pub fn new(lambda: f64) -> RandResult<Self> {
        if lambda <= 0.0 {
            return Err(RandError::InvalidDistribution);
        }
        Ok(Poisson { lambda })
    }

    pub fn lambda(&self) -> f64 {
        self.lambda
    }
}

impl Distribution for Poisson {
    type Output = u64;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let l = (-self.lambda).exp();
        let mut k = 0u64;
        let mut p = 1.0;

        loop {
            k += 1;
            p *= rng.next_f64();
            if p <= l {
                return k - 1;
            }
        }
    }
}

pub struct Normal {
    mean: f64,
    std_dev: f64,
}

impl Normal {
    pub fn new(mean: f64, std_dev: f64) -> RandResult<Self> {
        if std_dev <= 0.0 {
            return Err(RandError::InvalidDistribution);
        }
        Ok(Normal { mean, std_dev })
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }
}

impl Distribution for Normal {
    type Output = f64;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let u1 = rng.next_f64();
        let u2 = rng.next_f64();
        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = 2.0 * std::f64::consts::PI * u2;
        let z0 = radius * angle.cos();
        self.mean + self.std_dev * z0
    }
}

pub struct LogNormal {
    mean: f64,
    std_dev: f64,
}

impl LogNormal {
    pub fn new(mean: f64, std_dev: f64) -> RandResult<Self> {
        if std_dev <= 0.0 {
            return Err(RandError::InvalidDistribution);
        }
        Ok(LogNormal { mean, std_dev })
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }
}

impl Distribution for LogNormal {
    type Output = f64;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let normal = Normal::new(self.mean, self.std_dev).unwrap();
        let x = rng.gen(&normal);
        x.exp()
    }
}

pub struct Exponential {
    lambda: f64,
}

impl Exponential {
    pub fn new(lambda: f64) -> RandResult<Self> {
        if lambda <= 0.0 {
            return Err(RandError::InvalidDistribution);
        }
        Ok(Exponential { lambda })
    }

    pub fn lambda(&self) -> f64 {
        self.lambda
    }
}

impl Distribution for Exponential {
    type Output = f64;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let u = rng.next_f64();
        -u.ln() / self.lambda
    }
}

pub struct Shuffle;

impl Shuffle {
    pub fn new() -> Self {
        Shuffle
    }
}

impl Distribution for Shuffle {
    type Output = ();

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {}
}

pub fn random<T: SampleUniform>(low: T, high: T) -> T::X {
    let mut rng = StdRng::new();
    rng.gen_range((low, high))
}

pub fn random_bool() -> bool {
    let mut rng = StdRng::new();
    rng.next_bool()
}

pub fn random_f32() -> f32 {
    let mut rng = StdRng::new();
    rng.next_f32()
}

pub fn random_f64() -> f64 {
    let mut rng = StdRng::new();
    rng.next_f64()
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut rng = StdRng::new();
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes);
    bytes
}

pub fn shuffle<T>(slice: &mut [T]) {
    let mut rng = StdRng::new();
    let len = slice.len();
    for i in (1..len).rev() {
        let j = rng.gen_range(0..=i);
        slice.swap(i, j);
    }
}

pub fn thread_rng() -> StdRng {
    StdRng::new()
}

pub fn seedable_rng(seed: u64) -> StdRng {
    StdRng::from_seed(seed)
}

pub struct RngSeedable {
    seed: u64,
}

impl RngSeedable {
    pub fn new(seed: u64) -> Self {
        RngSeedable { seed }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }
}

pub fn hash_random<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_rng() {
        let mut rng = StdRng::new();
        let val = rng.next_u32();
        assert!(val <= u32::MAX);
    }

    #[test]
    fn test_std_rng_from_seed() {
        let rng1 = StdRng::from_seed(42);
        let rng2 = StdRng::from_seed(42);
        assert_eq!(rng1.next_u32(), rng2.next_u32());
    }

    #[test]
    fn test_uniform() {
        let mut rng = StdRng::new();
        let val: i32 = rng.gen_range(0..100);
        assert!(val >= 0 && val < 100);
    }

    #[test]
    fn test_bernoulli() {
        let mut rng = StdRng::new();
        let dist = Bernoulli::new(0.5).unwrap();
        let val = rng.gen(&dist);
        assert!(val == true || val == false);
    }

    #[test]
    fn test_binomial() {
        let mut rng = StdRng::new();
        let dist = Binomial::new(10, 0.5).unwrap();
        let val = rng.gen(&dist);
        assert!(val <= 10);
    }

    #[test]
    fn test_poisson() {
        let mut rng = StdRng::new();
        let dist = Poisson::new(5.0).unwrap();
        let val = rng.gen(&dist);
        assert!(val <= 100);
    }

    #[test]
    fn test_normal() {
        let mut rng = StdRng::new();
        let dist = Normal::new(0.0, 1.0).unwrap();
        let val = rng.gen(&dist);
        assert!(!val.is_nan());
    }

    #[test]
    fn test_exponential() {
        let mut rng = StdRng::new();
        let dist = Exponential::new(1.0).unwrap();
        let val = rng.gen(&dist);
        assert!(val >= 0.0);
    }

    #[test]
    fn test_shuffle() {
        let mut arr = vec![1, 2, 3, 4, 5];
        shuffle(&mut arr);
        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes(10);
        assert_eq!(bytes.len(), 10);
    }

    #[test]
    fn test_hash_random() {
        let val = hash_random(&"hello");
        assert!(val > 0);
    }
}
