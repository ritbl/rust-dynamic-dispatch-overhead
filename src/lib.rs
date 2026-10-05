//! Compare a direct function call with an indirect vtable call to the same body.
use std::hint::black_box;

pub trait Operation {
    fn apply(&self, value: u64) -> u64;
}

pub struct Transform {
    pub mask: u64,
}

impl Operation for Transform {
    // Both variants call this exact body. Disable inlining to isolate dispatch
    // more closely, rather than measuring inlining benefits in the static case.
    #[inline(never)]
    fn apply(&self, value: u64) -> u64 {
        value.rotate_left(7) ^ self.mask
    }
}

// Keep the loop boundaries out of the timing harness's optimization context.
// No barrier is needed per iteration: each call depends on the previous result.
#[inline(never)]
pub fn static_dispatch(operation: &Transform, iterations: u64, seed: u64) -> u64 {
    let operation = black_box(operation);
    let mut value = seed;
    for _ in 0..iterations {
        value = operation.apply(value);
    }
    value
}

#[inline(never)]
pub fn vtable_dispatch(operation: &dyn Operation, iterations: u64, seed: u64) -> u64 {
    // Obscure the concrete type once so the compiler cannot readily replace the
    // vtable lookup with a direct call. Verify emitted assembly for each build.
    let operation = black_box(operation);
    let mut value = seed;
    for _ in 0..iterations {
        value = operation.apply(value);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_dispatch_paths_produce_the_expected_result() {
        for mask in [0, 1, u64::MAX, 0x1234_5678_9abc_def0] {
            let operation = Transform { mask };
            for iterations in [0, 1, 2, 63, 1024] {
                let seed = 0xfeed_face;
                let mut expected: u64 = seed;
                for _ in 0..iterations {
                    expected = expected.rotate_left(7) ^ mask;
                }
                assert_eq!(static_dispatch(&operation, iterations, seed), expected);
                assert_eq!(vtable_dispatch(&operation, iterations, seed), expected);
            }
        }
    }
}
