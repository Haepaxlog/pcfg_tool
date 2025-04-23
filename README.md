# PCFG Tool
PCFG = [Probabilistic Context-Free Grammar](https://en.wikipedia.org/wiki/Probabilistic_context-free_grammar) \
PTB = Penn Treebank



This repo contains a binary with tools for PCFG-based parsing of natural language sentences.

## Usage

### pcfg_tool induce
  Induces a PCFG from an stdio stream given in the PTB format. The probabilities are calculated via relative frequency estimation. 
  If `-g {name}`is specified, the output will be printed as follows:
  * {name}.rules (non-lexical rules)
  * {name}.lexicon (lexical rules)
  * {name}.words (terminals)

## Building
```sh
make
```
