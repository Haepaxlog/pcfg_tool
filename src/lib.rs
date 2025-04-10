use std::collections::HashMap;

type Nonterminal = String;
type Terminal = String;

#[derive(Hash)]
enum Body {
    Lexical(Terminal),
    NonLexical(Vec<Nonterminal>),
}

#[derive(Hash)]
struct Rule {
    head: Nonterminal,
    body: Body,
}

type Quality = f64;

struct Grammar {
    initial: Nonterminal,
    rules: HashMap<Rule, Quality>,
}
