use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Burst {
    pub weight: usize, // g
    pub length: usize, // ℓ
}

#[derive(Debug)]
pub struct RandomnessAnalyzer {
    bits: Vec<u8>, // 0/1
}

impl RandomnessAnalyzer {
    /// Erwartet einen String wie "100101"
    pub fn new(bitstring: &str) -> Result<Self, String> {
        let mut bits = Vec::new();

        for (i, c) in bitstring.chars().enumerate() {
            match c {
                '0' => bits.push(0),
                '1' => bits.push(1),
                _ => return Err(format!("Invalid char '{}' at {}", c, i)),
            }
        }

        if bits.is_empty() {
            return Err("Empty bitstream".into());
        }

        Ok(Self { bits })
    }

    // --------------------------------------------------
    // 1) Bit Occurrence Probability p_e
    // --------------------------------------------------
    pub fn bop(&self) -> f64 {
        let ones = self.bits.iter().filter(|&&b| b == 1).count();
        ones as f64 / self.bits.len() as f64
    }

    // --------------------------------------------------
    // 2) Gap extraction
    // --------------------------------------------------
    pub fn gaps(&self) -> Vec<usize> {
        let ones: Vec<usize> = self.bits.iter()
            .enumerate()
            .filter_map(|(i, &b)| if b == 1 { Some(i) } else { None })
            .collect();

        ones.windows(2)
            .map(|w| w[1] - w[0] - 1)
            .collect()
    }

    // --------------------------------------------------
    // 3) Gap density v(k)
    // --------------------------------------------------
    pub fn gap_density(&self) -> BTreeMap<usize, f64> {
        let gaps = self.gaps();
        let mut map = BTreeMap::new();

        if gaps.is_empty() {
            return map;
        }

        let total = gaps.len() as f64;
        for k in gaps {
            *map.entry(k).or_insert(0.0) += 1.0 / total;
        }
        map
    }

    // --------------------------------------------------
    // 4) Gap distribution u(k) = P(gap >= k)
    // --------------------------------------------------
    pub fn gap_distribution(&self) -> BTreeMap<usize, f64> {
        let gaps = self.gaps();
        let mut map = BTreeMap::new();

        if gaps.is_empty() {
            return map;
        }

        let max = *gaps.iter().max().unwrap();
        let total = gaps.len() as f64;

        for k in 0..=max {
            let count = gaps.iter().filter(|&&g| g >= k).count();
            map.insert(k, count as f64 / total);
        }
        map
    }

    // --------------------------------------------------
    // 5) IID-DU Check
    // --------------------------------------------------
    pub fn iid_du_check(&self) -> (f64, f64, f64) {
        let p = self.bop();
        let gaps = self.gaps();

        if gaps.is_empty() {
            return (p, 0.0, p.abs());
        }

        let v0 = gaps.iter().filter(|&&k| k == 0).count() as f64 / gaps.len() as f64;
        (p, v0, (p - v0).abs())
    }

    // --------------------------------------------------
    // 6) Burst analysis
    // --------------------------------------------------
    pub fn bursts(&self, a: usize) -> Vec<Burst> {
        let positions: Vec<usize> = self.bits.iter()
            .enumerate()
            .filter_map(|(i, &b)| if b == 1 { Some(i) } else { None })
            .collect();

        if positions.is_empty() {
            return vec![];
        }

        let mut bursts = Vec::new();
        let mut start = positions[0];
        let mut last = positions[0];
        let mut weight = 1;

        for &p in positions.iter().skip(1) {
            let gap = p - last - 1;
            if gap < a {
                weight += 1;
                last = p;
            } else {
                bursts.push(Burst {
                    weight,
                    length: last - start + 1,
                });
                start = p;
                last = p;
                weight = 1;
            }
        }

        bursts.push(Burst {
            weight,
            length: last - start + 1,
        });

        bursts
    }

    // --------------------------------------------------
    // 7) Burstiness level B
    // --------------------------------------------------
    pub fn burstiness_level(&self) -> Option<f64> {
        let gaps = self.gaps();
        if gaps.is_empty() {
            return None;
        }

        let n = gaps.len() as f64;
        let mean = gaps.iter().map(|&k| k as f64).sum::<f64>() / n;

        let var = gaps.iter()
            .map(|&k| {
                let d = k as f64 - mean;
                d * d
            })
            .sum::<f64>() / n;

        let sigma = var.sqrt();
        let denom = sigma + mean;

        if denom == 0.0 {
            Some(0.0)
        } else {
            Some((sigma - mean) / denom)
        }
    }
}
