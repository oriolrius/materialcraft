// Lowbias32 integer hash by Chris Wellons, public domain
#[inline]
pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// The 24-bit sequence used by scalar node-parameter functions.
pub(crate) fn node_function(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(16_294_893).wrapping_add(10_493_233) & 0x00ff_ffff;
    *state as f32 / 16_777_216.0
}

/// The measured scalar sequence for functions evaluated inside a pattern graph.
pub(crate) fn pattern_function(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(3_626_811).wrapping_add(51_137) & 0x00ff_ffff;
    (*state >> 1) as f32 / 8_388_608.0
}

/// Quadrant child streams are independent of parameter-function draws.
/// Original constant-output probes establish a sixteen-depth repeating sequence.
pub(crate) fn quadrant_child(state: &mut u32, depth: u32) -> u32 {
    let (multiplier, increment) = match depth & 15 {
        1 => (1_691_414_u32, 52_627_u32),
        2 => (5_547_014, 52_673),
        3 => (1_289_321, 52_711),
        4 => (10_611_682, 52_747),
        5 => (2_603_697, 52_807),
        6 => (13_013_189, 52_859),
        7 => (14_482_087, 52_249),
        8 => (4_839_483, 52_289),
        9 => (3_906_728, 52_321),
        10 => (4_193_611, 52_379),
        11 => (4_656_553, 52_453),
        12 => (3_558_434, 52_511),
        13 => (7_796_889, 52_543),
        14 => (3_426_494, 52_189),
        15 => (11_525_242, 51_929),
        _ => (14_449_176, 52_571),
    };
    *state = state.wrapping_mul(multiplier).wrapping_add(increment) & 0x00ff_ffff;
    *state
}
