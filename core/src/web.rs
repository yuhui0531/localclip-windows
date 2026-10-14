use std::collections::BTreeSet;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

static TLDS: LazyLock<BTreeSet<&'static str>> = LazyLock::new(|| {
    include_str!("../data/tlds.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect()
});

pub fn web_link(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty()
        || text.len() > 8192
        || text.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return None;
    }
    let (prefix, tail, explicit) = match text.split_once("://") {
        Some((scheme, tail))
            if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") =>
        {
            (format!("{scheme}://"), tail, true)
        }
        Some(_) => return None,
        None => ("http://".into(), text, false),
    };
    let boundary = tail.find(['/', '?', '#']).unwrap_or(tail.len());
    let authority = &tail[..boundary];
    let (userinfo, host_port) = match authority.rsplit_once('@') {
        Some((user, host)) if explicit => (format!("{user}@"), host),
        Some(_) => return None,
        None => (String::new(), authority),
    };
    let (host, port) = if host_port.starts_with('[') {
        let end = host_port.find(']')?;
        if !explicit || host_port[1..end].parse::<std::net::Ipv6Addr>().is_err() {
            return None;
        }
        (&host_port[..=end], &host_port[end + 1..])
    } else {
        host_port
            .split_once(':')
            .map_or((host_port, ""), |(host, _)| {
                (host, &host_port[host.len()..])
            })
    };
    if host.is_empty() || (!port.is_empty() && (port.strip_prefix(':')?.parse::<u16>().is_err())) {
        return None;
    }
    let encoded_host = if host.starts_with('[') {
        host.to_string()
    } else {
        let mut labels = Vec::new();
        for label in host.split('.') {
            if label.is_empty()
                || label.starts_with('-')
                || label.ends_with('-')
                || label.chars().any(|c| {
                    !c.is_alphanumeric()
                        && c != '-'
                        && !unicode_normalization::char::is_combining_mark(c)
                })
            {
                return None;
            }
            let encoded = if label.is_ascii() {
                label.to_string()
            } else {
                format!(
                    "xn--{}",
                    punycode(&label.to_lowercase().nfc().collect::<String>())?
                )
            };
            if encoded.len() > 63 {
                return None;
            }
            labels.push(encoded);
        }
        if !explicit {
            let tld = labels.last()?.to_ascii_uppercase();
            // Product-approved macOS detector compatibility; see data/README.md.
            if labels.len() < 2
                || !host.is_ascii()
                || matches!(tld.as_str(), "AI" | "APP" | "DEV" | "MD")
                || !TLDS.contains(tld.as_str())
            {
                return None;
            }
        }
        labels.join(".")
    };
    Some(format!(
        "{prefix}{}{encoded_host}{port}{}",
        encode_non_ascii(&userinfo),
        encode_non_ascii(&tail[boundary..])
    ))
}

fn encode_non_ascii(value: &str) -> String {
    let mut result = String::new();
    for byte in value.bytes() {
        if byte.is_ascii()
            && !matches!(
                byte,
                b'"' | b'<' | b'>' | b'`' | b'{' | b'}' | b'\\' | b'^' | b'|'
            )
        {
            result.push(byte as char);
        } else {
            use std::fmt::Write;
            let _ = write!(result, "%{byte:02X}");
        }
    }
    result
}

// RFC 3492 section 6.3, with checked arithmetic for untrusted clipboard input.
fn punycode(value: &str) -> Option<String> {
    let points: Vec<u32> = value.chars().map(u32::from).collect();
    let mut output: String = value.chars().filter(char::is_ascii).collect();
    let basic = output.len() as u32;
    let (mut handled, mut n, mut delta, mut bias) = (basic, 128u32, 0u32, 72u32);
    if basic > 0 {
        output.push('-');
    }
    while handled < points.len() as u32 {
        let next = *points.iter().filter(|&&p| p >= n).min()?;
        delta = delta.checked_add((next - n).checked_mul(handled + 1)?)?;
        n = next;
        for &point in &points {
            if point < n {
                delta = delta.checked_add(1)?;
            }
            if point == n {
                let (mut q, mut k) = (delta, 36u32);
                loop {
                    let threshold = k.saturating_sub(bias).clamp(1, 26);
                    if q < threshold {
                        break;
                    }
                    output.push(digit(threshold + (q - threshold) % (36 - threshold)));
                    q = (q - threshold) / (36 - threshold);
                    k += 36;
                }
                output.push(digit(q));
                let mut adapted = if handled == basic {
                    delta / 700
                } else {
                    delta / 2
                };
                adapted += adapted / (handled + 1);
                let mut k = 0;
                while adapted > 455 {
                    adapted /= 35;
                    k += 36;
                }
                bias = k + 36 * adapted / (adapted + 38);
                delta = 0;
                handled += 1;
            }
        }
        delta = delta.checked_add(1)?;
        n = n.checked_add(1)?;
    }
    Some(output)
}
fn digit(value: u32) -> char {
    if value < 26 {
        (b'a' + value as u8) as char
    } else {
        (b'0' + (value - 26) as u8) as char
    }
}
