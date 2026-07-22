//! Port of fuzzy-units' `tests/convert.test.ts` (bun) to the calc crate.
//! `->` / `=>` / `as` / `2` separators are intentionally not supported;
//! those cases use `to` instead.

use calc::convert;

/// matches bun's `toBeCloseTo(x, 3)`
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 5e-4 * expected.abs().max(1.0),
        "expected {actual} ≈ {expected}"
    );
}

// temperature

#[test]
fn celsius_to_fahrenheit_by_default() {
    let r = convert("5c").unwrap();
    close(r.value, 5.0);
    assert_eq!(r.from.unwrap().id, "celsius");
    assert_eq!(r.to.unwrap().id, "fahrenheit");
    close(r.result, 41.0);
}

#[test]
fn fahrenheit_to_celsius_by_default() {
    let r = convert("32f").unwrap();
    assert_eq!(r.from.unwrap().id, "fahrenheit");
    assert_eq!(r.to.unwrap().id, "celsius");
    close(r.result, 0.0);
}

#[test]
fn celsius_to_fahrenheit_explicit() {
    close(convert("100 celsius to fahrenheit").unwrap().result, 212.0);
    close(convert("212 fahrenheit to celsius").unwrap().result, 100.0);
    close(convert("0 kelvin in celsius").unwrap().result, -273.15);
    close(convert("-40 c to f").unwrap().result, -40.0);
}

#[test]
fn temperature_typos() {
    let r = convert("100 celcius").unwrap();
    assert_eq!(r.from.unwrap().id, "celsius");
    close(r.result, 212.0);
    let r = convert("32 farenheit").unwrap();
    assert_eq!(r.from.unwrap().id, "fahrenheit");
}

// length

#[test]
fn length() {
    let r = convert("1 km to mi").unwrap();
    assert_eq!(r.from.unwrap().id, "kilometer");
    assert_eq!(r.to.unwrap().id, "mile");
    close(r.result, 0.621371);
    close(convert("1 mile to km").unwrap().result, 1.60934);
    close(convert("5280 ft to mi").unwrap().result, 1.0);
    close(convert("100 cm to inch").unwrap().result, 39.3701);
    assert_eq!(convert("1 meter").unwrap().to.unwrap().id, "foot");
    assert_eq!(convert("10 km").unwrap().to.unwrap().id, "mile");
}

// weight

#[test]
fn weight() {
    let r = convert("1 kg to lbs").unwrap();
    assert_eq!(r.from.unwrap().id, "kilogram");
    assert_eq!(r.to.unwrap().id, "pound");
    close(r.result, 2.20462);
    close(convert("1 lb to kg").unwrap().result, 0.453592);
    close(convert("16 oz to lb").unwrap().result, 1.0);
    close(convert("1000 g to kg").unwrap().result, 1.0);
}

// volume

#[test]
fn volume() {
    close(convert("1 gallon to liter").unwrap().result, 3.78541);
    close(convert("1 liter to gallon").unwrap().result, 0.264172);
    close(convert("1 cup to ml").unwrap().result, 236.588);
}

// time

#[test]
fn time() {
    close(convert("1 hour to minutes").unwrap().result, 60.0);
    close(convert("60 min to hr").unwrap().result, 1.0);
    close(convert("1 day to hours").unwrap().result, 24.0);
    close(convert("1 week to days").unwrap().result, 7.0);
}

// digital storage

#[test]
fn storage_si_decimal() {
    close(convert("1 gb to mb").unwrap().result, 1000.0);
    close(convert("1000 kb to mb").unwrap().result, 1.0);
    close(convert("1 tb to gb").unwrap().result, 1000.0);
}

#[test]
fn storage_iec_binary() {
    let r = convert("1 GiB to MiB").unwrap();
    assert_eq!(r.from.unwrap().id, "gibibyte");
    assert_eq!(r.to.unwrap().id, "mebibyte");
    close(r.result, 1024.0);
    let r = convert("1 TiB to GiB").unwrap();
    assert_eq!(r.from.unwrap().id, "tebibyte");
    assert_eq!(r.to.unwrap().id, "gibibyte");
    close(r.result, 1024.0);
    close(convert("1024 KiB to MiB").unwrap().result, 1.0);
}

#[test]
fn storage_si_iec_cross_defaults() {
    let r = convert("1 gb").unwrap();
    assert_eq!(r.from.unwrap().id, "gigabyte");
    assert_eq!(r.to.unwrap().id, "gibibyte");
    close(r.result, 0.931323);
    let r = convert("1 GiB").unwrap();
    assert_eq!(r.from.unwrap().id, "gibibyte");
    assert_eq!(r.to.unwrap().id, "gigabyte");
    close(r.result, 1.073742);
}

#[test]
fn storage_bits() {
    close(convert("8 bit to byte").unwrap().result, 1.0);
    close(convert("1 byte to bit").unwrap().result, 8.0);
    close(convert("1 gigabit to megabit").unwrap().result, 1000.0);
    let r = convert("1 Gb to megabyte").unwrap();
    assert_eq!(r.from.unwrap().id, "gigabit");
    assert_eq!(r.to.unwrap().id, "megabyte");
    close(r.result, 125.0);
}

#[test]
fn storage_case_sensitive_abbreviations() {
    let cases = [
        ("1 GB", "gigabyte"),
        ("1 Gb", "gigabit"),
        ("1 GiB", "gibibyte"),
        ("1 MB", "megabyte"),
        ("1 Mb", "megabit"),
        ("1 MiB", "mebibyte"),
        ("1 KB", "kilobyte"),
        ("1 Kb", "kilobit"),
        ("1 KiB", "kibibyte"),
        ("1 TB", "terabyte"),
        ("1 Tb", "terabit"),
        ("1 TiB", "tebibyte"),
    ];
    for (input, expected_id) in cases {
        let r = convert(input).unwrap();
        assert_eq!(r.from.unwrap().id, expected_id, "input: {input}");
    }
}

// separators (`to` / `in`; `->`, `=>`, `as`, `2` intentionally unsupported)

#[test]
fn separators() {
    close(convert("5 celsius to fahrenheit").unwrap().result, 41.0);
    close(convert("5 celsius in fahrenheit").unwrap().result, 41.0);
}

// fuzzy / typo tolerance

#[test]
fn fuzzy_typos() {
    assert_eq!(convert("100 celcius to f").unwrap().from.unwrap().id, "celsius");
    assert_eq!(convert("100 farenheight to c").unwrap().from.unwrap().id, "fahrenheit");
    assert_eq!(convert("5 kilometre to mile").unwrap().from.unwrap().id, "kilometer");
    // not an alias — resolved purely by nucleo fuzzy matching
    assert_eq!(convert("5 kilometr to mile").unwrap().from.unwrap().id, "kilometer");
    assert_eq!(convert("100 fahrenhei to c").unwrap().from.unwrap().id, "fahrenheit");
}

// edge cases

#[test]
fn edge_cases() {
    let r = convert("3.5 km to mi").unwrap();
    assert_eq!(r.from.unwrap().id, "kilometer");
    close(r.result, 2.1748);

    let r = convert("5c").unwrap();
    assert!(r.display.contains('5'), "display: {}", r.display);
    assert!(r.display.contains("Celsius"), "display: {}", r.display);
    assert!(r.display.contains("41"), "display: {}", r.display);
    assert!(r.display.contains("Fahrenheit"), "display: {}", r.display);

    assert!(convert("").is_err());
    assert!(convert("5 xyzzy").is_err());
    assert!(convert("5 kg to km").is_err());
}
