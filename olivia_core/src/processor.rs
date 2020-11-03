#[derive(Debug, PartialEq)]
pub struct Processor {}

impl Processor {
    pub fn new() -> Processor {
        Processor {}
    }
}

impl Processor {
    pub fn process(&mut self, out_left: &mut [f32], out_right: &mut [f32]) {
        for channel in [out_left, out_right].iter_mut() {
            for o in channel.iter_mut() {
                *o = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outputs_are_cleared() {
        let mut left = [1.0, 2.0];
        let mut right = [3.0, 4.0];

        let want_left = [0.0, 0.0];
        let want_right = [0.0, 0.0];

        let mut p = Processor::new();
        assert_ne!([left, right], [want_left, want_right]);
        p.process(&mut left, &mut right);
        assert_eq!([left, right], [want_left, want_right]);
    }
}
