use std::{char, io::Read};

const SUB_STRING_LENGTH: usize = 8;

// FIX: might need to add reference to parent
#[derive(Default, Debug)]
struct Node {
    len: usize,
    right: Option<Box<Node>>,
    left: Option<Box<Node>>,
    str: Option<String>
}

impl Node {
    fn new() -> Box<Self> {
        Box::new(Node::default())
    }

    fn from(text: &str) -> Box<Self> {
        if text.len() > SUB_STRING_LENGTH {
            panic!("This is suppose to happen");
        }

        let mut node = Node::new();
        node.len = text.len();
        node.str = Some(String::from(text));
        node
    }

    // sum leaves of left subtree
    fn get_left_len(&self) -> usize {
        if self.is_leaf() {
            return self.len;
        }

        fn helper(node: &Option<Box<Node>>, count: usize) -> usize {
            if let Some(node) = node {
                if node.is_leaf() {
                    return node.len + count;
                }

                let count = helper(&node.left, count);
                let count = helper(&node.right, count);

                return count;
            }

            count
        }

        return helper(&self.left, 0);
    }

    fn join(left: Box<Self>, right: Box<Self>) -> Box<Self> {
        let mut node = Node::new();

        node.left = Some(left);
        node.right = Some(right);
        node.len = node.get_left_len();
        node
    }

    fn is_leaf(&self) -> bool {
        self.str.is_some()
    }
}

#[derive(Debug)]
pub struct Rope {
    root: Option<Box<Node>>
}

impl Rope {
    fn new() -> Self {
        Rope {
            root: None
        }
    }

    fn from(text: String) -> Self {
        let mut nodes = Vec::new();
        for chunk in  text.chars().collect::<Vec<char>>().chunks(SUB_STRING_LENGTH) {
            let mut temp = String::new();
            for c in chunk.iter().to_owned() {
                temp.push(*c);
            }

            nodes.push(Node::from(&temp));
        }

        while nodes.len() > 1 {
            let first = nodes.remove(0);
            let sec = nodes.remove(0);

            let new_node = Node::join(first, sec);

            nodes.insert(0, new_node);
        }

        if nodes.len() < 1 {
            return Rope::new();
        }

        let mut rope = Rope::new();
        let root = nodes.pop().unwrap();

        rope.root = Some(root);

        rope
    }

    fn to_string(&self) -> String {
        fn helper(node: &Option<Box<Node>>, s: &mut String) {
            if let Some(node) = node {
                if node.is_leaf() {
                    let temp = node.str.clone().unwrap();
                    s.push_str(&temp);
                } else {
                    helper(&node.left, s);
                    helper(&node.right, s);
                }
            }
        }

        let mut s = String::new();
        helper(&self.root, &mut s);
        return s;
    }
}

#[test]
fn node_test() {
    let node = Node::new();

    assert_eq!(node.len, 0);
    assert!(node.right.is_none());
    assert!(node.left.is_none());
    assert!(node.str.is_none());
}

#[test]
fn rope_test() {
    use std::fs::File;

    let x = String::from("hello, this is a testing line\n");
    let rope = Rope::from(x.clone());

    assert_eq!(rope.to_string(), x);


    let mut big_test = String::new();

    for i in 1..=100 {
        big_test.push_str(&format!("{i}\n"))
    }

    let rope = Rope::from(big_test.clone());
    println!("Root: {:#?}", rope);
    assert_eq!(rope.to_string(), big_test);

    if let Some(root) = rope.root {
        println!("Root: {:#?}", root);
        assert!(root.str.is_none());
        assert!(!root.is_leaf());
        // assert_eq!(x.len(), root.get_left_len()); // not a valid test
    } else {
        assert!(false)
    }

    // testing on file
    if let Ok(mut file) = File::open("./src/editor.rs") {
        let mut reader = Vec::new();
        file.read_to_end(&mut reader).unwrap();
        let text = String::from_utf8(reader).unwrap();
        let rope = Rope::from(text.clone());
        assert_eq!(rope.to_string(), text);
    }
}
