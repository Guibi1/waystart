use crate::CalcError;

/// Base dimension indices.
pub const LEN: usize = 0;
pub const MASS: usize = 1;
pub const TIME: usize = 2;
pub const CURRENT: usize = 3;
pub const TEMP: usize = 4;
pub const DATA: usize = 5;

pub const BASE_SYMBOLS: [&str; 6] = ["m", "kg", "s", "A", "K", "B"];

pub type Dims = [i8; 6];

pub const DIMENSIONLESS: Dims = [0; 6];

pub const fn dims(entries: &[(usize, i8)]) -> Dims {
    let mut d = DIMENSIONLESS;
    let mut k = 0;
    while k < entries.len() {
        let (i, e) = entries[k];
        d[i] = e;
        k += 1;
    }
    d
}

/// One unit in a display compound, e.g. `km` in `km/h`.
#[derive(Debug, Clone, Copy)]
pub struct DispUnit {
    pub symbol: &'static str,
    /// canonical unit id, e.g. "gibibyte"
    pub id: &'static str,
    /// human-readable name, e.g. "Gibibyte"
    pub name: &'static str,
    /// factor to SI base (value_si = display_value * factor + offset)
    pub factor: f64,
    /// affine offset, only meaningful for single-unit quantities (temperature)
    pub offset: f64,
    pub exp: i8,
    /// dimensions of the unit itself (unraised)
    pub dims: Dims,
}

/// Well-known derived units used to simplify compound results
/// (`1kg*1m/1s^2` displays as `1 N`). (symbol, id, name, dims)
pub const DERIVED: &[(&str, &str, &str, Dims)] = &[
    ("Hz", "hertz", "Hertz", dims(&[(TIME, -1)])),
    ("N", "newton", "Newton", dims(&[(MASS, 1), (LEN, 1), (TIME, -2)])),
    ("Pa", "pascal", "Pascal", dims(&[(MASS, 1), (LEN, -1), (TIME, -2)])),
    ("J", "joule", "Joule", dims(&[(MASS, 1), (LEN, 2), (TIME, -2)])),
    ("W", "watt", "Watt", dims(&[(MASS, 1), (LEN, 2), (TIME, -3)])),
];

/// A evaluated quantity: SI value, physical dimensions, and a preferred
/// display unit compound (so `30m/2s` renders as `15 m/s`).
#[derive(Debug, Clone)]
pub struct Quantity {
    pub si: f64,
    pub dims: Dims,
    pub disp: Vec<DispUnit>,
    /// true when the value came straight from a `%` literal; enables
    /// Raycast-style `80 + 25%` = 80 + 25% of 80
    pub pct: bool,
}

impl Quantity {
    pub fn plain(value: f64) -> Self {
        Self {
            si: value,
            dims: DIMENSIONLESS,
            disp: Vec::new(),
            pct: false,
        }
    }

    pub fn is_dimensionless(&self) -> bool {
        self.dims == DIMENSIONLESS
    }

