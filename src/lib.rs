use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

#[derive(Clone, Copy, PartialEq)]
enum Tok {
    Unit(u128),
    Tens(u128),
    Hundred,
    Scale(u128),
    And,
    Neg,
    Point,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Sign {
    Positive,
    Negative,
}

struct Parsed {
    sign: Sign,
    int: u128,
    frac: Option<String>,
}

const HUNDRED: u128 = 100;
const TEEN_START: u128 = 10;
const TENS_START: u128 = 20;
const TEN: u128 = 10;
const DIGIT_RADIX: u32 = 10;

const WORDS: &[(&str, Tok)] = &[
    ("zero", Tok::Unit(0)),
    ("one", Tok::Unit(1)),
    ("two", Tok::Unit(2)),
    ("three", Tok::Unit(3)),
    ("four", Tok::Unit(4)),
    ("five", Tok::Unit(5)),
    ("six", Tok::Unit(6)),
    ("seven", Tok::Unit(7)),
    ("eight", Tok::Unit(8)),
    ("nine", Tok::Unit(9)),
    ("ten", Tok::Unit(10)),
    ("eleven", Tok::Unit(11)),
    ("twelve", Tok::Unit(12)),
    ("thirteen", Tok::Unit(13)),
    ("fourteen", Tok::Unit(14)),
    ("fifteen", Tok::Unit(15)),
    ("sixteen", Tok::Unit(16)),
    ("seventeen", Tok::Unit(17)),
    ("eighteen", Tok::Unit(18)),
    ("nineteen", Tok::Unit(19)),
    ("twenty", Tok::Tens(20)),
    ("thirty", Tok::Tens(30)),
    ("forty", Tok::Tens(40)),
    ("fifty", Tok::Tens(50)),
    ("sixty", Tok::Tens(60)),
    ("seventy", Tok::Tens(70)),
    ("eighty", Tok::Tens(80)),
    ("ninety", Tok::Tens(90)),
    ("hundred", Tok::Hundred),
    ("thousand", Tok::Scale(1_000)),
    ("million", Tok::Scale(1_000_000)),
    ("billion", Tok::Scale(1_000_000_000)),
    ("trillion", Tok::Scale(1_000_000_000_000)),
    ("quadrillion", Tok::Scale(1_000_000_000_000_000)),
    ("quintillion", Tok::Scale(1_000_000_000_000_000_000)),
    ("sextillion", Tok::Scale(1_000_000_000_000_000_000_000)),
    ("septillion", Tok::Scale(1_000_000_000_000_000_000_000_000)),
    (
        "octillion",
        Tok::Scale(1_000_000_000_000_000_000_000_000_000),
    ),
    (
        "nonillion",
        Tok::Scale(1_000_000_000_000_000_000_000_000_000_000),
    ),
    (
        "decillion",
        Tok::Scale(1_000_000_000_000_000_000_000_000_000_000_000),
    ),
    (
        "undecillion",
        Tok::Scale(1_000_000_000_000_000_000_000_000_000_000_000_000),
    ),
    ("and", Tok::And),
    ("minus", Tok::Neg),
    ("negative", Tok::Neg),
    ("point", Tok::Point),
];

fn split(s: &str, out: &mut Vec<Tok>) -> bool {
    if s.is_empty() {
        return true;
    }

    for (word, tok) in WORDS {
        let Some(rest) = s.strip_prefix(word) else {
            continue;
        };

        out.push(*tok);
        if split(rest, out) {
            return true;
        }
        out.pop();
    }

    false
}

fn eval(mut toks: &[Tok]) -> Option<(Sign, u128)> {
    let mut sign = Sign::Positive;
    if let [Tok::Neg, rest @ ..] = toks {
        sign = Sign::Negative;
        toks = rest;
    }

    if toks == [Tok::Unit(0)] {
        return Some((sign, 0));
    }

    let mut total: u128 = 0;
    let mut cur: u128 = 0;
    let mut last_scale = u128::MAX;
    let mut seen = false;

    for tok in toks {
        match *tok {
            Tok::And if seen => {}
            Tok::And | Tok::Neg | Tok::Point | Tok::Unit(0) => return None,
            Tok::Unit(v) => {
                let r = cur % HUNDRED;
                let after_tens = v < TEEN_START && r >= TENS_START && r.is_multiple_of(TEN);
                if r != 0 && !after_tens {
                    return None;
                }
                cur += v;
                seen = true;
            }
            Tok::Tens(v) => {
                if !cur.is_multiple_of(HUNDRED) {
                    return None;
                }
                cur += v;
                seen = true;
            }
            Tok::Hundred => {
                if !(1..TEEN_START).contains(&cur) {
                    return None;
                }
                cur *= HUNDRED;
            }
            Tok::Scale(s) => {
                if cur == 0 || s >= last_scale {
                    return None;
                }
                total = total.checked_add(cur.checked_mul(s)?)?;
                cur = 0;
                last_scale = s;
            }
        }
    }

    if !seen {
        return None;
    }

    Some((sign, total.checked_add(cur)?))
}

fn parse(ident: &str) -> Option<Parsed> {
    let flat = ident.replace('_', "");
    let mut toks = Vec::new();
    if !split(&flat, &mut toks) {
        return None;
    }

    let Some(at) = toks.iter().position(|t| *t == Tok::Point) else {
        let (sign, int) = eval(&toks)?;
        return Some(Parsed {
            sign,
            int,
            frac: None,
        });
    };

    let (sign, int) = eval(&toks[..at])?;
    let digits = &toks[at + 1..];
    if digits.is_empty() {
        return None;
    }

    let mut frac = String::new();
    for tok in digits {
        let Tok::Unit(d @ 0..=9) = *tok else {
            return None;
        };
        frac.push(char::from_digit(d as u32, DIGIT_RADIX)?);
    }

    Some(Parsed {
        sign,
        int,
        frac: Some(frac),
    })
}

fn value_src(p: &Parsed) -> String {
    let sign = if p.sign == Sign::Negative { "-" } else { "" };

    match &p.frac {
        Some(frac) => format!("{sign}{}.{frac}", p.int),
        None => format!("{sign}{}", p.int),
    }
}

fn fn_src(vis: &str, name: &str, p: &Parsed) -> TokenStream {
    format!(
        "{vis} fn {name}<T>() -> T \
         where T: ::core::str::FromStr, <T as ::core::str::FromStr>::Err: ::core::fmt::Debug \
         {{ {:?}.parse().expect(\"number does not fit target type\") }}",
        value_src(p),
    )
    .parse()
    .expect("valid fn")
}

fn error_at(span: Span, msg: &str) -> TokenStream {
    let mut name = Ident::new("compile_error", span);
    name.set_span(span);
    let mut bang = Punct::new('!', Spacing::Alone);
    bang.set_span(span);
    let mut lit = Literal::string(msg);
    lit.set_span(span);
    let mut args = Group::new(Delimiter::Parenthesis, TokenTree::from(lit).into());
    args.set_span(span);
    let mut semi = Punct::new(';', Spacing::Alone);
    semi.set_span(span);

    let trees: [TokenTree; 4] = [name.into(), bang.into(), args.into(), semi.into()];

    trees.into_iter().collect()
}

fn tail(rest: &[TokenTree]) -> Option<Vec<TokenTree>> {
    if let Some(TokenTree::Group(g)) = rest.last() && g.delimiter() == Delimiter::Brace {
        return Some(g.stream().into_iter().collect());
    }

    let colon = rest
        .iter()
        .rposition(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ':'))?;

