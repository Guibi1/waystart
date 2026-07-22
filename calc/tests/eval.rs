use calc::{CalcError, eval};

fn evals(input: &str) -> String {
    eval(input).unwrap_or_else(|e| panic!("`{input}` failed: {e}")).to_string()
}

#[test]
fn raycast_example() {
    assert_eq!(evals("30m/2s"), "15 m/s");
}

#[test]
fn arithmetic() {
    assert_eq!(evals("1+2*3"), "7");
    assert_eq!(evals("(1+2)*3"), "9");
    assert_eq!(evals("10/4"), "2.5");
    assert_eq!(evals("2^10"), "1024");
    assert_eq!(evals("-3^2"), "-9");
    assert_eq!(evals("(-3)^2"), "9");
    assert_eq!(evals("1/3"), "0.3333333333");
    assert_eq!(evals("8 mod 3"), "2");
    assert_eq!(evals("2pi"), "6.2831853072");
    assert_eq!(evals("pi"), "3.1415926536");
}

#[test]
fn percentages() {
    assert_eq!(evals("50%"), "0.5");
    assert_eq!(evals("20% of 50"), "10");
    assert_eq!(evals("200*10%"), "20");
    assert_eq!(evals("80+25%"), "100");
}

#[test]
fn unit_arithmetic() {
    assert_eq!(evals("30m / 2s"), "15 m/s");
    assert_eq!(evals("1h / 2"), "0.5 h");
    assert_eq!(evals("1km + 500m"), "1.5 km");
    assert_eq!(evals("2m * 3m"), "6 m²");
    assert_eq!(evals("10m/s * 2s"), "20 m");
    assert_eq!(evals("100m / 5s^2"), "20 m/s²");
    assert_eq!(evals("(2m)^2"), "4 m²");
    assert_eq!(evals("2m^2"), "2 m²");
}

#[test]
fn conversions() {
    assert_eq!(evals("1km to m"), "1000 m");
    assert_eq!(evals("1km in m"), "1000 m");
    assert_eq!(evals("30m/2s to km/h"), "54 km/h");
    assert_eq!(evals("60mph to kmh"), "96.56064 km/h");
    assert_eq!(evals("5ft to cm"), "152.4 cm");
    assert_eq!(evals("5 in ft"), "0.4166666667 ft");
    // bare `value unit` converts to a default target (inch → cm)
    assert_eq!(evals("5in"), "12.7 cm");
    assert_eq!(evals("1gal to l"), "3.785411784 L");
    assert_eq!(evals("100kph to m/s"), "27.7777777778 m/s");
    assert_eq!(evals("1GiB to MB"), "1073.741824 MB");
    assert_eq!(evals("16 oz to lb"), "1 lb");
}

#[test]
fn temperature() {
    assert_eq!(evals("0c to f"), "32 °F");
    assert_eq!(evals("100c to f"), "212 °F");
    assert_eq!(evals("32f to c"), "0 °C");
    assert_eq!(evals("20°C"), "68 °F");
    assert_eq!(evals("0c to k"), "273.15 K");
}

#[test]
fn case_insensitive_units() {
    assert_eq!(evals("30M/2S"), "15 m/s");
    assert_eq!(evals("1KM to M"), "1000 m");
    assert_eq!(evals("5In"), "12.7 cm");
    assert_eq!(evals("1GB to mb"), "1000 MB");
}

#[test]
fn plural_units() {
    assert_eq!(evals("5 meters to feet"), "16.4041994751 ft");
    assert_eq!(evals("2 hours to minutes"), "120 min");
    assert_eq!(evals("3 inches to cm"), "7.62 cm");
}

#[test]
fn unicode_operators() {
    assert_eq!(evals("2×3"), "6");
    assert_eq!(evals("6÷2"), "3");
    assert_eq!(evals("5−2"), "3");
    assert_eq!(evals("2π").len() > 0, true);
    assert_eq!(evals("2m²"), "2 m²");
    assert_eq!(evals("6'"), "1.8288 m");
    assert_eq!(evals("6\""), "15.24 cm");
}

#[test]
fn errors() {
    assert_eq!(eval("").unwrap_err(), CalcError::Empty);
    assert_eq!(eval("1/0").unwrap_err(), CalcError::DivisionByZero);
    assert_eq!(eval("1m + 1s").unwrap_err(), CalcError::MismatchedDimensions {
        from: "m".into(),
        to: "s".into(),
    });
    assert!(matches!(eval("5 to 3"), Err(CalcError::InvalidConversionTarget)));
    assert!(matches!(eval("hello"), Err(CalcError::UnknownUnit(_))));
    assert!(matches!(eval("(1+2"), Err(CalcError::UnclosedParen)));
    assert!(matches!(eval("1m to s"), Err(CalcError::MismatchedDimensions { .. })));
    assert!(matches!(eval("1 +"), Err(CalcError::Empty)));
}

#[test]
fn display_derivations() {
    // quantities without an explicit display unit fall back to derived/base units
    assert_eq!(evals("1kg * 1m / 1s^2"), "1 N");
    assert_eq!(evals("1N * 1m"), "1 J");
    assert_eq!(evals("1J / 1s"), "1 W");
    assert_eq!(evals("1 / 1s"), "1 Hz");
}
