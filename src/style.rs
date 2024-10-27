// Stitches css node and html node to generate a style node that will make up the style tree
// Single node in the dom tree has a single node in the style tree (not to be confused with css tree)

use std::collections::HashMap;

use crate::{
    css::{Rule, Selector, SimpleSelector, Specificity, StylesSheet, Value},
    dom::{ElementData, Node},
};

// Map css property names to values
type PropertyMap = HashMap<String, Value>;

// Node associated with style data
struct StyledNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}

/// single CSS rule and the specificity of its most specific matching selector.
type MatchedRule<'a> = (Specificity, &'a Rule);

// If a rule matches an element return the rule else none
fn match_rule<'a>(element: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(element, selector))
        .map(|selector| (selector.specificity(), rule))
}

// find all css rules maching given element
fn matching_rules<'a>(element: &ElementData, stylesheet: &'a StylesSheet) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(element, rule))
        .collect()
}

// Matches the selector by looking into the element
fn matches(element: &ElementData, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(s) => matches_simple_selector(element, s),
    }
}

fn matches_simple_selector(element: &ElementData, selector: &SimpleSelector) -> bool {
    // Check tag name selector
    if selector
        .tag_name
        .iter()
        .any(|name| element.tag_name != *name)
    {
        return false;
    }

    if selector.id.iter().any(|id| element.id() != Some(id)) {
        return false;
    }

    if selector
        .class
        .iter()
        .any(|c| !element.classes().contains(c.as_str()))
    {
        return false;
    }

    true
}