    Some(rest[colon + 1..].to_vec())
}

fn names(tail: &[TokenTree]) -> Result<Vec<(Ident, String)>, TokenStream> {
    let mut out = Vec::new();
    let is_comma = |t: &TokenTree| matches!(t, TokenTree::Punct(p) if p.as_char() == ',');

    for entry in tail.split(is_comma) {
        let (orig, alias) = match entry {
            [] => continue,
            [TokenTree::Ident(i)] => (i, i.to_string()),
            [TokenTree::Ident(i), TokenTree::Ident(kw), TokenTree::Ident(a)]
                if kw.to_string() == "as" =>
            {
                (i, a.to_string())
            }
            [first, ..] => return Err(error_at(first.span(), "expected a number name")),
        };
        out.push((orig.clone(), alias));
    }

    Ok(out)
}

#[proc_macro_attribute]
pub fn numbers(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trees: Vec<TokenTree> = item.into_iter().collect();
    let is_use = |t: &TokenTree| matches!(t, TokenTree::Ident(i) if i.to_string() == "use");
    let Some(use_at) = trees.iter().position(is_use) else {
        return error_at(Span::call_site(), "#[numbers] goes on a `use` item");
    };

    let vis = trees[..use_at].iter().cloned().collect::<TokenStream>().to_string();
    let rest = match &trees[use_at + 1..] {
        [head @ .., TokenTree::Punct(p)] if p.as_char() == ';' => head,
        all => all,
    };
    let Some(tail) = tail(rest) else {
        return error_at(Span::call_site(), "expected `use rust_num::name;`");
    };
    let names = match names(&tail) {
        Ok(names) => names,
        Err(err) => return err,
    };

    let mut out = TokenStream::new();
    for (orig, alias) in names {
        let Some(parsed) = parse(&orig.to_string()) else {
            return error_at(orig.span(), "not an English number");
        };
        out.extend(fn_src(&vis, &alias, &parsed));
    }

    out
}

