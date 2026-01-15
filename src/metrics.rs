use lazy_static::lazy_static;
use prometheus;

// Create and register metrics according to documentation.
// TODO: normal metric definitions.
lazy_static! {
    pub static ref HIGH_FIVE_COUNTER: prometheus::IntCounter =
        prometheus::register_int_counter!("highfives", "Number of high fives received").unwrap();
}
