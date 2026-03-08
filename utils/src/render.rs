use serde::{Deserialize, Serialize};

// Rendering data model consumed by docs/src/_assets/renderer.js.
pub type RenderState = Vec<RenderBlock>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RenderBlock {
    #[serde(rename = "text")]
    Text(RenderTextBlock),
    #[serde(rename = "table")]
    Table(RenderTableBlock),
    #[serde(rename = "container")]
    Container(RenderContainerBlock),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderTextBlock {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "className", skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderTableBlock {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<RenderBlock>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<RenderTableRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "className", skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderTableRow {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cells: Vec<RenderBlock>,
    #[serde(rename = "className", skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderContainerBlock {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<RenderBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<RenderOrientation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<RenderDisplay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "className", skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderOrientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderDisplay {
    Inline,
    Block,
}

#[macro_export]
macro_rules! render_state {
    ($($block:expr),* $(,)?) => {
        vec![$($block),*]
    };
}

#[macro_export]
macro_rules! render_text {
    ($text:expr $(, $key:ident : $value:expr )* $(,)?) => {{
        #[allow(unused_mut)]
        let mut block = $crate::RenderTextBlock {
            text: ($text).to_string(),
            title: None,
            class_name: None,
        };
        $( $crate::render_text!(@set block, $key, $value); )*
        $crate::RenderBlock::Text(block)
    }};
    (@set $block:ident, title, $value:expr) => {
        $block.title = Some(($value).to_string());
    };
    (@set $block:ident, class, $value:expr) => {
        $block.class_name = Some(($value).to_string());
    };
}

#[macro_export]
macro_rules! render_row {
    (cells: $cells:expr $(, $key:ident : $value:expr )* $(,)?) => {{
        #[allow(unused_mut)]
        let mut row = $crate::RenderTableRow {
            cells: $cells,
            class_name: None,
        };
        $( $crate::render_row!(@set row, $key, $value); )*
        row
    }};
    ([$($cell:expr),* $(,)?] $(, $key:ident : $value:expr )* $(,)?) => {{
        #[allow(unused_mut)]
        let mut row = $crate::RenderTableRow {
            cells: vec![$($cell),*],
            class_name: None,
        };
        $( $crate::render_row!(@set row, $key, $value); )*
        row
    }};
    (@set $row:ident, class, $value:expr) => {
        $row.class_name = Some(($value).to_string());
    };
}

#[macro_export]
macro_rules! render_table {
    (
        columns: [$($column:expr),* $(,)?],
        rows: [$($row:expr),* $(,)?]
        $(, $key:ident : $value:expr )* $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut block = $crate::RenderTableBlock {
            columns: vec![$($column),*],
            rows: vec![$($row),*],
            title: None,
            class_name: None,
        };
        $( $crate::render_table!(@set block, $key, $value); )*
        $crate::RenderBlock::Table(block)
    }};
    (
        columns: $columns:expr,
        rows: $rows:expr
        $(, $key:ident : $value:expr )* $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut block = $crate::RenderTableBlock {
            columns: $columns,
            rows: $rows,
            title: None,
            class_name: None,
        };
        $( $crate::render_table!(@set block, $key, $value); )*
        $crate::RenderBlock::Table(block)
    }};
    (@set $block:ident, title, $value:expr) => {
        $block.title = Some(($value).to_string());
    };
    (@set $block:ident, class, $value:expr) => {
        $block.class_name = Some(($value).to_string());
    };
}

#[macro_export]
macro_rules! render_container {
    (
        children: [$($child:expr),* $(,)?]
        $(, $key:ident : $value:expr )* $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut block = $crate::RenderContainerBlock {
            children: vec![$($child),*],
            orientation: None,
            display: None,
            title: None,
            class_name: None,
        };
        $( $crate::render_container!(@set block, $key, $value); )*
        $crate::RenderBlock::Container(block)
    }};
    (
        children: $children:expr
        $(, $key:ident : $value:expr )* $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut block = $crate::RenderContainerBlock {
            children: $children,
            orientation: None,
            display: None,
            title: None,
            class_name: None,
        };
        $( $crate::render_container!(@set block, $key, $value); )*
        $crate::RenderBlock::Container(block)
    }};
    (@set $block:ident, orientation, $value:expr) => {
        $block.orientation = Some($value);
    };
    (@set $block:ident, display, $value:expr) => {
        $block.display = Some($value);
    };
    (@set $block:ident, title, $value:expr) => {
        $block.title = Some(($value).to_string());
    };
    (@set $block:ident, class, $value:expr) => {
        $block.class_name = Some(($value).to_string());
    };
}

#[cfg(test)]
mod render_schema_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_block_shape_matches_renderer_contract() {
        let state: RenderState = vec![RenderBlock::Text(RenderTextBlock {
            text: "ok".to_string(),
            title: Some("status".to_string()),
            class_name: Some("good".to_string()),
        })];
        let value = serde_json::to_value(state).expect("serialize should succeed");
        assert_eq!(
            value,
            json!([{
                "kind": "text",
                "text": "ok",
                "title": "status",
                "className": "good"
            }])
        );
    }

    #[test]
    fn render_macros_build_valid_shape() {
        let state = render_state![
            render_text!("hello", title: "greeting"),
            render_table!(
                columns: [render_text!("k"), render_text!("v")],
                rows: [render_row!([render_text!("a"), render_text!("1")], class: "ok")],
                class: "table-main"
            ),
            render_container!(
                children: [render_text!("x"), render_text!("y")],
                orientation: RenderOrientation::Horizontal,
                display: RenderDisplay::Inline
            ),
        ];
        let value = serde_json::to_value(state).expect("serialize should succeed");
        assert!(value.is_array());
    }
}
