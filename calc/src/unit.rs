use nucleo_matcher::Matcher;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};

use crate::quantity::{CURRENT, DATA, DispUnit, Dims, LEN, MASS, TEMP, TIME, dims};

pub struct UnitDef {
    /// canonical identifier, e.g. "gibibyte"
    pub id: &'static str,
    /// human-readable name, e.g. "Gibibyte"
    pub name: &'static str,
    /// accepted spellings; mixed-case entries are case-sensitive,
    /// all-lowercase entries also match case-insensitively
    pub names: &'static [&'static str],
    pub symbol: &'static str,
    /// value_si = display_value * factor + offset
    pub factor: f64,
    pub offset: f64,
    pub dims: Dims,
}

const fn u(
    id: &'static str,
    name: &'static str,
    names: &'static [&'static str],
    symbol: &'static str,
    factor: f64,
    dims: Dims,
) -> UnitDef {
    UnitDef {
        id,
        name,
        names,
        symbol,
        factor,
        offset: 0.0,
        dims,
    }
}

const fn affine(
    id: &'static str,
    name: &'static str,
    names: &'static [&'static str],
    symbol: &'static str,
    factor: f64,
    offset: f64,
    dims: Dims,
) -> UnitDef {
    UnitDef {
        id,
        name,
        names,
        symbol,
        factor,
        offset,
        dims,
    }
}

