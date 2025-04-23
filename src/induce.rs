use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    ptb::{Descendants, ParseTree},
    Body, Grammar, Nonterminal, OccurenceRules, Probability, ProbabilityRules, Rule, Terminal,
};

pub trait PCFGGrammar {
    /// Given an initial and parse trees it reuturns a normalised grammar
    fn from_parse_trees(
        initial: Nonterminal,
        parse_trees: Vec<ParseTree<String>>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn normalise(&mut self);

    fn nonlexical_rules(&self) -> ProbabilityRules;

    fn lexical_rules(&self) -> ProbabilityRules;

    fn nonterminals(&self) -> Vec<Nonterminal>;

    fn terminals(&self) -> Vec<Terminal>;
}

trait PTBRuleInducer {
    /// Normalises a given ruleset with occurences into a ruleset with probabilites
    fn normalise_rules(occurence_rules: OccurenceRules) -> ProbabilityRules;

    /// Accumulates rules into occurence_rules thereby counting their occurence
    fn count_rule_occurence(occurence_rules: &mut OccurenceRules, rules: Vec<Rule>);

    /// Traverses the parse tree breadth-first until we have read all rules starting at the subtree given by initial_subtree()
    fn read_rules(
        initial: &str,
        parse_tree: ParseTree<String>,
        inital_subtree: fn(&str, ParseTree<String>) -> Option<ParseTree<String>>,
    ) -> Option<Vec<Rule>>;
}

impl PCFGGrammar for Grammar {
    fn from_parse_trees(
        initial: Nonterminal,
        parse_trees: Vec<ParseTree<String>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut occurence_rules: OccurenceRules = HashMap::new();

        for parse_tree in parse_trees {
            let tree_rules = Self::read_rules(
                &initial,
                parse_tree,
                // Assume starting at the first tree root for now
                |_initial: &str, parse_tree: ParseTree<String>| -> Option<ParseTree<String>> {
                    Some(parse_tree)
                },
            );

            match tree_rules {
                Some(rules) => {
                    Self::count_rule_occurence(&mut occurence_rules, rules);
                }
                None => {
                    return Err(("There are no rules to read").into());
                }
            }
        }

        let probability_rules = Self::normalise_rules(occurence_rules);

        Ok(Grammar {
            initial,
            rules: probability_rules,
        })
    }

    fn normalise(&mut self) {
        let mut occurence_rules = HashMap::new();

        Self::count_rule_occurence(
            &mut occurence_rules,
            self.rules.clone().into_keys().collect(),
        );

        self.rules = Self::normalise_rules(occurence_rules);
    }

    fn nonterminals(&self) -> Vec<Nonterminal> {
        let mut nonterminals: HashSet<Nonterminal> = HashSet::new();
        // TODO: Should this be written into the nonterminals ??
        // The grammar's intitial must be by defintion a Nonterminal
        nonterminals.insert((*self.initial).to_string());

        for rule in self.rules.keys() {
            // The head must always be a nonterminal
            nonterminals.insert(rule.head.clone());

            // We need to this here, because not every nonterminal will have a rule, where it is a head
            match &rule.body {
                Body::Lexical(_) => {}
                Body::NonLexical(nts) => nts.iter().for_each(|nonterminal| {
                    nonterminals.insert(nonterminal.to_owned());
                }),
            }
        }

        nonterminals.into_iter().collect()
    }

    fn terminals(&self) -> Vec<Terminal> {
        let mut terminals: HashSet<Terminal> = HashSet::new();

        for rule in self.rules.keys() {
            match &rule.body {
                Body::Lexical(terminal) => {
                    terminals.insert(terminal.to_owned());
                }
                Body::NonLexical(_) => {}
            }
        }

        terminals.into_iter().collect()
    }

    fn nonlexical_rules(&self) -> ProbabilityRules {
        let mut rules = self.rules.clone();

        for (rule, _probability) in self.rules.iter() {
            if rule.is_lexical_rule() {
                rules.remove(rule);
            }
        }

        rules
    }

    fn lexical_rules(&self) -> ProbabilityRules {
        let mut rules = self.rules.clone();

        for (rule, _probability) in self.rules.iter() {
            if !rule.is_lexical_rule() {
                rules.remove(rule);
            }
        }

        rules
    }
}

impl PTBRuleInducer for Grammar {
    fn normalise_rules(occurence_rules: OccurenceRules) -> ProbabilityRules {
        // Sort rules for their head (e.g. NP -> DT NN has head NP)
        let sorted_rules =
            occurence_rules
                .into_iter()
                .fold(HashMap::new(), |mut acc, (rule, occurence)| {
                    let entry = acc
                        .entry(rule.head.to_string())
                        .or_insert(Vec::<(Rule, u32)>::new());
                    entry.push((rule, occurence));
                    acc
                });

        // Calculate probabilites on the sorted rules
        // rule_probability = rule_occurence / sum(rule_occurence_with_same_head)
        sorted_rules
            .into_iter()
            .fold(HashMap::new(), |mut acc, (_head, occurence_rules)| {
                let total_head_occurences: Probability = occurence_rules
                    .iter()
                    .map(|(_head, occurence)| *occurence as f64)
                    .sum();

                occurence_rules.into_iter().for_each(|(rule, occurence)| {
                    acc.insert(
                        rule,
                        occurence as Probability / total_head_occurences as Probability,
                    );
                });
                acc
            })
    }

