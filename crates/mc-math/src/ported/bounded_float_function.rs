use std::sync::Arc;

/// Faithful port of `net.minecraft.util.BoundedFloatFunction`.
pub struct BoundedFloatFunction<C: ?Sized + 'static> {
    apply_fn: Arc<dyn Fn(&C) -> f32 + Send + Sync>,
    min_value: f32,
    max_value: f32,
}

impl<C: ?Sized + 'static> Clone for BoundedFloatFunction<C> {
    fn clone(&self) -> Self {
        Self {
            apply_fn: Arc::clone(&self.apply_fn),
            min_value: self.min_value,
            max_value: self.max_value,
        }
    }
}

impl<C: ?Sized + 'static> BoundedFloatFunction<C> {
    // JAVA: net.minecraft.util.BoundedFloatFunction#apply
    pub fn apply(&self, input: &C) -> f32 {
        (self.apply_fn)(input)
    }

    // JAVA: net.minecraft.util.BoundedFloatFunction#minValue
    pub fn min_value(&self) -> f32 {
        self.min_value
    }

    // JAVA: net.minecraft.util.BoundedFloatFunction#maxValue
    pub fn max_value(&self) -> f32 {
        self.max_value
    }

    // JAVA: net.minecraft.util.BoundedFloatFunction#comap
    pub fn comap<C2: 'static, F>(&self, f: F) -> BoundedFloatFunction<C2>
    where
        F: Fn(&C2) -> C + Send + Sync + 'static,
        C: Sized,
    {
        let inner = self.clone();
        BoundedFloatFunction {
            apply_fn: Arc::new(move |x: &C2| inner.apply(&f(x))),
            min_value: self.min_value,
            max_value: self.max_value,
        }
    }
}

impl BoundedFloatFunction<f32> {
    // JAVA: net.minecraft.util.BoundedFloatFunction#createUnlimited
    pub fn create_unlimited<F>(f: F) -> BoundedFloatFunction<f32>
    where
        F: Fn(f32) -> f32 + Send + Sync + 'static,
    {
        BoundedFloatFunction {
            apply_fn: Arc::new(move |x: &f32| f(*x)),
            min_value: f32::NEG_INFINITY,
            max_value: f32::INFINITY,
        }
    }

    // JAVA: net.minecraft.util.BoundedFloatFunction#IDENTITY
    pub fn identity() -> BoundedFloatFunction<f32> {
        Self::create_unlimited(|x| x)
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedFloatFunction;

    #[test]
    fn identity_applies_and_bounds() {
        let id = BoundedFloatFunction::identity();
        assert_eq!(id.apply(&1.5_f32).to_bits(), 1.5_f32.to_bits());
        assert_eq!(id.apply(&-42.0_f32).to_bits(), (-42.0_f32).to_bits());
        assert!(id.min_value().is_infinite() && id.min_value().is_sign_negative());
        assert!(id.max_value().is_infinite() && id.max_value().is_sign_positive());
    }

    #[test]
    fn create_unlimited_custom_function() {
        let f = BoundedFloatFunction::create_unlimited(|x| x * 2.0);
        assert_eq!(f.apply(&3.0_f32).to_bits(), 6.0_f32.to_bits());
        assert_eq!(f.apply(&-1.25_f32).to_bits(), (-2.5_f32).to_bits());
        assert!(f.min_value().is_infinite() && f.min_value().is_sign_negative());
        assert!(f.max_value().is_infinite() && f.max_value().is_sign_positive());
    }

    #[test]
    fn comap_maps_input_and_preserves_bounds() {
        let base = BoundedFloatFunction::create_unlimited(|x: f32| x + 1.0);
        let mapped: BoundedFloatFunction<i32> = base.comap(|x: &i32| *x as f32);
        assert_eq!(mapped.apply(&10_i32).to_bits(), 11.0_f32.to_bits());
        assert_eq!(mapped.apply(&-5_i32).to_bits(), (-4.0_f32).to_bits());
        assert_eq!(mapped.min_value().to_bits(), base.min_value().to_bits());
        assert_eq!(mapped.max_value().to_bits(), base.max_value().to_bits());
    }

    #[test]
    fn float_edge_cases_nan_and_infinity() {
        let id = BoundedFloatFunction::identity();
        assert!(id.apply(&f32::NAN).is_nan());
        assert_eq!(id.apply(&f32::INFINITY).to_bits(), f32::INFINITY.to_bits());
        assert_eq!(
            id.apply(&f32::NEG_INFINITY).to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
        assert_eq!(id.apply(&-0.0_f32).to_bits(), (-0.0_f32).to_bits());
    }
}
