use roxmltree::Node;

use crate::youtube::{YouTubeError, YouTubeResult};

pub fn get_child_by_name<'a>(node: &'a Node, name: &str) -> YouTubeResult<Node<'a, 'a>> {
    node.descendants()
        .find(|n| n.has_tag_name(name))
        .ok_or_else(|| YouTubeError::ParserError(format!("missing tag {name}")))
}

pub fn get_child_text_by_name<'a>(node: &'a Node, name: &str) -> YouTubeResult<&'a str> {
    let child = get_child_by_name(node, name)?;
    child
        .text()
        .ok_or_else(|| YouTubeError::ParserError("missing tag content for {name}".to_string()))
}

pub fn get_children_by_name<'a>(node: &'a Node, name: &str) -> impl Iterator<Item = Node<'a, 'a>> {
    node.descendants().filter(move |n| n.has_tag_name(name))
}
