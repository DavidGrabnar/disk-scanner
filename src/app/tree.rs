use eframe::egui::{CollapsingHeader, Ui};

pub enum Action {
    NoAction,
    Selected(String),
}

pub struct Tree(Node);

impl Tree {
    pub fn new(root: Node) -> Self {
        Self(root)
    }
}

impl Tree {
    pub fn ui(self, ui: &mut Ui) -> Action {
        self.0.ui(ui)
    }
}

#[derive(Clone)]
pub struct Node(String, Vec<Node>);

impl Node {
    pub fn new(label: String, children: Vec<Node>) -> Self {
        Self(label, children)
    }
}

impl Node {
    fn ui(self, ui: &mut Ui) -> Action {
        let collapsible = CollapsingHeader::new(self.0.clone());

        let response = collapsible.show(ui, |ui| {
            self.1
                .into_iter()
                .fold(Action::NoAction, |curr_action, child| {
                    let action = child.ui(ui);
                    if let Action::Selected(_x) = &action {
                        return action;
                    }
                    curr_action
                })
        });

        if let Some(Action::Selected(id)) = response.body_returned {
            return Action::Selected(id);
        }
        if response.header_response.clicked() {
            return Action::Selected(self.0);
        }
        Action::NoAction
    }
}
