use midpoint_foundry::algorithms::{
    baseline_circle::BaselineCircle, circle_base::CircleBase, gap_fill_circle::GapFillCircle,
    midpoint_circle::MidpointCircle, parallel_midpoint_circle::ParallelMidpointCircle,
    scanline_circle::ScanlineCircle, sitaraman_fill_circle::SitaramanFillCircle,
};

fn main() {
    // Test that we can create instances of all the circle algorithms
    let _baseline = BaselineCircle::default();
    let _midpoint = MidpointCircle::default();
    let _parallel_midpoint = ParallelMidpointCircle::default();
    let _scanline = ScanlineCircle::default();
    let _gap_fill = GapFillCircle::default();
    let _sitaraman = SitaramanFillCircle::default();

    // Test CircleBase directly
    let _base = CircleBase::new(0, 0, 10);

    println!("All circle algorithms instantiated successfully!");
    println!("Refactoring completed.");
}
