use std::collections::HashMap;

use depends::{
    core::{Clean, UpdateInput},
    derives::Value,
};

use crate::models::Comment;

#[derive(Debug, Default, Value)]
#[depends(custom_clean)]
pub struct Comments {
    /// Map of all comments.
    comments: HashMap<i64, Comment>,
    /// Map of all comments that have been added since the last generation.
    new_comment_ids: Vec<i64>,
    /// The current generation of the comments. If the generation changes, there
    /// are new comments.
    #[depends(hash)]
    generation: usize,
}

impl Comments {
    /// An iterator over all comments that have been added since the last
    /// generation.
    pub fn new_comments(&self) -> impl Iterator<Item = &Comment> + '_ {
        self.new_comment_ids.iter().map(|id| &self.comments[id])
    }
}

impl Clean for Comments {
    fn clean(&mut self) {
        self.new_comment_ids.clear();
    }
}

impl UpdateInput for Comments {
    type Update = Comment;

    fn update_mut(&mut self, update: Self::Update) {
        let comment_id = update.id;
        self.comments.insert(comment_id, update);
        self.new_comment_ids.push(comment_id);
        self.generation += 1;
    }
}