    fn count_rule_occurence(occurence_rules: &mut OccurenceRules, rules: Vec<Rule>) {
        rules.into_iter().for_each(|rule| {
            *occurence_rules.entry(rule).or_insert(0) += 1;
        })
    }

    fn read_rules(
        initial: &str,
        parse_tree: ParseTree<String>,
        inital_subtree: fn(&str, ParseTree<String>) -> Option<ParseTree<String>>,
    ) -> Option<Vec<Rule>> {
        let subtree = inital_subtree(initial, parse_tree)?;

        let mut rules = Vec::<Rule>::new();
        let mut queue: VecDeque<&ParseTree<String>> = VecDeque::new();

        queue.push_front(&subtree);

        while let Some(tree) = queue.pop_front() {
            rules.push(Rule {
                head: tree.root.clone(),
                body: match &tree.descendants {
                    Descendants::Atom(atom) => Body::Lexical(atom.to_string()),
                    Descendants::Expressions(parse_trees) => {
                        parse_trees.iter().for_each(|tree| queue.push_front(&tree));

                        Body::NonLexical(
                            parse_trees
                                .into_iter()
                                .map(|tree| tree.root.clone())
                                .collect(),
                        )
                    }
                },
            })
        }
        Some(rules)
    }
}

// NOTE: Currently unused
/// Traverses the tree breadth-first until we hit the first subtree root that matches the intial
fn first_matching_subtree(
    initial: &str,
    parse_tree: ParseTree<String>,
) -> Option<ParseTree<String>> {
    let mut search_queue: VecDeque<&ParseTree<String>> = VecDeque::new();
    search_queue.push_front(&parse_tree);

    let subtree: Option<ParseTree<String>> = loop {
        if search_queue.is_empty() {
            break None;
        }

        let tree = search_queue
            .pop_front()
            .expect("queue was checked to be not empty");

        if tree.root == initial {
            break Some(tree.clone());
        }

        match &tree.descendants {
            Descendants::Atom(_) => {}
            Descendants::Expressions(parse_trees) => parse_trees
                .iter()
                .for_each(|tree| search_queue.push_front(&tree)),
        }
    };

    subtree
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::ptb::PTBParser;

    use super::*;
    #[test]
    fn reads_off_rules_correctly() {
        let input = "(S (NP (NNP Julius)) (VP (VB stabs) (NP (NN him))))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_tree = output;

        let rules = Grammar::read_rules(
            &initial,
            parse_tree,
            |_initial: &str, parse_tree: ParseTree<String>| Some(parse_tree),
        );

        assert!(rules.is_some());
        assert_eq!(
            HashSet::from_iter(rules.expect("This is Some").into_iter()) as HashSet<Rule>,
            HashSet::from_iter(vec![
                Rule {
                    head: "S".to_string(),
                    body: Body::NonLexical(vec!["NP".to_string(), "VP".to_string(),])
                },
                Rule {
                    head: "NP".to_string(),
                    body: Body::NonLexical(vec!["NNP".to_string()])
                },
                Rule {
                    head: "NNP".to_string(),
                    body: Body::Lexical("Julius".to_string())
                },
                Rule {
                    head: "VP".to_string(),
                    body: Body::NonLexical(vec!["VB".to_string(), "NP".to_string()])
                },
                Rule {
                    head: "VB".to_string(),
                    body: Body::Lexical("stabs".to_string())
                },
                Rule {
                    head: "NP".to_string(),
                    body: Body::NonLexical(vec!["NN".to_string()])
                },
                Rule {
                    head: "NN".to_string(),
                    body: Body::Lexical("him".to_string())
                }
            ])
        );
    }

    #[test]
    fn counts_correct_rule_occurence() {
        let input = "(S (NP (VP some)) (NP (VP some)) (NP (VP other)))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_tree = output;

        let rules = Grammar::read_rules(
            &initial,
            parse_tree,
            |_initial: &str, parse_tree: ParseTree<String>| Some(parse_tree),
        );

        assert!(rules.is_some());

        let mut occurence_rules: OccurenceRules = HashMap::new();
        Grammar::count_rule_occurence(&mut occurence_rules, rules.expect("This is some"));

        assert_eq!(
            occurence_rules,
            HashMap::from_iter(vec![
                (
                    Rule {
                        head: "S".to_string(),
                        body: Body::NonLexical(vec![
                            "NP".to_string(),
                            "NP".to_string(),
                            "NP".to_string()
                        ])
                    },
                    1
                ),
                (
                    Rule {
                        head: "NP".to_string(),
                        body: Body::NonLexical(vec!["VP".to_string()])
                    },
                    3
                ),
                (
                    Rule {
                        head: "VP".to_string(),
                        body: Body::Lexical("some".to_string())
                    },
                    2
                ),
                (
                    Rule {
                        head: "VP".to_string(),
                        body: Body::Lexical("other".to_string())
                    },
                    1
                )
            ])
        )
    }

    #[test]
    fn normalises_rules_correctly() {
        let input = "(S (NP (VP some)) (NP (VP some)) (NP (VP other)))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_tree = output;

        let rules = Grammar::read_rules(
            &initial,
            parse_tree,
            |_initial: &str, parse_tree: ParseTree<String>| Some(parse_tree),
        );

        assert!(rules.is_some());

        let mut occurence_rules: OccurenceRules = HashMap::new();
        Grammar::count_rule_occurence(&mut occurence_rules, rules.expect("This is some"));

        let normalised_rules = Grammar::normalise_rules(occurence_rules);
        assert_eq!(
            normalised_rules,
            HashMap::from_iter(vec![
                (
                    Rule {
                        head: "S".to_string(),
                        body: Body::NonLexical(vec![
                            "NP".to_string(),
                            "NP".to_string(),
                            "NP".to_string()
                        ])
                    },
                    1.0 as f64
                ),
                (
                    Rule {
                        head: "NP".to_string(),
                        body: Body::NonLexical(vec!["VP".to_string()])
                    },
                    1.0 as f64
                ),
                (
                    Rule {
                        head: "VP".to_string(),
                        body: Body::Lexical("some".to_string())
                    },
                    2.0 / 3.0 as f64
                ),
                (
                    Rule {
                        head: "VP".to_string(),
                        body: Body::Lexical("other".to_string())
                    },
                    1.0 / 3.0 as f64
                )
            ])
        )
    }

    #[test]
    fn instantiates_correct_grammar() {
        let input = "(S (NP (NNP Julius)) (VP (VB stabs) (NP (NN him))))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_trees = vec![output];

        let grammar = Grammar::from_parse_trees(initial as Nonterminal, parse_trees)
            .expect("This is a valid initial");

        assert_eq!(
            grammar,
            Grammar {
                initial: "S".to_string(),
                rules: HashMap::from_iter(vec![
                    (
                        Rule {
                            head: "S".to_string(),
                            body: Body::NonLexical(vec!["NP".to_string(), "VP".to_string(),])
                        },
                        1.0
                    ),
                    (
                        Rule {
                            head: "NP".to_string(),
                            body: Body::NonLexical(vec!["NNP".to_string()])
                        },
                        0.5
                    ),
                    (
                        Rule {
                            head: "NNP".to_string(),
                            body: Body::Lexical("Julius".to_string())
                        },
                        1.0
                    ),
                    (
                        Rule {
                            head: "VP".to_string(),
                            body: Body::NonLexical(vec!["VB".to_string(), "NP".to_string()])
                        },
                        1.0
                    ),
                    (
                        Rule {
                            head: "VB".to_string(),
                            body: Body::Lexical("stabs".to_string())
                        },
                        1.0
                    ),
                    (
                        Rule {
                            head: "NP".to_string(),
                            body: Body::NonLexical(vec!["NN".to_string()])
                        },
                        0.5
                    ),
                    (
                        Rule {
                            head: "NN".to_string(),
                            body: Body::Lexical("him".to_string())
                        },
                        1.0
                    )
                ])
            }
        )
    }

    #[test]
    fn correct_nonterminals() {
        let input = "(S (NP (NNP Julius)) (VP (VB stabs) (NP (NN him))))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_trees = vec![output];

        let grammar = Grammar::from_parse_trees(initial as Nonterminal, parse_trees)
            .expect("This is a valid initial");

        let nonterminals = grammar.nonterminals();
        assert_eq!(
            HashSet::from_iter(nonterminals) as HashSet<Nonterminal>,
            HashSet::from_iter(vec![
                "S".to_string(),
                "NP".to_string(),
                "NNP".to_string(),
                "VP".to_string(),
                "VB".to_string(),
                "NN".to_string()
            ])
        )
    }

    #[test]
    fn correct_terminals() {
        let input = "(S (NP (NNP Julius)) (VP (VB stabs) (NP (NN him))))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_trees = vec![output];

        let grammar = Grammar::from_parse_trees(initial as Nonterminal, parse_trees)
            .expect("This is a valid initial");

        let nonterminals = grammar.terminals();
        assert_eq!(
            HashSet::from_iter(nonterminals) as HashSet<Terminal>,
            HashSet::from_iter(vec![
                "Julius".to_string(),
                "stabs".to_string(),
                "him".to_string()
            ])
        )
    }

    #[test]
    fn probabilities_add_to_one() {
        let input = vec![
            "(ROOT (S (NP-SBJ (NP (NNP Pierre) (NNP Vinken)) (, ,) (ADJP (NP (CD 61) (NNS years)) (JJ old)) (, ,)) (VP (MD will) (VP (VB join) (NP (DT the) (NN board)) (PP-CLR (IN as) (NP (DT a) (JJ nonexecutive) (NN director))) (NP-TMP (NNP Nov.) (CD 29)))) (. .)))",
            "(ROOT (S (NP-SBJ (NNP Mr.) (NNP Vinken)) (VP (VBZ is) (NP-PRD (NP (NN chairman)) (PP (IN of) (NP (NP (NNP Elsevier) (NNP N.V.)) (, ,) (NP (DT the) (NNP Dutch) (VBG publishing) (NN group)))))) (. .)))",
            "(ROOT (S (NP-SBJ (NP (NNP Rudolph) (NNP Agnew)) (, ,) (UCP (ADJP (NP (CD 55) (NNS years)) (JJ old)) (CC and) (NP (NP (JJ former) (NN chairman)) (PP (IN of) (NP (NNP Consolidated) (NNP Gold) (NNP Fields) (NNP PLC))))) (, ,)) (VP (VBD was) (VP (VBN named) (NP (NP (DT a) (JJ nonexecutive) (NN director)) (PP (IN of) (NP (DT this) (JJ British) (JJ industrial) (NN conglomerate)))))) (. .)))",
            "(ROOT (S (S-TPC (NP-SBJ (NP (NP (DT A) (NN form)) (PP (IN of) (NP (NN asbestos)))) (RRC (ADVP-TMP (RB once)) (VP (VBN used) (S-CLR (VP (TO to) (VP (VB make) (NP (NNP Kent) (NN cigarette) (NNS filters)))))))) (VP (VBZ has) (VP (VBN caused) (S (NP-SBJ (NP (DT a) (JJ high) (NN percentage)) (PP (IN of) (NP (NN cancer) (NNS deaths))) (PP-LOC (IN among) (NP (NP (DT a) (NN group)) (PP (IN of) (NP (NNS workers)))))) (VP (VBN exposed) (PP-CLR (TO to) (NP (PRP it))) (ADVP-TMP (NP (QP (RBR more) (IN than) (CD 30)) (NNS years)) (IN ago))))))) (, ,) (NP-SBJ (NNS researchers)) (VP (VBD reported)) (. .)))",
            "(ROOT (S (S-TPC (NP-SBJ (NP (DT The) (NN asbestos) (NN fiber)) (, ,) (NP (NN crocidolite)) (, ,)) (VP (VBZ is) (ADJP-PRD (RB unusually) (JJ resilient)) (SBAR-TMP (IN once) (S (NP-SBJ (PRP it)) (VP (VBZ enters) (NP (DT the) (NNS lungs))))) (, ,) (PP (IN with) (NP (NP (NP (RB even) (JJ brief) (NNS exposures)) (PP-DIR (TO to) (NP (PRP it)))) (PP (VBG causing) (NP (NNS symptoms))) (SBAR (WHNP (WDT that)) (S (VP (VBP show) (PRT (RP up)) (ADVP-TMP (NP (NNS decades)) (JJ later))))))))) (, ,) (NP-SBJ (NNS researchers)) (VP (VBD said)) (. .)))",
        ];
        let parse_trees: Vec<ParseTree<String>> = input
            .into_iter()
            .map(|input| PTBParser::parse(input).expect("This should be parsable"))
            .collect();

        let initial = String::from("ROOT");

        let grammar = Grammar::from_parse_trees(initial as Nonterminal, parse_trees)
            .expect("This is a valid initial");

        let mut sum_head = HashMap::new();
        for (rule, probability) in grammar.rules.iter() {
            *sum_head.entry(rule.head.clone()).or_insert(0.0) += probability;
        }

        // We should compare against the machine epsilon to assert equality, since there are rounding errors associated with IEEE 754
        let epsilon = f64::EPSILON;
        let approx_equal = |a: f64, b: f64| -> bool { (a - b).abs() <= epsilon };

        for (_head, total) in sum_head.iter() {
            assert!(approx_equal(*total, 1.0));
        }
    }
}
