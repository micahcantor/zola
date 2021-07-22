use std::str;

use katex;
use regex::Regex;

pub fn render_katex(content: &str) -> String {
    let inline_math_re = Regex::new(
            r"(?<![\\\$])\$ # non-escaped opening dollar and non-double-dollar
            (
              [^\s\$] # immediately followed by a non-whitespace character
              [^\$]*
              (?<![\\\s\$]) # closing dollar is immediately preceeded by a non-whitespace,
                            # non-backslash character
            )
            \$(?![\d\$]) # closing dollar is not immediately followed by a digit or another dollar"
        ).unwrap();
    let display_math_re = Regex::new(
            r"(?<!\\)\$\$ # opening double-dollar not preceeded by a backslash
            (?=[^\s]|\h*\n\h*[^\$\s]) # either no whitespace, or a single newline
                                      # followed by a non-empty line
            ([^\$]*[^\s\$]) # any amount of characters not ending in whitespace
            (?:\h*\n\h*)? # a possibly empty line before closing dollars
            \$\$"
        ).unwrap();


    let inline = render_katex_aux(content, inline_math_re, false);
    render_katex_aux(&inline.to_owned(), display_math_re, true)
}

fn render_katex_aux(content: &str, rex: Regex, display: bool) -> String {
    let k_opts = katex::Opts::builder().display_mode(display).build().unwrap();
    let mut last: usize = 0;
    let mut with_katex = String::with_capacity(content.len());
    for caps in rex.captures_iter(content) {
      let replace = caps.get(0).unwrap();
      let tex = caps.get(1).unwrap();
      with_katex.push_str(&content[last..replace.start()]);
      last = replace.end();
      let s = &content[tex.start()..tex.end()];
      let k_html = katex::render_with_opts(s, k_opts.clone()).unwrap();
      with_katex.push_str(&k_html);
      // println!("{:?}", k_html);
    }
    with_katex.push_str(&content[last..]);
    with_katex
}


#[cfg(test)]
mod tests {
    use super::*;

    fn unchanged(eg: &str) {
        assert_eq!(eg, render_katex(eg));
    }

    fn changed(eg: &str) {
        let result = render_katex(eg);
        assert!(result.len() > eg.len());
        assert_ne!(eg, &result[..eg.len()]);
    }

    #[test]
    fn no_math_unchanged() {
        unchanged("This is just a sentence.");
    }

    #[test]
    fn price_not_math_unchanged() {
        unchanged("This has a number that is not math $3 000");
    }

    #[test]
    fn two_consecutive_prices_unchanged() {
        unchanged("Here are two consecutive prices with no whitspace $50$60");
        unchanged("Here are two consecutive prices with whitspace $50 $60");
        unchanged("Here are two consecutive prices with comma $50,$60");
    }

    #[test]
    fn backslash_proceeding_dollar_unchanged() {
        unchanged(r"\$F = ma$");
        unchanged(r"$F = ma\$");
    }

    #[test]
    fn double_dollar_unchanged() {
        unchanged(r"$$F = ma$");
        unchanged(r"$F = ma$$");
    }

    #[test]
    fn internal_whitespace_padding_unchanged() {
        unchanged(r"$ F = ma$");
        unchanged(r"$F = ma $");
        unchanged(r"$$ \int_0^1 x^2 = \frac{1}{2}$$");
        unchanged(r"$$\int_0^1 x^2 = \frac{1}{2} $$");
        unchanged(
r"$$
\int_0^1 x^2 = \frac{1}{2}
$$"
        );
        unchanged(
r"$$
\int_0^1 x^2 = \frac{1}{2}
$$"
        );
    }

    #[test]
    fn bad_internal_dollar_unchanged() {
        unchanged(r"$$\int_0^1 x^2 = \frac{1}${2}$$");
    }

    #[test]
    fn double_dollar_escaped_unchanged() {
        unchanged(r"\$$\int_0^1 x^2 = \frac{1}{2}$$");
        unchanged(r"$\$\int_0^1 x^2 = \frac{1}{2}$$");
        unchanged(r"$$\int_0^1 x^2 = \frac{1}${2}\$$");
        unchanged(r"$$\int_0^1 x^2 = \frac{1}{2}$\$");
    }

    #[test]
    fn random_double_dollar_unchanged() {
        unchanged(r"Hey $$ planet");
    }

    #[test]
    fn working_inline() {
        let eg = r"Consider $π = \frac{1}{2}τ$ for a moment.";
        let result = render_katex(eg);
        assert!(result.len() > eg.len());
        assert_ne!(eg, result);
        assert_eq!(eg[..9], result[..9]);
        assert_eq!(eg[eg.len()-14..], result[result.len()-14..]);
    }

    #[test]
    fn working_multiline() {
        changed(
r"$$\sum_{i = 0}^n i = \frac{1}{2}n(n+1)$$"
        );
        // N.B. trailing whitespace is deliberate and should not disable math mode.
        changed(
r"    $$ 
        \sum_{i = 0}^n i = \frac{1}{2}n(n+1) 
    $$"
        );
    }

    #[test]
    fn multiple_formulae() {
        let eg = r"Consider $π = \frac{1}{2}τ$, then
            $$
                4 \int_{-1}^1 \sqrt{1 - x^2} \mathop{dx} = τ
            $$
            and also consider $A = πr^2$ for a moment.";
        let result = render_katex(eg);
        assert!(result.len() > eg.len());
        assert!(result.contains(", then"));
        assert!(result.contains("and also consider "));
        assert_ne!(eg, result);
        assert_eq!(eg[..9], result[..9]);
        assert_eq!(eg[eg.len()-14..], result[result.len()-14..]);
    }
}