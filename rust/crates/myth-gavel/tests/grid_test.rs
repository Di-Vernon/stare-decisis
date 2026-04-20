use myth_common::{Enforcement, Level, Recurrence};
use myth_gavel::Grid;

#[test]
fn default_matrix_corners() {
    let grid = Grid::new();
    assert_eq!(grid.lookup(Level::Info, Recurrence::I), Enforcement::Dismiss);
    assert_eq!(grid.lookup(Level::Info, Recurrence::VI), Enforcement::Advisory);
    assert_eq!(grid.lookup(Level::Critical, Recurrence::I), Enforcement::Strike);
    assert_eq!(grid.lookup(Level::Critical, Recurrence::VI), Enforcement::Strike);
}

#[test]
fn default_matrix_level3_spine() {
    // L3 row from the design: [Advisory, Caution, Caution, Warn, Warn, Warn]
    let grid = Grid::new();
    assert_eq!(grid.lookup(Level::Medium, Recurrence::I), Enforcement::Advisory);
    assert_eq!(grid.lookup(Level::Medium, Recurrence::II), Enforcement::Caution);
    assert_eq!(grid.lookup(Level::Medium, Recurrence::III), Enforcement::Caution);
    assert_eq!(grid.lookup(Level::Medium, Recurrence::IV), Enforcement::Warn);
    assert_eq!(grid.lookup(Level::Medium, Recurrence::V), Enforcement::Warn);
    assert_eq!(grid.lookup(Level::Medium, Recurrence::VI), Enforcement::Warn);
}

#[test]
fn override_takes_precedence_over_default() {
    let mut grid = Grid::new();
    // default L3×III = Caution; override to Warn.
    assert_eq!(grid.lookup(Level::Medium, Recurrence::III), Enforcement::Caution);
    grid.set_override(Level::Medium, Recurrence::III, Enforcement::Warn);
    assert_eq!(grid.lookup(Level::Medium, Recurrence::III), Enforcement::Warn);

    // Non-overridden cells remain from the default matrix.
    assert_eq!(grid.lookup(Level::Medium, Recurrence::II), Enforcement::Caution);
}

#[test]
fn level5_row_is_fully_strike() {
    let grid = Grid::new();
    for r in [
        Recurrence::I,
        Recurrence::II,
        Recurrence::III,
        Recurrence::IV,
        Recurrence::V,
        Recurrence::VI,
    ] {
        assert_eq!(
            grid.lookup(Level::Critical, r),
            Enforcement::Strike,
            "L5 × {:?} must be Strike",
            r
        );
    }
}