fn word(tok: Tok) -> &'static str {
    WORDS
        .iter()
        .find(|(_, t)| *t == tok)
        .map(|(w, _)| *w)
        .expect("every token has a word")
}

fn hundreds(n: u128, out: &mut Vec<&'static str>) {
    let h = n / HUNDRED;
    if h > 0 {
        out.push(word(Tok::Unit(h)));
        out.push(word(Tok::Hundred));
    }

    let r = n % HUNDRED;
    if r < TENS_START {
        if r > 0 {
            out.push(word(Tok::Unit(r)));
        }
        return;
    }

    out.push(word(Tok::Tens(r / TEN * TEN)));
    if r.is_multiple_of(10) {
        out.push(word(Tok::Unit(r % TEN)));
    }
}

fn to_words(mut n: u128) -> Vec<&'static str> {
    if n == 0 {
        return vec![word(Tok::Unit(0))];
    }

    let mut out = Vec::new();
    for (_, tok) in WORDS.iter().rev() {
        let Tok::Scale(s) = *tok else {
            continue;
        };
        if n < s {
            continue;
        }

        hundreds(n / s, &mut out);
        out.push(word(*tok));
        n %= s;
    }
    hundreds(n, &mut out);

    out
}

fn compile_error(msg: &str) -> TokenStream {
    format!("compile_error!({msg:?});")
        .parse()
        .expect("valid compile_error")
}

#[proc_macro]
pub fn words(item: TokenStream) -> TokenStream {
    let flat = item.to_string().replace(['_', ' '], "");
    let (neg, text) = match flat.strip_prefix('-') {
        Some(t) => (true, t),
        None => (false, flat.as_str()),
    };
    let (int, frac) = match text.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (text, None),
    };
    let Ok(n) = int.parse::<u128>() else {
        return compile_error("words! expects an integer or decimal literal");
    };

    let mut out = Vec::new();
    if neg {
        out.push(word(Tok::Neg));
    }
    out.extend(to_words(n));

    if let Some(frac) = frac {
        out.push(word(Tok::Point));
        for c in frac.chars() {
            let Some(d) = c.to_digit(DIGIT_RADIX) else {
                return compile_error("words! expects an integer or decimal literal");
            };
            out.push(word(Tok::Unit(d.into())));
        }
    }

    TokenTree::from(Literal::string(&out.join(" "))).into()
}