static UNITS: &[UnitDef] = &[
    // length
    u("meter", "Meter", &["m", "meter", "metre"], "m", 1.0, dims(&[(LEN, 1)])),
    u("kilometer", "Kilometer", &["km", "kilometer", "kilometre"], "km", 1e3, dims(&[(LEN, 1)])),
    u("centimeter", "Centimeter", &["cm", "centimeter", "centimetre"], "cm", 1e-2, dims(&[(LEN, 1)])),
    u("millimeter", "Millimeter", &["mm", "millimeter", "millimetre"], "mm", 1e-3, dims(&[(LEN, 1)])),
    u("micrometer", "Micrometer", &["um", "µm", "μm", "micrometer", "micrometre", "micron"], "µm", 1e-6, dims(&[(LEN, 1)])),
    u("nanometer", "Nanometer", &["nm", "nanometer", "nanometre"], "nm", 1e-9, dims(&[(LEN, 1)])),
    u("mile", "Mile", &["mi", "mile"], "mi", 1609.344, dims(&[(LEN, 1)])),
    u("nauticalmile", "Nautical Mile", &["nmi", "nauticalmile"], "nmi", 1852.0, dims(&[(LEN, 1)])),
    u("yard", "Yard", &["yd", "yard"], "yd", 0.9144, dims(&[(LEN, 1)])),
    u("foot", "Foot", &["ft", "foot", "feet"], "ft", 0.3048, dims(&[(LEN, 1)])),
    u("inch", "Inch", &["in", "inch"], "in", 0.0254, dims(&[(LEN, 1)])),
    // mass
    u("kilogram", "Kilogram", &["kg", "kilogram", "kilo"], "kg", 1.0, dims(&[(MASS, 1)])),
    u("gram", "Gram", &["g", "gram"], "g", 1e-3, dims(&[(MASS, 1)])),
    u("milligram", "Milligram", &["mg", "milligram"], "mg", 1e-6, dims(&[(MASS, 1)])),
    u("tonne", "Tonne", &["t", "tonne"], "t", 1e3, dims(&[(MASS, 1)])),
    u("pound", "Pound", &["lb", "lbs", "pound"], "lb", 0.45359237, dims(&[(MASS, 1)])),
    u("ounce", "Ounce", &["oz", "ounce"], "oz", 0.028349523125, dims(&[(MASS, 1)])),
    u("stone", "Stone", &["st", "stone"], "st", 6.35029318, dims(&[(MASS, 1)])),
    // time
    u("second", "Second", &["s", "sec", "second"], "s", 1.0, dims(&[(TIME, 1)])),
    u("millisecond", "Millisecond", &["ms", "millisecond"], "ms", 1e-3, dims(&[(TIME, 1)])),
    u("microsecond", "Microsecond", &["us", "µs", "μs", "microsecond"], "µs", 1e-6, dims(&[(TIME, 1)])),
    u("nanosecond", "Nanosecond", &["ns", "nanosecond"], "ns", 1e-9, dims(&[(TIME, 1)])),
    u("minute", "Minute", &["min", "minute"], "min", 60.0, dims(&[(TIME, 1)])),
    u("hour", "Hour", &["h", "hr", "hour"], "h", 3600.0, dims(&[(TIME, 1)])),
    u("day", "Day", &["day"], "day", 86400.0, dims(&[(TIME, 1)])),
    u("week", "Week", &["wk", "week"], "wk", 604800.0, dims(&[(TIME, 1)])),
    u("month", "Month", &["month"], "month", 2629746.0, dims(&[(TIME, 1)])),
    u("year", "Year", &["yr", "year"], "yr", 31557600.0, dims(&[(TIME, 1)])),
    // temperature (affine)
    u("kelvin", "Kelvin", &["k", "kelvin"], "K", 1.0, dims(&[(TEMP, 1)])),
    affine("celsius", "Celsius", &["c", "°c", "celsius", "celcius"], "°C", 1.0, 273.15, dims(&[(TEMP, 1)])),
    affine(
        "fahrenheit",
        "Fahrenheit",
        &["f", "°f", "fahrenheit", "farenheit", "farenheight"],
        "°F",
        5.0 / 9.0,
        273.15 - 32.0 * 5.0 / 9.0,
        dims(&[(TEMP, 1)]),
    ),
    // electric current
    u("ampere", "Ampere", &["a", "amp", "ampere"], "A", 1.0, dims(&[(CURRENT, 1)])),
    u("milliampere", "Milliampere", &["ma", "milliamp", "milliampere"], "mA", 1e-3, dims(&[(CURRENT, 1)])),
    // data — mixed-case names ("GB" vs "Gb") are case-sensitive
    u("byte", "Byte", &["byte", "B"], "B", 1.0, dims(&[(DATA, 1)])),
    u("bit", "Bit", &["bit"], "bit", 0.125, dims(&[(DATA, 1)])),
    u("kilobyte", "Kilobyte", &["kilobyte", "kb", "KB"], "kB", 1e3, dims(&[(DATA, 1)])),
    u("kilobit", "Kilobit", &["kilobit", "kbit", "Kb"], "Kb", 125.0, dims(&[(DATA, 1)])),
    u("kibibyte", "Kibibyte", &["kibibyte", "kib", "KiB"], "KiB", 1024.0, dims(&[(DATA, 1)])),
    u("megabyte", "Megabyte", &["megabyte", "mb", "MB"], "MB", 1e6, dims(&[(DATA, 1)])),
    u("megabit", "Megabit", &["megabit", "mbit", "Mb"], "Mb", 125_000.0, dims(&[(DATA, 1)])),
    u("mebibyte", "Mebibyte", &["mebibyte", "mib", "MiB"], "MiB", 1_048_576.0, dims(&[(DATA, 1)])),
    u("gigabyte", "Gigabyte", &["gigabyte", "gb", "GB"], "GB", 1e9, dims(&[(DATA, 1)])),
    u("gigabit", "Gigabit", &["gigabit", "gbit", "Gb"], "Gb", 125_000_000.0, dims(&[(DATA, 1)])),
    u("gibibyte", "Gibibyte", &["gibibyte", "gib", "GiB"], "GiB", 1_073_741_824.0, dims(&[(DATA, 1)])),
    u("terabyte", "Terabyte", &["terabyte", "tb", "TB"], "TB", 1e12, dims(&[(DATA, 1)])),
    u("terabit", "Terabit", &["terabit", "tbit", "Tb"], "Tb", 125_000_000_000.0, dims(&[(DATA, 1)])),
    u("tebibyte", "Tebibyte", &["tebibyte", "tib", "TiB"], "TiB", 1_099_511_627_776.0, dims(&[(DATA, 1)])),
    u("petabyte", "Petabyte", &["petabyte", "pb", "PB"], "PB", 1e15, dims(&[(DATA, 1)])),
    u("petabit", "Petabit", &["petabit", "pbit", "Pb"], "Pb", 125_000_000_000_000.0, dims(&[(DATA, 1)])),
    u("pebibyte", "Pebibyte", &["pebibyte", "pib", "PiB"], "PiB", 1_125_899_906_842_624.0, dims(&[(DATA, 1)])),
    // volume
    u("liter", "Liter", &["l", "liter", "litre"], "L", 1e-3, dims(&[(LEN, 3)])),
    u("milliliter", "Milliliter", &["ml", "milliliter", "millilitre"], "mL", 1e-6, dims(&[(LEN, 3)])),
    u("gallon", "Gallon", &["gal", "gallon"], "gal", 3.785411784e-3, dims(&[(LEN, 3)])),
    u("quart", "Quart", &["qt", "quart"], "qt", 9.46352946e-4, dims(&[(LEN, 3)])),
    u("pint", "Pint", &["pt", "pint"], "pt", 4.73176473e-4, dims(&[(LEN, 3)])),
    u("cup", "Cup", &["cup"], "cup", 2.365882365e-4, dims(&[(LEN, 3)])),
    // speed
    u("kmh", "Kilometer per Hour", &["kmh", "kph", "kmph"], "km/h", 1000.0 / 3600.0, dims(&[(LEN, 1), (TIME, -1)])),
    u("mph", "Mile per Hour", &["mph"], "mph", 0.44704, dims(&[(LEN, 1), (TIME, -1)])),
    u("knot", "Knot", &["kn", "knot"], "kn", 0.514444444444, dims(&[(LEN, 1), (TIME, -1)])),
    // force / pressure / energy / power
    u("newton", "Newton", &["n", "newton"], "N", 1.0, dims(&[(MASS, 1), (LEN, 1), (TIME, -2)])),
    u("pascal", "Pascal", &["pa", "pascal"], "Pa", 1.0, dims(&[(MASS, 1), (LEN, -1), (TIME, -2)])),
    u("kilopascal", "Kilopascal", &["kpa", "kilopascal"], "kPa", 1e3, dims(&[(MASS, 1), (LEN, -1), (TIME, -2)])),
    u("bar", "Bar", &["bar"], "bar", 1e5, dims(&[(MASS, 1), (LEN, -1), (TIME, -2)])),
    u("atmosphere", "Atmosphere", &["atm", "atmosphere"], "atm", 101325.0, dims(&[(MASS, 1), (LEN, -1), (TIME, -2)])),
    u("psi", "PSI", &["psi"], "psi", 6894.75729317, dims(&[(MASS, 1), (LEN, -1), (TIME, -2)])),
    u("joule", "Joule", &["j", "joule"], "J", 1.0, dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    u("kilojoule", "Kilojoule", &["kj", "kilojoule"], "kJ", 1e3, dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    u("calorie", "Calorie", &["cal", "calorie"], "cal", 4.184, dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    u("kilocalorie", "Kilocalorie", &["kcal", "kilocalorie"], "kcal", 4184.0, dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    u("watthour", "Watt Hour", &["wh", "watthour"], "Wh", 3600.0, dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    u("kilowatthour", "Kilowatt Hour", &["kwh", "kilowatthour"], "kWh", 3.6e6, dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    u("watt", "Watt", &["w", "watt"], "W", 1.0, dims(&[(MASS, 1), (LEN, 2), (TIME, -3)])),
    u("kilowatt", "Kilowatt", &["kw", "kilowatt"], "kW", 1e3, dims(&[(MASS, 1), (LEN, 2), (TIME, -3)])),
    u("megawatt", "Megawatt", &["mwatt", "megawatt"], "MW", 1e6, dims(&[(MASS, 1), (LEN, 2), (TIME, -3)])),
    u("horsepower", "Horsepower", &["hp", "horsepower"], "hp", 745.699871582, dims(&[(MASS, 1), (LEN, 2), (TIME, -3)])),
    u("hertz", "Hertz", &["hz", "hertz"], "Hz", 1.0, dims(&[(TIME, -1)])),
    // angle (dimensionless ratios)
    u("degree", "Degree", &["deg", "degree"], "°", std::f64::consts::PI / 180.0, dims(&[])),
    u("radian", "Radian", &["rad", "radian"], "rad", 1.0, dims(&[])),
];

/// Default conversion targets for bare `value unit` queries (`5c` → °F).
static DEFAULT_TARGETS: &[(&str, &str)] = &[
    ("celsius", "fahrenheit"),
    ("fahrenheit", "celsius"),
    ("kelvin", "celsius"),
    ("meter", "foot"),
    ("foot", "meter"),
    ("kilometer", "mile"),
    ("mile", "kilometer"),
    ("centimeter", "inch"),
    ("millimeter", "inch"),
    ("inch", "centimeter"),
    ("yard", "meter"),
    ("nauticalmile", "kilometer"),
    ("kilogram", "pound"),
    ("pound", "kilogram"),
    ("gram", "ounce"),
    ("ounce", "gram"),
    ("stone", "kilogram"),
    ("liter", "gallon"),
    ("gallon", "liter"),
    ("cup", "milliliter"),
    ("byte", "bit"),
    ("bit", "byte"),
    ("kilobyte", "kibibyte"),
    ("kibibyte", "kilobyte"),
    ("megabyte", "mebibyte"),
    ("mebibyte", "megabyte"),
    ("gigabyte", "gibibyte"),
    ("gibibyte", "gigabyte"),
    ("terabyte", "tebibyte"),
    ("tebibyte", "terabyte"),
    ("petabyte", "pebibyte"),
    ("pebibyte", "petabyte"),
    ("kmh", "mph"),
    ("mph", "kmh"),
];

pub fn default_target(def: &UnitDef) -> Option<&'static UnitDef> {
    let target = DEFAULT_TARGETS
        .iter()
        .find(|(from, _)| *from == def.id)?
        .1;
    UNITS.iter().find(|d| d.id == target)
}

/// Look up a unit by name: exact case-sensitive match first, then
/// case-insensitive (all-lowercase names only, so `Gb` ≠ `gb`), then
/// plural stripping, then nucleo fuzzy matching for typos.
pub fn lookup(name: &str) -> Option<&'static UnitDef> {
    // exact (case-sensitive)
    if let Some(def) = UNITS.iter().find(|d| d.names.contains(&name)) {
        return Some(def);
    }
    // case-insensitive against all-lowercase names
    let lower = name.to_lowercase();
    if lower != name {
        if let Some(def) = UNITS.iter().find(|d| {
            d.names
                .iter()
                .any(|n| n.chars().all(|c| !c.is_uppercase()) && *n == lower)
        }) {
            return Some(def);
        }
    }
    // plurals: "meters" -> "meter", "inches" -> "inch"
    for stripped in [lower.strip_suffix('s'), lower.strip_suffix("es").map(|s| s)]
        .into_iter()
        .flatten()
    {
        if stripped.len() > 1 {
            if let Some(def) = UNITS.iter().find(|d| {
                d.names
                    .iter()
                    .any(|n| n.chars().all(|c| !c.is_uppercase()) && *n == stripped)
            }) {
                return Some(def);
            }
        }
    }
    fuzzy_lookup(&lower)
}

/// Typo tolerance via nucleo fuzzy matching (same matcher as the launcher).
/// Only for word-length input; abbreviations must be spelled correctly.
fn fuzzy_lookup(name: &str) -> Option<&'static UnitDef> {
    if name.len() < 4 {
        return None;
    }
    let pattern = Pattern::new(name, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy);
    let mut matcher = Matcher::default();
    let mut best: Option<(u32, &UnitDef)> = None;
    for def in UNITS {
        for candidate in def.names {
            let haystack = nucleo_matcher::Utf32String::from(*candidate);
            if let Some(score) = pattern.score(haystack.slice(..), &mut matcher) {
                if best.is_none_or(|(best_score, _)| score > best_score) {
                    best = Some((score, def));
                }
            }
        }
    }
    best.map(|(_, def)| def)
}

impl UnitDef {
    pub fn disp_unit(&self, exp: i8) -> DispUnit {
        DispUnit {
            symbol: self.symbol,
            id: self.id,
            name: self.name,
            factor: self.factor,
            offset: self.offset,
            exp,
            dims: self.dims,
        }
    }
}
