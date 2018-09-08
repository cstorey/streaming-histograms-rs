use std::cmp;
use std::collections::BTreeMap;
use std::ops;

#[derive(Debug, Copy, Clone)]
struct NonNanF64(f64);

#[derive(Debug, Clone)]
pub struct Histogram {
    size: usize,
    buckets: BTreeMap<NonNanF64, f64>,
}

impl cmp::PartialEq for NonNanF64 {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}
impl cmp::Eq for NonNanF64 {}
impl cmp::PartialOrd for NonNanF64 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl cmp::Ord for NonNanF64 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.partial_cmp(&other.0).expect("Not nan")
    }
}
impl ops::Sub for NonNanF64 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let res = self.0 - other.0;
        assert!(!res.is_nan());
        NonNanF64(res)
    }
}

impl NonNanF64 {
    fn of(val: f64) -> Self {
        assert!(!val.is_nan());
        NonNanF64(val)
    }

    fn as_f64(&self) -> f64 {
        self.0
    }
}

impl Histogram {
    pub fn new(size: usize) -> Self {
        let buckets = BTreeMap::new();
        Histogram { size, buckets }
    }

    pub fn add(&mut self, value: f64) {
        let k = NonNanF64::of(value);
        *self.buckets.entry(k).or_default() += 1.0;

        self.trim()
    }

    fn trim(&mut self) {
        while self.buckets.len() > self.size {
            if let Some((qa, qb)) = self
                .buckets
                .keys()
                .cloned()
                .zip(self.buckets.keys().cloned().skip(1))
                .min_by_key(|(qa, qb)| *qb - *qa)
            {
                let ka = self.buckets.remove(&qa).expect("bucket observation");
                let kb = self.buckets.remove(&qb).expect("bucket observation");
                let midcnt = ka + kb;
                let midpt = (qa.0 * ka + qb.0 * kb) / midcnt;
                self.buckets.insert(NonNanF64::of(midpt), midcnt);
            } else {
                panic!("Could not find a minimal bucket")
            }
        }
    }

    pub fn merge_from(&mut self, other: &Histogram) {
        for (&p, &m) in other.buckets.iter() {
            *self.buckets.entry(p).or_default() += m;
        }
        self.trim()
    }

    pub fn sum(&self, leq: f64) -> f64 {
        let mut sum = 0.0;
        println!("leq:{}", leq);

        let left_inf = (&NonNanF64::of(::std::f64::MIN), &0.0);
        let right_inf = (&NonNanF64::of(::std::f64::MAX), &0.0);
        for ((&px, &mx), (&py, &my)) in ([left_inf].iter().cloned().chain(self.buckets.iter()))
            .zip(self.buckets.iter().chain([right_inf].iter().cloned()))
        {
            println!(
                "px:{:e},mx:{}; py:{:e},my:{}",
                px.as_f64(),
                mx,
                py.as_f64(),
                my
            );
            println!(
                "px {:?} leq {:?} py",
                px.cmp(&NonNanF64::of(leq)),
                NonNanF64::of(leq).cmp(&py)
            );
            if leq >= py.as_f64() {
                sum += mx;
                println!("add centroid: {}@{:e} -> {}", mx, px.0, sum);
            } else if px.as_f64() <= leq && leq < py.as_f64() {
                // Trapezoid formed by (px,0), (px, mx), (py, my), (py, 0)
                //
                let width = (py - px).as_f64();
                assert!(!width.is_nan());
                assert!(width != 0.0);
                assert!(width != -0.0);
                let height_diff = my - mx;
                assert!(!height_diff.is_nan());
                let gradient = height_diff / width;
                assert!(!gradient.is_nan());
                let offset_from_left = leq - px.as_f64();
                assert!(!offset_from_left.is_nan());
                let proportion_of_width = offset_from_left / width;
                assert!(
                    !proportion_of_width.is_nan(),
                    "offset_from_left={} / width={} -> {}",
                    offset_from_left,
                    width,
                    proportion_of_width
                );
                let height_at_leq = mx + gradient * offset_from_left;
                assert!(!height_at_leq.is_nan());

                // Area of trapezoid (px,0), (px, mx), (pb, mb), (pb, 0)
                // Find height at midpoint of mx and b
                let nobservations = ((mx + height_at_leq) / 2.0) * proportion_of_width;
                assert!(!nobservations.is_nan());
                // We make the assumption that each center px is surrounded by
                // mx/2 points on each side. Hence we add an estimate of same.
                let obs_left_of_mx = mx / 2.0;
                assert!(!obs_left_of_mx.is_nan());
                println!("Total observation is running:{:e} + left of mx:{:e} + trapezoid observations:{:e} => {:e}",
                    sum, obs_left_of_mx, nobservations, sum + obs_left_of_mx + nobservations);
                sum += obs_left_of_mx + nobservations;

                println!(
                    "mb:{:e}; nobservations:{:e}; fudge:{:e} => {:e}",
                    height_at_leq, nobservations, obs_left_of_mx, sum
                );
                break;
            }
        }

        sum
    }
}
