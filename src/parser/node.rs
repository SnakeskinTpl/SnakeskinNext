#[derive(Debug)]
pub enum Node {
    SingleComment{
        line: u32,
        content: String
    },

    MultComment {
        line_start: u32,
        line_end: u32,
        content: String
    },

    Directive {
        line_start: u32,
        line_end: u32,
        content: String,
        ext: bool,
        parent: Option<Box<Node>>,
        children: Vec<Node>
    },

    Expr {
        line_start: u32,
        line_end: u32,
        content: String
    }
}
