pub mod tape;
pub mod chart;
pub mod history;

#[derive(Clone, Debug)]
pub struct Bar {
    pub ts: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
