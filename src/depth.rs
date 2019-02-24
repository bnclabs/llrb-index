/// Depth calculates minimum, maximum, average and percentile of leaf-node
/// depth in the LLRB tree.
#[derive(Clone, Default)]
pub struct Depth {
    samples: usize,
    min: usize,
    max: usize,
    total: usize,
    depths: Vec<usize>,
}

impl Depth {
    pub(crate) fn new() -> Depth {
        let mut depth = Depth {
            samples: 0,
            min: 0,
            max: 0,
            total: 0,
            depths: Vec::with_capacity(256),
        };
        depth.depths.resize(100, 0);
        depth
    }

    pub(crate) fn sample(&mut self, depth: usize) {
        self.samples += 1;
        self.total += depth;
        if self.min == 0 || self.min > depth {
            self.min = depth
        }
        if self.max == 0 || self.max < depth {
            self.max = depth
        }
        self.depths[depth as usize] += 1;
    }

    /// Return the average depth of leaf-nodes in LLRB tree.
    pub fn mean(&self) -> usize {
        self.total / self.samples
    }

    /// Return number of leaf-nodes sample for depth in LLRB tree.
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Return minimum depth of leaf-node in LLRB tree.
    pub fn min(&self) -> usize {
        self.min
    }

    /// Return maximum depth of leaf-node in LLRB tree.
    pub fn max(&self) -> usize {
        self.max
    }

    /// Return depth as tuple of percentiles, each tuple provides
    /// (percentile, depth). Percentiles are available for
    /// 80th, 90nth, 95th, 96th, 97th, 98th, 99th.
    pub fn percentiles(&self) -> Vec<(u8, usize)> {
        let mut percentiles = [
            (0.80, 0_usize /*depth*/),
            (0.90, 0_usize /*depth*/),
            (0.95, 0_usize /*depth*/),
            (0.96, 0_usize /*depth*/),
            (0.97, 0_usize /*depth*/),
            (0.98, 0_usize /*depth*/),
            (0.99, 0_usize /*depth*/),
        ];
        let mut iter = percentiles.iter_mut();
        let mut item: &mut (f64, usize) = iter.next().unwrap();
        let mut acc = 0_f64;
        for (depth, count) in self.depths.iter().enumerate() {
            acc += *count as f64;
            if acc > ((self.samples as f64) * item.0) {
                item.1 = depth;
                match iter.next() {
                    Some(x) => item = x,
                    None => break,
                }
            }
        }
        percentiles
            .iter()
            .map(|item| (((item.0 * 100.0) as u8), item.1))
            .collect()
    }
}