    fn normalize_disp(&mut self) {
        self.disp.retain(|u| u.exp != 0);
        let mut i = 0;
        while i < self.disp.len() {
            let mut j = i + 1;
            while j < self.disp.len() {
                if self.disp[i].symbol == self.disp[j].symbol && self.disp[i].factor == self.disp[j].factor {
                    self.disp[i].exp += self.disp[j].exp;
                    self.disp.remove(j);
                } else {
                    j += 1;
                }
            }
            if self.disp[i].exp == 0 {
                self.disp.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Replace a compound display unit with a well-known derived unit
    /// (N, J, W, ...) unless the user already wrote a single named unit
    /// of the same dimension (keep `kWh`, `min`, `°C`).
    fn simplify_disp(&mut self) {
        for (sym, id, name, d) in DERIVED {
            if *d == self.dims {
                let single_same = self.disp.len() == 1 && self.disp[0].dims == self.dims;
                if !single_same {
                    self.disp = vec![DispUnit {
                        symbol: sym,
                        id,
                        name,
                        factor: 1.0,
                        offset: 0.0,
                        exp: 1,
                        dims: self.dims,
                    }];
                }
                return;
            }
        }
    }

    fn check_finite(&self) -> Result<(), CalcError> {
        if self.si.is_finite() {
            Ok(())
        } else {
            Err(CalcError::NotFinite)
        }
    }

    pub fn neg(&self) -> Result<Quantity, CalcError> {
        let mut q = self.clone();
        // `-40 c` is -40 °C, not -(40 °C in kelvin): negate the display value
        if q.disp.len() == 1 && q.disp[0].offset != 0.0 && q.disp[0].exp == 1 {
            q.si = 2.0 * q.disp[0].offset - q.si;
        } else {
            q.si = -q.si;
        }
        q.check_finite()?;
        Ok(q)
    }

    pub fn add(&self, other: &Quantity) -> Result<Quantity, CalcError> {
        let operand = self.operand_of(other)?;
        let mut q = self.clone();
        q.si += operand;
        q.pct = false;
        q.check_finite()?;
        Ok(q)
    }

    pub fn sub(&self, other: &Quantity) -> Result<Quantity, CalcError> {
        let operand = self.operand_of(other)?;
        let mut q = self.clone();
        q.si -= operand;
        q.pct = false;
        q.check_finite()?;
        Ok(q)
    }

    /// The effective addend/subtrahend: a `%` literal means "percent of
    /// the left operand" (`80 + 25%` → 20), anything else must have
    /// matching dimensions.
    fn operand_of(&self, other: &Quantity) -> Result<f64, CalcError> {
        if other.pct {
            Ok(self.si * other.si)
        } else {
            self.require_same_dims(other)?;
            Ok(other.si)
        }
    }

    pub fn mul(&self, other: &Quantity) -> Result<Quantity, CalcError> {
        // affine units (temperature) cannot be multiplied
        if self.has_affine_disp() && !self.disp.is_empty() && !other.is_dimensionless_plain() {
            return Err(CalcError::AffineArithmetic);
        }
        if other.has_affine_disp() && !other.disp.is_empty() {
            return Err(CalcError::AffineArithmetic);
        }
        let mut q = Quantity {
            si: self.si * other.si,
            dims: add_dims(self.dims, other.dims)?,
            disp: [self.disp.clone(), other.disp.clone()].concat(),
            pct: false,
        };
        q.normalize_disp();
        q.simplify_disp();
        q.check_finite()?;
        Ok(q)
    }

    pub fn div(&self, other: &Quantity) -> Result<Quantity, CalcError> {
        if other.si == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        if self.has_affine_disp() && !self.disp.is_empty() {
            return Err(CalcError::AffineArithmetic);
        }
        if other.has_affine_disp() && !other.disp.is_empty() {
            return Err(CalcError::AffineArithmetic);
        }
        let mut negated = other.disp.clone();
        for u in &mut negated {
            u.exp = -u.exp;
        }
        let mut q = Quantity {
            si: self.si / other.si,
            dims: sub_dims(self.dims, other.dims)?,
            disp: [self.disp.clone(), negated].concat(),
            pct: false,
        };
        q.normalize_disp();
        q.simplify_disp();
        q.check_finite()?;
        Ok(q)
    }

    pub fn pow(&self, exp: &Quantity) -> Result<Quantity, CalcError> {
        if !exp.is_dimensionless() {
            return Err(CalcError::DimensionfulExponent);
        }
        let e = exp.si;
        if e.fract() == 0.0 && e.abs() <= 16.0 {
            let n = e as i8;
            let mut q = Quantity {
                si: self.si.powi(n as i32),
                dims: scale_dims(self.dims, n)?,
                disp: self
                    .disp
                    .iter()
                    .map(|u| DispUnit { exp: u.exp * n, ..*u })
                    .collect(),
                pct: false,
            };
            q.normalize_disp();
            q.check_finite()?;
            Ok(q)
        } else {
            if !self.is_dimensionless() {
                return Err(CalcError::FractionalDimensionfulExponent);
            }
            let q = Quantity::plain(self.si.powf(e));
            q.check_finite()?;
            Ok(q)
        }
    }

    pub fn percent(&self) -> Quantity {
        let mut q = self.clone();
        q.si /= 100.0;
        q.pct = true;
        q
    }

    /// Attach a unit to a plain number: `20 C`, `30 m`.
    /// Affine units (temperature) convert absolute values: 20 °C = 293.15 K.
    pub fn with_unit(&self, unit: DispUnit, dim: Dims) -> Result<Quantity, CalcError> {
        if !self.is_dimensionless() || !self.disp.is_empty() {
            // multiplying an existing quantity by a unit is plain multiplication,
            // but affine units only make sense standalone
            if unit.offset != 0.0 {
                return Err(CalcError::AffineArithmetic);
            }
            return self.mul(&Quantity {
                si: unit.factor,
                dims: dim,
                disp: vec![unit],
                pct: false,
            });
        }
        if unit.offset != 0.0 {
            Ok(Quantity {
                si: self.si * unit.factor + unit.offset,
                dims: dim,
                disp: vec![unit],
                pct: false,
            })
        } else {
            Ok(Quantity {
                si: self.si * unit.factor,
                dims: dim,
                disp: vec![unit],
                pct: false,
            })
        }
    }

    /// Convert into the unit of another quantity: `30m/2s to km/h`.
    pub fn convert_to(&self, target: &Quantity) -> Result<Quantity, CalcError> {
        if target.dims == DIMENSIONLESS || target.disp.is_empty() {
            return Err(CalcError::InvalidConversionTarget);
        }
        if self.dims != target.dims {
            return Err(CalcError::MismatchedDimensions {
                from: self.unit_string(),
                to: target.unit_string(),
            });
        }
        Ok(Quantity {
            si: self.si,
            dims: self.dims,
            disp: target.normalized_disp(),
            pct: false,
        })
    }

    fn normalized_disp(&self) -> Vec<DispUnit> {
        let mut d = self.disp.clone();
        // collapse a target like `2 ft` or `1000 m` down to its unit compound
        for u in &mut d {
            u.exp = u.exp.signum();
        }
        // normalize keeps sign-only exponents merged
        let mut q = Quantity { si: 1.0, dims: self.dims, disp: d, pct: false };
        q.normalize_disp();
        q.disp
    }

    fn require_same_dims(&self, other: &Quantity) -> Result<(), CalcError> {
        if self.dims == other.dims {
            Ok(())
        } else {
            Err(CalcError::MismatchedDimensions {
                from: self.unit_string(),
                to: other.unit_string(),
            })
        }
    }

    fn is_dimensionless_plain(&self) -> bool {
        self.is_dimensionless() && self.disp.is_empty()
    }

    fn has_affine_disp(&self) -> bool {
        self.disp.iter().any(|u| u.offset != 0.0)
    }

    /// The value as displayed (SI converted through the display units).
    pub fn display_value(&self) -> f64 {
        if self.disp.len() == 1 && self.disp[0].offset != 0.0 && self.disp[0].exp == 1 {
            (self.si - self.disp[0].offset) / self.disp[0].factor
        } else {
            let factor: f64 = self.disp.iter().map(|u| u.factor.powi(u.exp as i32)).product();
            self.si / factor
        }
    }

    /// Unit string like `m/s`, `km/h`, `kg·m/s²`. Empty for dimensionless.
    pub fn unit_string(&self) -> String {
        if self.disp.is_empty() && self.dims != DIMENSIONLESS {
            // derive from dims, preferring well-known derived units
            for (sym, _, _, d) in DERIVED {
                if *d == self.dims {
                    return sym.to_string();
                }
            }
            if self.dims == dims(&[(LEN, 3)]) {
                return "L".to_string();
            }
            let units: Vec<DispUnit> = self
                .dims
                .iter()
                .enumerate()
                .filter(|&(_, &e)| e != 0)
                .map(|(i, &e)| DispUnit {
                    symbol: BASE_SYMBOLS[i],
                    id: BASE_SYMBOLS[i],
                    name: BASE_SYMBOLS[i],
                    factor: 1.0,
                    offset: 0.0,
                    exp: e,
                    dims: dims(&[(i, 1)]),
                })
                .collect();
            return format_units(&units);
        }
        format_units(&self.disp)
    }
}

pub fn format_units(units: &[DispUnit]) -> String {
    let num: Vec<&DispUnit> = units.iter().filter(|u| u.exp > 0).collect();
    let den: Vec<&DispUnit> = units.iter().filter(|u| u.exp < 0).collect();

    let mut out = String::new();
    if num.is_empty() && !den.is_empty() {
        out.push('1');
    } else {
        let parts: Vec<String> = num.iter().map(|u| format_unit(u)).collect();
        out.push_str(&parts.join("·"));
    }
    if !den.is_empty() {
        let parts: Vec<String> = den
            .iter()
            .map(|u| {
                let mut u = **u;
                u.exp = -u.exp;
                format_unit(&u)
            })
            .collect();
        out.push('/');
        out.push_str(&parts.join("·"));
    }
    out
}

fn format_unit(u: &DispUnit) -> String {
    match u.exp {
        1 => u.symbol.to_string(),
        2 => format!("{}²", u.symbol),
        3 => format!("{}³", u.symbol),
        e => format!("{}^{}", u.symbol, e),
    }
}

pub fn add_dims(a: Dims, b: Dims) -> Result<Dims, CalcError> {
    let mut out = DIMENSIONLESS;
    for i in 0..out.len() {
        out[i] = a[i]
            .checked_add(b[i])
            .ok_or(CalcError::DimensionOverflow)?;
    }
    Ok(out)
}

pub fn sub_dims(a: Dims, b: Dims) -> Result<Dims, CalcError> {
    let mut out = DIMENSIONLESS;
    for i in 0..out.len() {
        out[i] = a[i]
            .checked_sub(b[i])
            .ok_or(CalcError::DimensionOverflow)?;
    }
    Ok(out)
}

pub fn scale_dims(a: Dims, n: i8) -> Result<Dims, CalcError> {
    let mut out = DIMENSIONLESS;
    for i in 0..out.len() {
        out[i] = a[i]
            .checked_mul(n)
            .ok_or(CalcError::DimensionOverflow)?;
    }
    Ok(out)
}

/// Format a float the way Raycast does: integers stay integers,
/// floats get up to 10 decimal places with trailing zeros trimmed.
pub fn format_number(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    // kill floating point noise (14.999999999999998)
    let rounded = (v * 1e10).round() / 1e10;
    if rounded.fract() == 0.0 && rounded.abs() < 1e15 {
        return format!("{}", rounded as i64);
    }
    let s = format!("{:.10}", rounded);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

impl std::fmt::Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = format_number(self.display_value());
        let unit = self.unit_string();
        if unit.is_empty() {
            write!(f, "{}", value)
        } else {
            write!(f, "{} {}", value, unit)
        }
    }
}
