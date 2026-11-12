/// FX-Map regions use left, right, top, bottom coordinates in pattern space.
pub(super) struct Region {
    start: [f32; 2],
    span: [f32; 2],
}

impl Region {
    pub fn new(value: [f32; 4]) -> Result<Self, String> {
        let start = [value[0], value[2]];
        let span = [value[1] - value[0], value[3] - value[2]];
        if value.iter().any(|value| !value.is_finite()) || span.iter().any(|value| !value.is_finite() || value.abs() < f32::EPSILON) {
            return Err("pattern regions must be finite with nonzero width and height".into());
        }
        Ok(Self { start, span })
    }

    pub fn point(&self, pixel: [u32; 2], size: [u32; 2]) -> [f32; 2] {
        std::array::from_fn(|axis| self.start[axis] + (pixel[axis] as f32 + 0.5) / size[axis] as f32 * self.span[axis])
    }

    pub fn bounds(&self, center: [f32; 2], radius: [f32; 2], size: [u32; 2]) -> [[u32; 2]; 2] {
        std::array::from_fn(|axis| {
            let a = (center[axis] - radius[axis] - self.start[axis]) / self.span[axis];
            let b = (center[axis] + radius[axis] - self.start[axis]) / self.span[axis];
            [
                (a.min(b) * size[axis] as f32).floor().clamp(0.0, size[axis] as f32) as u32,
                (a.max(b) * size[axis] as f32).ceil().clamp(0.0, size[axis] as f32) as u32,
            ]
        })
    }

    pub fn period(&self) -> [f32; 2] {
        self.span.map(f32::abs)
    }

    pub fn repeats(&self, center: [f32; 2], radius: [f32; 2], period: [f32; 2]) -> Result<[[i32; 2]; 2], String> {
        let mut result = [[0; 2]; 2];
        for axis in 0..2 {
            let end = self.start[axis] + self.span[axis];
            let first = ((self.start[axis].min(end) - center[axis] - radius[axis]) / period[axis]).ceil();
            let last = ((self.start[axis].max(end) - center[axis] + radius[axis]) / period[axis]).floor();
            // A tiny period or huge pattern cannot create unbounded work, even outside the image.
            if !first.is_finite() || !last.is_finite() || first < i32::MIN as f32 || last >= i32::MAX as f32 {
                return Err("pattern repetition exceeds its coordinate budget".into());
            }
            result[axis] = [first as i32, last as i32];
        }
        Ok(result)
    }
}
