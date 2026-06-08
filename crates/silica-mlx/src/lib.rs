//! MLX feature boundary for SilicaRAW.
//!
//! Task 0101 placeholder only.

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-mlx";

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-mlx");
    }
}
