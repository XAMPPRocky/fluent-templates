/// A concrete type that provides a loader agnostic implementation
#[allow(dead_code)]
pub struct FluentHelper<L> {
    loader: L,
}

impl<L> FluentHelper<L> {
    /// Create a new `FluentHelper`.
    pub fn new(loader: L) -> Self {
        Self { loader }
    }
}

#[cfg(feature = "handlebars")]
mod handlebars;

#[cfg(feature = "tera")]
mod tera;
