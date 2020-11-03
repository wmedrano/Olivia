#[derive(Debug, PartialEq)]
pub struct Ports<'a> {
    pub left: &'a mut [f32],
    pub right: &'a mut [f32],
}

pub struct Processor {}

impl Processor {
    pub fn new() -> Processor {
        Processor {}
    }
}

impl Processor {
    pub fn process(&mut self, ports: &mut Ports<'_>) {
        for channel in [&mut ports.left, &mut ports.right].iter_mut() {
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
        let mut ports = Ports{
            left: &mut left,
            right: &mut right,
        };

        let mut want_left = [0.0, 0.0];
        let mut want_right = [0.0, 0.0];
        let want_ports = Ports{
            left: &mut want_left,
            right: &mut want_right,
        };

        let mut p = Processor::new();
        p.process(&mut ports);
        assert_eq!(ports, want_ports);
    }
}
