// Stitches css node and html node to generate a style node that will make up the style tree
// Single node in the dom tree has a single node in the style tree (not to be confused with css tree)
// TODO: Things that need to be included -
// 1. Cascading
// 2. Initial and/or computed values (Implementing this would require separate code for each
//    property, based on its css specs).
// 3. Inheritance
// 4. The style attribute

use std::{cmp::Ordering, collections::HashMap};

use crate::{
    css::{
        CSSOrigin, Declaration, Rule, Selector, SimpleSelector, Specificity, StylesSheet, Value,
        INHERITED_PROPERTY,
    },
    dom::{ElementData, Node, NodeType},
};

// Map css property names to values
type PropertyMap = HashMap<String, Value>;

// Node associated with style data
#[derive(Debug)]
pub struct StyledNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}

/// single CSS rule and the specificity of its most specific matching selector.
type MatchedRule<'a> = (Specificity, CSSOrigin, &'a Rule);

// If a rule matches an element return the rule else none
fn match_rule<'a>(
    element: &ElementData,
    origin: CSSOrigin,
    rule: &'a Rule,
) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(element, selector))
        .map(|selector| (selector.specificity(), origin, rule))
}

// find all css rules maching given element
fn matching_rules<'a>(
    element: &ElementData,
    origin: CSSOrigin,
    stylesheet: &'a StylesSheet,
) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(element, origin, rule))
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

// apply styles to the single element, retuning the specified values
fn specified_values(
    element: &ElementData,
    stylesheets: &[StylesSheet],
    parent_specified_values: Option<&PropertyMap>,
) -> PropertyMap {
    let mut values = HashMap::new();
    let mut author_rules: Vec<MatchedRule> = Vec::new();
    let mut user_rules: Vec<MatchedRule> = Vec::new();

    stylesheets.iter().for_each(|s| match s.origin {
        CSSOrigin::User => {
            user_rules.append(&mut matching_rules(element, CSSOrigin::User, s));
        }
        CSSOrigin::Author => {
            author_rules.append(&mut matching_rules(element, CSSOrigin::Author, s));
        }
    });

    let mut applied_rules = author_rules;
    applied_rules.extend(user_rules);

    applied_rules.sort_by(|&(a, a_origin, _), &(b, b_origin, _)| {
        if a_origin == CSSOrigin::User && b_origin == CSSOrigin::Author {
            Ordering::Less
        } else if a_origin == CSSOrigin::Author && b_origin == CSSOrigin::User {
            Ordering::Greater
        } else {
            a.cmp(&b)
        }
    });

    let mut declarations: Vec<Declaration> = Vec::new();
    applied_rules.iter().for_each(|&(_, _, rule)| {
        // TODO: Share immutable reference instead of cloning to avoid large memory usage
        declarations.extend(rule.declarations.iter().cloned());
    });

    declarations.sort_by(|a, b| {
        if a.origin == CSSOrigin::User && a.is_important {
            Ordering::Greater
        } else if (b.origin == CSSOrigin::User && b.is_important)
            || (a.origin == CSSOrigin::User && b.origin == CSSOrigin::Author)
        {
            Ordering::Less
        } else if (a.origin == CSSOrigin::Author && b.origin == CSSOrigin::User)
            || (a.is_important && !b.is_important)
        {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    for dec in declarations {
        values.insert(dec.name.clone(), dec.value.clone());
    }

    if let Some(parent_values) = parent_specified_values {
        for (k, v) in parent_values {
            if !values.contains_key(k) && INHERITED_PROPERTY.contains_key(k) {
                values.insert(k.clone(), v.clone());
            }
        }
    }

    values
}

// apply a stylesheet to an entire DOM tree and return style node tree
pub fn style_tree<'a>(
    root: &'a Node,
    stylesheets: &'a Vec<StylesSheet>,
    parent_specified_values: Option<&PropertyMap>,
) -> StyledNode<'a> {
    let specified_values = match root.node_type {
        NodeType::Element(ref element) => {
            specified_values(element, stylesheets, parent_specified_values)
        }
        NodeType::Text(_) => HashMap::new(),
        NodeType::Comment(_) => HashMap::new(),
    };

    let children = root
        .children
        .iter()
        .map(|child| style_tree(child, stylesheets, Some(&specified_values)))
        .collect();

    StyledNode {
        node: root,
        specified_values,
        children,
    }
}
