use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    ptb::{Descendants, ParseTree},
    Body, Grammar, Nonterminal, Probability, ProbabilityRules, Rule, Terminal,
};

type OccurenceRules = HashMap<Rule, u32>;

trait PCFGGrammar {
    /// Given an initial and parse trees it reutrns a normalised grammar
    fn from_parse_trees(
        initial: Nonterminal,
        parse_trees: Vec<ParseTree<String>>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn normalise(&mut self);

    /// Normalises a given ruleset with occurences into a ruleset with probabilites
    fn normalise_rules(occurence_rules: OccurenceRules) -> ProbabilityRules;

    fn nonterminals(self) -> Vec<Nonterminal>;

    fn terminals(self) -> Vec<Terminal>;

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

    fn normalise_rules(occurence_rules: OccurenceRules) -> ProbabilityRules {
        // Sort rules for their head (e.g. NP -> DT NN has head NP)
        let sorted_rules =
            occurence_rules
                .into_iter()
                .fold(HashMap::new(), |mut acc, (rule, occurence)| {
                    let entry = acc
                        .entry(rule.head.clone())
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

    fn nonterminals(self) -> Vec<Nonterminal> {
        let mut nonterminals: HashSet<Nonterminal> = HashSet::new();
        // The grammar's intitial must be by defintion a Nonterminal
        nonterminals.insert(self.initial);

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

    fn terminals(self) -> Vec<Terminal> {
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
        let subtree = inital_subtree(initial, parse_tree);

        if subtree.is_none() {
            return None;
        }

        let mut rules = Vec::<Rule>::new();
        let mut queue = VecDeque::new();

        queue.push_front(subtree.expect("this must be a value"));

        while !queue.is_empty() {
            let tree = queue
                .pop_front()
                .expect("queue was checked to not be empty");

            rules.push(Rule {
                head: tree.root,
                body: match tree.descendants {
                    Descendants::Atom(atom) => Body::Lexical(atom),
                    Descendants::Expressions(parse_trees) => {
                        parse_trees
                            .iter()
                            .for_each(|tree| queue.push_front(tree.clone()));

                        Body::NonLexical(parse_trees.into_iter().map(|tree| tree.root).collect())
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
    let mut search_queue = VecDeque::new();
    search_queue.push_front(parse_tree);

    let subtree: Option<ParseTree<String>> = loop {
        if search_queue.is_empty() {
            break None;
        }

        let tree = search_queue
            .pop_front()
            .expect("queue was checked to be not empty");

        if tree.root == initial {
            break Some(tree);
        }

        match tree.descendants {
            Descendants::Atom(_) => {}
            Descendants::Expressions(parse_trees) => parse_trees
                .iter()
                .for_each(|tree| search_queue.push_front(tree.clone())),
        }
    };

    subtree
}

#[cfg(test)]
mod tests {
    use crate::ptb::PTBParser;

    use super::*;

    #[test]
    fn read_off_rules() {
        let input = "(ROOT (S (NP-SBJ (NP (NNP Pierre) (NNP Vinken)) (, ,) (ADJP (NP (CD 61) (NNS years)) (JJ old)) (, ,)) (VP (MD will) (VP (VB join) (NP (DT the) (NN board)) (PP-CLR (IN as) (NP (DT a) (JJ nonexecutive) (NN director))) (NP-TMP (NNP Nov.) (CD 29)))) (. .)))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("ROOT");
        let parse_trees = vec![output];

        let grammar = Grammar::from_parse_trees(initial as Nonterminal, parse_trees)
            .expect("This should have a valid initial");

        println!("{}", grammar);
    }

    #[test]
    fn read_off_rules_simple() {
        let input = "(S (NP (NNP Julius)) (VP (VB stabs) (NP (NN him))))";
        let output = PTBParser::parse(input).expect("This should be parsable");

        let initial = String::from("S");
        let parse_trees = vec![output];

        let grammar = Grammar::from_parse_trees(initial as Nonterminal, parse_trees)
            .expect("This is a valid initial");

        println!("{}", grammar);
    }
}
