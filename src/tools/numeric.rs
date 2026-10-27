//! Bounded checked rational arithmetic over explicit decimal strings.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const INVALID: &str = "Numeric inputs exceed the supported decimal or operation limits.";
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NumericOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    PercentageOf,
    PercentageChange,
    ConvertUnit,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NumericRequest {
    pub operation: NumericOperation,
    /// Plain decimal notation: at most 18 integer and 9 fractional digits; no exponent or separators.
    #[schemars(length(min = 1, max = 29))]
    pub value: String,
    #[serde(default)]
    #[schemars(length(min = 1, max = 29))]
    pub other_value: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 3))]
    pub from_unit: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 3))]
    pub to_unit: Option<String>,
    /// Presentation precision, not an accuracy guarantee. Default 6; maximum 9.
    #[serde(default = "default_places")]
    #[schemars(range(min = 0, max = 9))]
    pub decimal_places: u32,
}
fn default_places() -> u32 {
    6
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct NumericOutput {
    pub inputs: NumericRequest,
    pub result: String,
    pub result_unit: Option<String>,
    pub exact_fraction: String,
    pub rounded: bool,
    pub rounding_rule: String,
    pub explanation: String,
    pub limitations: Vec<String>,
}
#[derive(Clone, Copy)]
struct Fraction {
    n: i128,
    d: i128,
}
fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let next = a % b;
        a = b;
        b = next;
    }
    a
}
impl Fraction {
    fn new(n: i128, d: i128) -> Result<Self, String> {
        if d == 0 {
            return Err("Division by zero is not supported.".into());
        }
        let sign = if d < 0 { -1 } else { 1 };
        let n = n.checked_mul(sign).ok_or(INVALID)?;
        let d = d.checked_abs().ok_or(INVALID)?;
        let g = gcd(n.checked_abs().ok_or(INVALID)?, d);
        Ok(Self { n: n / g, d: d / g })
    }
    fn mul(self, other: Self) -> Result<Self, String> {
        let a = gcd(self.n.checked_abs().ok_or(INVALID)?, other.d);
        let b = gcd(other.n.checked_abs().ok_or(INVALID)?, self.d);
        Self::new(
            (self.n / a).checked_mul(other.n / b).ok_or(INVALID)?,
            (self.d / b).checked_mul(other.d / a).ok_or(INVALID)?,
        )
    }
    fn div(self, other: Self) -> Result<Self, String> {
        if other.n == 0 {
            return Err("Division by zero is not supported.".into());
        }
        self.mul(Self::new(other.d, other.n)?)
    }
    fn add(self, other: Self, subtract: bool) -> Result<Self, String> {
        let g = gcd(self.d, other.d);
        let left = self.n.checked_mul(other.d / g).ok_or(INVALID)?;
        let right = other.n.checked_mul(self.d / g).ok_or(INVALID)?;
        let n = if subtract {
            left.checked_sub(right)
        } else {
            left.checked_add(right)
        }
        .ok_or(INVALID)?;
        Self::new(n, (self.d / g).checked_mul(other.d).ok_or(INVALID)?)
    }
}
fn parse(s: &str) -> Result<Fraction, String> {
    if s.is_empty() || s.len() > 29 || s.trim() != s {
        return Err(INVALID.into());
    }
    let (negative, body) = if let Some(body) = s.strip_prefix('-') {
        (true, body)
    } else {
        (false, s.strip_prefix('+').unwrap_or(s))
    };
    let mut parts = body.split('.');
    let whole = parts.next().ok_or(INVALID)?;
    let fractional = parts.next();
    if parts.next().is_some()
        || whole.is_empty()
        || whole.len() > 18
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fractional
            .is_some_and(|f| f.is_empty() || f.len() > 9 || !f.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(INVALID.into());
    }
    let f = fractional.unwrap_or("");
    let denominator = 10_i128.pow(f.len() as u32);
    let whole = whole.parse::<i128>().map_err(|_| INVALID)?;
    let frac = if f.is_empty() {
        0
    } else {
        f.parse::<i128>().map_err(|_| INVALID)?
    };
    let n = whole
        .checked_mul(denominator)
        .and_then(|n| n.checked_add(frac))
        .ok_or(INVALID)?;
    Fraction::new(if negative { -n } else { n }, denominator)
}
// Ratios are exact relative to millimetres, milligrams, millilitres or milliseconds.
fn unit(name: &str) -> Option<(&'static str, i128)> {
    Some(match name {
        "mm" => ("length", 1),
        "cm" => ("length", 10),
        "m" => ("length", 1000),
        "km" => ("length", 1_000_000),
        "mg" => ("mass", 1),
        "g" => ("mass", 1000),
        "kg" => ("mass", 1_000_000),
        "ml" => ("volume", 1),
        "l" => ("volume", 1000),
        "ms" => ("time", 1),
        "s" => ("time", 1000),
        "min" => ("time", 60_000),
        "h" => ("time", 3_600_000),
        _ => return None,
    })
}
fn display(f: Fraction, places: u32) -> Result<(String, bool), String> {
    let scale = 10_i128.pow(places);
    let magnitude = f.n.checked_abs().ok_or(INVALID)?;
    // Separate the integer part to avoid overflowing merely to format a large value.
    let whole = magnitude / f.d;
    let remainder = magnitude % f.d;
    let scaled = remainder.checked_mul(scale).ok_or(INVALID)?;
    let mut fractional = scaled / f.d;
    let residual = scaled % f.d;
    let rounded = residual != 0;
    // Comparing with ceil(d/2) avoids multiplying the residual by two.
    if residual >= f.d / 2 + f.d % 2 {
        fractional = fractional.checked_add(1).ok_or(INVALID)?;
    }
    let whole = whole.checked_add(fractional / scale).ok_or(INVALID)?;
    fractional %= scale;
    let sign = if f.n < 0 && (whole != 0 || fractional != 0) {
        "-"
    } else {
        ""
    };
    let mut result = if places == 0 {
        format!("{sign}{whole}")
    } else {
        format!(
            "{sign}{whole}.{fractional:0width$}",
            width = places as usize
        )
    };
    if places > 0 {
        while result.ends_with('0') {
            result.pop();
        }
        if result.ends_with('.') {
            result.pop();
        }
    }
    Ok((result, rounded))
}
pub fn explain_numeric_values(request: NumericRequest) -> Result<NumericOutput, String> {
    if request.decimal_places > 9 {
        return Err(INVALID.into());
    }
    let left = parse(&request.value)?;
    let (result, result_unit, explanation) = if request.operation == NumericOperation::ConvertUnit {
        if request.other_value.is_some() {
            return Err("Unit conversion accepts one numeric value.".into());
        }
        let from = request
            .from_unit
            .as_deref()
            .and_then(unit)
            .ok_or("Choose a supported source unit.")?;
        let to = request
            .to_unit
            .as_deref()
            .and_then(unit)
            .ok_or("Choose a supported target unit.")?;
        if from.0 != to.0 {
            return Err("Conversion requires units of the same dimension.".into());
        }
        (
            left.mul(Fraction::new(from.1, to.1)?)?,
            request.to_unit.clone(),
            "Multiply the supplied value by the exact ratio of the source and target units.".into(),
        )
    } else {
        if request.from_unit.is_some() || request.to_unit.is_some() {
            return Err("Units are accepted only for unit conversion.".into());
        }
        let right = parse(
            request
                .other_value
                .as_deref()
                .ok_or("This operation requires a second decimal value.")?,
        )?;
        match request.operation {
            NumericOperation::Add => (
                left.add(right, false)?,
                None,
                "Add the two supplied decimal values exactly before presentation rounding.".into(),
            ),
            NumericOperation::Subtract => (
                left.add(right, true)?,
                None,
                "Subtract other_value from value.".into(),
            ),
            NumericOperation::Multiply => (
                left.mul(right)?,
                None,
                "Multiply the two supplied decimal values.".into(),
            ),
            NumericOperation::Divide => (
                left.div(right)?,
                None,
                "Divide value by other_value.".into(),
            ),
            NumericOperation::PercentageOf => (
                left.mul(right)?.div(Fraction::new(100, 1)?)?,
                None,
                "Treat value as a percentage: value / 100 × other_value.".into(),
            ),
            NumericOperation::PercentageChange => (
                right.add(left, true)?.div(left)?.mul(Fraction::new(100, 1)?)?,
                Some("%".into()),
                "Treat value as the old value and other_value as the new value: (new − old) / old × 100. A zero old value is undefined; negative baselines need contextual interpretation.".into(),
            ),
            NumericOperation::ConvertUnit => return Err(INVALID.into()),
        }
    };
    let (value, rounded) = display(result, request.decimal_places)?;
    Ok(NumericOutput {
        inputs: request,
        result: value,
        result_unit,
        exact_fraction: format!("{}/{}", result.n, result.d),
        rounded,
        rounding_rule: "Nearest at requested decimal_places; exact halfway values round away from zero. Trailing zeros are omitted.".into(),
        explanation,
        limitations: vec![
            "Uses checked integer rational arithmetic, not binary floating point; operations that exceed the bounded arithmetic range are rejected.".into(),
            "Does not infer quantities or units from prose, assess measurement uncertainty, convert currencies or advise on doses. Verify the supplied values and their context.".into(),
        ],
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn req(operation: NumericOperation, value: &str, other: &str) -> NumericRequest {
        NumericRequest {
            operation,
            value: value.into(),
            other_value: Some(other.into()),
            from_unit: None,
            to_unit: None,
            decimal_places: 6,
        }
    }
    #[test]
    fn decimal_arithmetic_is_exact() -> Result<(), String> {
        let o = explain_numeric_values(req(NumericOperation::Add, "0.1", "0.2"))?;
        assert_eq!(o.result, "0.3");
        assert_eq!(o.exact_fraction, "3/10");
        assert!(!o.rounded);
        Ok(())
    }
    #[test]
    fn repeating_and_negative_rounding() -> Result<(), String> {
        let mut r = req(NumericOperation::Divide, "1", "3");
        r.decimal_places = 2;
        let o = explain_numeric_values(r)?;
        assert_eq!(o.result, "0.33");
        assert!(o.rounded);
        let mut r = req(NumericOperation::Multiply, "-1.005", "1");
        r.decimal_places = 2;
        assert_eq!(explain_numeric_values(r)?.result, "-1.01");
        Ok(())
    }
    #[test]
    fn conversion_and_rejections() -> Result<(), String> {
        let r = NumericRequest {
            operation: NumericOperation::ConvertUnit,
            value: "90".into(),
            other_value: None,
            from_unit: Some("min".into()),
            to_unit: Some("h".into()),
            decimal_places: 6,
        };
        assert_eq!(explain_numeric_values(r)?.result, "1.5");
        assert!(explain_numeric_values(req(NumericOperation::Divide, "1", "0")).is_err());
        assert!(explain_numeric_values(req(NumericOperation::Add, "1e3", "1")).is_err());
        assert!(
            explain_numeric_values(req(
                NumericOperation::Multiply,
                "999999999999999999.999999999",
                "999999999999999999.999999999"
            ))
            .is_err()
        );
        Ok(())
    }
    #[test]
    fn percentage_contract() -> Result<(), String> {
        assert_eq!(
            explain_numeric_values(req(NumericOperation::PercentageOf, "25", "80"))?.result,
            "20"
        );
        assert_eq!(
            explain_numeric_values(req(NumericOperation::PercentageChange, "80", "100"))?.result,
            "25"
        );
        Ok(())
    }
}
