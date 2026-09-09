//! Small, deterministic natural-language math router.
//!
//! This deliberately handles only unambiguous arithmetic. Anything involving
//! variables, units, equations, or unclear wording returns `None` so the agent
//! can ask the LLM rather than inventing an answer.

use regex::Regex;

pub fn evaluate_query(query: &str) -> Option<String> {
    let raw = query.trim();
    let q = normalize(raw);

    // First accept regular expression syntax: `2 + 2`, `sqrt(9)`, etc.
    if looks_like_expression(&q) {
        if let Ok(value) = meval::eval_str(&q) {
            return Some(format!("{raw} = {}", format_number(value)));
        }
    }

    // Percent must precede generic binary operations.
    if let Some((part, whole)) = capture_pair(&q, r"^([-+]?\d+(?:\.\d+)?)\s*%\s*of\s*([-+]?\d+(?:\.\d+)?)$") {
        return Some(format!("{part}% of {whole} = {}", format_number(part * whole / 100.0)));
    }
    if let Some((part, whole)) = capture_pair(&q, r"^what is\s+([-+]?\d+(?:\.\d+)?)\s+percent\s+of\s+([-+]?\d+(?:\.\d+)?)$") {
        return Some(format!("{part}% of {whole} = {}", format_number(part * whole / 100.0)));
    }

    let operations: [(&str, &str, fn(f64, f64) -> f64); 8] = [
        (r"(?:add|plus|sum of)", "+", |a, b| a + b),
        (r"(?:subtract|minus|take)", "-", |a, b| a - b),
        (r"(?:multiply|times|product of)", "*", |a, b| a * b),
        (r"(?:divide)", "/", |a, b| a / b),
        (r"(?:divided by|over)", "/", |a, b| a / b),
        (r"(?:mod|modulo|remainder of)", "%", |a, b| a % b),
        (r"(?:power of|to the power of|raised to)", "^", |a, b| a.powf(b)),
        (r"(?:average of|mean of)", "avg", |a, b| (a + b) / 2.0),
    ];

    for (words, symbol, calculate) in operations {
        let pattern = format!(
            r"^(?:what is |calculate |compute |evaluate )?([-+]?\d+(?:\.\d+)?)\s+{}\s+([-+]?\d+(?:\.\d+)?)$",
            words
        );
        if let Some((a, b)) = capture_pair(&q, &pattern) {
            if symbol == "/" && b == 0.0 {
                return Some("Division by zero is undefined.".into());
            }
            return Some(format!("{raw} = {}", format_number(calculate(a, b))));
        }

        // Verb-first alternatives: "add 2 and 3", "divide 12 by 4".
        let verb_first = match symbol {
            "+" => r"^(?:add|sum)\s+([-+]?\d+(?:\.\d+)?)\s+(?:and|with|to)\s+([-+]?\d+(?:\.\d+)?)$",
            "-" => r"^(?:subtract)\s+([-+]?\d+(?:\.\d+)?)\s+from\s+([-+]?\d+(?:\.\d+)?)$",
            "*" => r"^(?:multiply)\s+([-+]?\d+(?:\.\d+)?)\s+(?:and|by|with)\s+([-+]?\d+(?:\.\d+)?)$",
            "/" => r"^(?:divide)\s+([-+]?\d+(?:\.\d+)?)\s+by\s+([-+]?\d+(?:\.\d+)?)$",
            _ => continue,
        };
        if let Some((a, b)) = capture_pair(&q, verb_first) {
            // English grammar reverses operands in: subtract A from B.
            let (left, right) = if symbol == "-" { (b, a) } else { (a, b) };
            if symbol == "/" && right == 0.0 {
                return Some("Division by zero is undefined.".into());
            }
            return Some(format!("{raw} = {}", format_number(calculate(left, right))));
        }
    }

    // Aggregate lists: `sum 1, 2, 3`, `add 1 2 3`, `average 2, 4, 6`.
    for (prefix, label, f) in [
        (r"^(?:sum|add)\s+", "sum", sum as fn(&[f64]) -> f64),
        (r"^(?:average|mean)\s+", "average", mean as fn(&[f64]) -> f64),
    ] {
        if let Some(rest) = Regex::new(prefix).ok()?.find(&q).map(|m| &q[m.end()..]) {
            let nums = numbers(rest);
            if nums.len() >= 2 && only_number_separators(rest) {
                return Some(format!("{label} = {}", format_number(f(&nums))));
            }
        }
    }

    None
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .replace(',', " ")
        .replace('?', "")
        .replace("multiplied by", "multiply")
        .replace("divided by", "divide")
        .replace("what is ", "what is ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn capture_pair(input: &str, pattern: &str) -> Option<(f64, f64)> {
    let caps = Regex::new(pattern).ok()?.captures(input)?;
    Some((caps.get(1)?.as_str().parse().ok()?, caps.get(2)?.as_str().parse().ok()?))
}

fn numbers(s: &str) -> Vec<f64> {
    Regex::new(r"[-+]?\d+(?:\.\d+)?")
        .expect("static regex")
        .find_iter(s)
        .filter_map(|m| m.as_str().parse::<f64>().ok())
        .collect()
}

fn only_number_separators(s: &str) -> bool {
    Regex::new(r"^\s*[-+]?\d+(?:\.\d+)?(?:\s*(?:and|,)?\s*[-+]?\d+(?:\.\d+)?)+\s*$")
        .expect("static regex")
        .is_match(s)
}

fn looks_like_expression(s: &str) -> bool {
    s.chars().any(|c| "+-*/^().".contains(c)) && s.chars().any(|c| c.is_ascii_digit())
}

fn sum(xs: &[f64]) -> f64 { xs.iter().sum() }
fn mean(xs: &[f64]) -> f64 { sum(xs) / xs.len() as f64 }

fn format_number(value: f64) -> String {
    if !value.is_finite() { return value.to_string(); }
    if value.fract().abs() < f64::EPSILON { return format!("{value:.0}"); }
    format!("{value:.10}").trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::evaluate_query;

    #[test]
    fn common_words_need_no_llm() {
        assert_eq!(evaluate_query("add 7 and 5").unwrap(), "add 7 and 5 = 12");
        assert_eq!(evaluate_query("sum 1, 2, 3, 4").unwrap(), "sum = 10");
        assert_eq!(evaluate_query("subtract 3 from 10").unwrap(), "subtract 3 from 10 = 7");
        assert_eq!(evaluate_query("multiply 6 by 7").unwrap(), "multiply 6 by 7 = 42");
        assert_eq!(evaluate_query("divide 20 by 4").unwrap(), "divide 20 by 4 = 5");
        assert_eq!(evaluate_query("25 percent of 80").unwrap(), "25% of 80 = 20");
    }

    #[test]
    fn rejects_ambiguous_math() {
        assert!(evaluate_query("solve x plus 4").is_none());
        assert_eq!(evaluate_query("divide 5 by 0").unwrap(), "Division by zero is undefined.");
    }
}
