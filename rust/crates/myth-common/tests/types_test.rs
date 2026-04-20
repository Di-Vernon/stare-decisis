use myth_common::{Category, Enforcement, Level, Recurrence};

#[test]
fn recurrence_boundaries() {
    assert_eq!(Recurrence::from_count(0.0), Recurrence::I);
    assert_eq!(Recurrence::from_count(0.99), Recurrence::I);
    assert_eq!(Recurrence::from_count(1.0), Recurrence::II);
    assert_eq!(Recurrence::from_count(1.99), Recurrence::II);
    assert_eq!(Recurrence::from_count(2.0), Recurrence::III);
    assert_eq!(Recurrence::from_count(3.99), Recurrence::III);
    assert_eq!(Recurrence::from_count(4.0), Recurrence::IV);
    assert_eq!(Recurrence::from_count(6.99), Recurrence::IV);
    assert_eq!(Recurrence::from_count(7.0), Recurrence::V);
    assert_eq!(Recurrence::from_count(11.99), Recurrence::V);
    assert_eq!(Recurrence::from_count(12.0), Recurrence::VI);
    assert_eq!(Recurrence::from_count(999.0), Recurrence::VI);
}

#[test]
fn level_labels() {
    assert_eq!(Level::Info.label(), "INFO");
    assert_eq!(Level::Low.label(), "LOW");
    assert_eq!(Level::Medium.label(), "MEDIUM");
    assert_eq!(Level::High.label(), "HIGH");
    assert_eq!(Level::Critical.label(), "CRITICAL");
}

#[test]
fn level_ordering() {
    assert!(Level::Info < Level::Critical);
    assert!(Level::High > Level::Low);
}

#[test]
fn enforcement_blocking_set() {
    assert!(!Enforcement::Dismiss.is_blocking());
    assert!(!Enforcement::Note.is_blocking());
    assert!(!Enforcement::Advisory.is_blocking());
    assert!(!Enforcement::Caution.is_blocking());
    assert!(Enforcement::Warn.is_blocking());
    assert!(Enforcement::Strike.is_blocking());
    assert!(Enforcement::Seal.is_blocking());
}

#[test]
fn enforcement_serde_roundtrip() {
    let e = Enforcement::Strike;
    let j = serde_json::to_string(&e).unwrap();
    assert_eq!(j, r#""strike""#);
    let back: Enforcement = serde_json::from_str(&j).unwrap();
    assert_eq!(back, Enforcement::Strike);
}

#[test]
fn category_serde_snake_case() {
    let c = Category::DataSafety;
    let j = serde_json::to_string(&c).unwrap();
    assert_eq!(j, r#""data_safety""#);
    let back: Category = serde_json::from_str(&j).unwrap();
    assert_eq!(back, Category::DataSafety);
}
