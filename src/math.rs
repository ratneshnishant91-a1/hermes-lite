use regex::Regex;

pub fn evaluate(input: &str) -> Option<String> {
    let q = input.trim().to_lowercase();
    let pairs = [
        (r"^(?:add|sum)\s+([-+]?\d+(?:\.\d+)?)\s+(?:and|with)\s+([-+]?\d+(?:\.\d+)?)$", |a: f64,b: f64| a+b),
        (r"^subtract\s+([-+]?\d+(?:\.\d+)?)\s+from\s+([-+]?\d+(?:\.\d+)?)$", |a,b| b-a),
        (r"^multiply\s+([-+]?\d+(?:\.\d+)?)\s+(?:by|and)\s+([-+]?\d+(?:\.\d+)?)$", |a,b| a*b),
        (r"^divide\s+([-+]?\d+(?:\.\d+)?)\s+by\s+([-+]?\d+(?:\.\d+)?)$", |a,b| a/b),
    ];
    for (pattern, operation) in pairs {
        let cap = Regex::new(pattern).ok()?.captures(&q)?;
        let a: f64 = cap.get(1)?.as_str().parse().ok()?;
        let b: f64 = cap.get(2)?.as_str().parse().ok()?;
        if pattern.contains("divide") && b == 0.0 { return Some("Division by zero is undefined.".into()); }
        return Some(format_number(operation(a,b)));
    }
    if q.chars().any(|c| "+-*/^()".contains(c)) && q.chars().any(|c| c.is_ascii_digit()) {
        return meval::eval_str(q).ok().map(format_number);
    }
    None
}
fn format_number(n: f64) -> String { if n.fract() == 0.0 { format!("{n:.0}") } else { n.to_string() } }
