pub trait Feature {
    fn enabled() -> bool;
    fn enable(&mut self);
    fn disable(&mut self);
}

pub trait Source: Feature {}
pub trait Transformer: Feature {}
pub trait Sink: Feature {}
pub trait UIElement: Feature {}
pub trait UIBackend: Feature {}
pub trait DBBackend: Feature {}
