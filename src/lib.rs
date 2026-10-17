use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ItemUse, UseTree, parse_macro_input};

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

struct Tokens<'a> {
    rest: &'a str,
}

impl Iterator for Tokens<'_> {
    type Item = Option<Tok>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }

        let found = WORDS
            .iter()
            .filter(|(word, _)| self.rest.starts_with(word))
            .max_by_key(|(word, _)| word.len());
        let Some((word, tok)) = found else {
            self.rest = "";
            return Some(None);
        };

        self.rest = &self.rest[word.len()..];

        Some(Some(*tok))
    }
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
    let toks = Tokens { rest: &flat }.collect::<Option<Vec<Tok>>>()?;

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
        frac.push(char::from_digit(u32::try_from(d).ok()?, DIGIT_RADIX)?);
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

fn collect(tree: &UseTree, out: &mut Vec<(Ident, Ident)>) -> syn::Result<()> {
    match tree {
        UseTree::Path(p) => collect(&p.tree, out),
        UseTree::Group(g) => g.items.iter().try_for_each(|t| collect(t, out)),
        UseTree::Name(n) => {
            out.push((n.ident.clone(), n.ident.clone()));
            Ok(())
        }
        UseTree::Rename(r) => {
            out.push((r.ident.clone(), r.rename.clone()));
            Ok(())
        }
        UseTree::Glob(g) => Err(syn::Error::new_spanned(g, "globs are not supported")),
    }
}

fn expand(item: &ItemUse) -> syn::Result<TokenStream2> {
    let mut names = Vec::new();
    collect(&item.tree, &mut names)?;

    let attrs = &item.attrs;
    let vis = &item.vis;
    let mut out = TokenStream2::new();

    for (orig, alias) in names {
        let Some(parsed) = parse(&orig.to_string()) else {
            return Err(syn::Error::new(orig.span(), "not an English number"));
        };
        let value = value_src(&parsed);

        out.extend(quote! {
            #(#attrs)*
            #vis fn #alias<T>() -> T
            where
                T: ::core::str::FromStr,
                <T as ::core::str::FromStr>::Err: ::core::fmt::Debug,
            {
                #value.parse().expect("number does not fit target type")
            }
        });
    }

    Ok(out)
}

#[proc_macro_attribute]
pub fn numbers(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemUse);

    expand(&item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
