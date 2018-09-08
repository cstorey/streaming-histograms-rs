extern crate streaming_quantiles;

use streaming_quantiles::*;
#[test]
fn trival_empty() {
    let h = Histogram::new(5);
    assert_sum_within(&h, 0.0, 0.0, 0.001);
}

#[test]
fn trival_single_observation_point_off_right() {
    let mut h = Histogram::new(5);
    h.add(0.0);
    assert_sum_within(&h, 1.0, 1.0, 0.5);
}

#[test]
fn trival_single_observation_point_at_center() {
    let mut h = Histogram::new(5);
    h.add(0.0);
    assert_sum_within(&h, 1.0, 0.0, 0.5);
}

#[test]
fn trival_single_observation_point_off_left() {
    let mut h = Histogram::new(5);
    h.add(1.0);
    assert_sum_within(&h, 0.0, 0.0, 0.5);
}

#[test]
fn trival_three_observation_point_on_mid_point() {
    let mut h = Histogram::new(5);
    h.add(1.0);
    h.add(2.0);
    h.add(3.0);
    assert_sum_within(&h, 2.0, 2.0, 0.5);
}

// This ends up evaluating to two, since our final "end" sample is
// effectively at infinity, so we have effectively an epsilon area trapezum
// Left of our sample.
#[test]
fn trival_three_observation_point_at_rightmost_point() {
    let mut h = Histogram::new(5);
    h.add(1.0);
    h.add(2.0);
    h.add(2.0);
    assert_sum_within(&h, 3.0, 2.0, 1.0);
}

#[test]
fn trival_three_wide_gulf() {
    let mut h = Histogram::new(5);
    h.add(1.0);
    h.add(2.0);
    h.add(12.0);
    assert_sum_within(&h, 2.5, 7.0, 0.5);
}

#[test]
fn compression() {
    let mut h = Histogram::new(3);
    h.add(1.0);
    h.add(1.5);
    h.add(2.0);
    h.add(5.0);
    assert_sum_within(&h, 3.0, 2.0, 0.5);
}

#[test]
fn paper_appendix_example() {
    let mut h = Histogram::new(5);
    let mut k = Histogram::new(5);
    for (i, obs) in [23, 19, 10, 16, 36, 2, 9].iter().cloned().enumerate() {
        h.add(obs as f64);
        println!("{}: Added: {}; {:?}", i, obs, h);
    }
    k.add(32.);
    k.add(30.);
    k.add(45.);
    h.merge_from(&k);
    println!("Merged in: {:?}", k);
    println!("Result => {:?}", h);

    assert_sum_within(&h, 3.28, 15.0, 0.01);
}

fn assert_sum_within(h: &Histogram, expected: f64, leq: f64, margin: f64) {
    let value = h.sum(leq);
    let lb = expected - margin;
    let ub = expected + margin;
    assert!(
        lb <= value && value < ub,
        "Wanted is {:.3} <= h.sum({:.3})={:.3} < {:.3}; from histogram: {:?}",
        lb,
        leq,
        value,
        ub,
        h
    );
}
