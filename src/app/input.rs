use super::state::{Direction, InputBuffer};

impl InputBuffer {
    pub(super) fn move_cursor(&mut self, dir: Direction) {
        if dir == Direction::Left {
            let cursor_moved_left = self.cursor.saturating_sub(1);
            self.cursor = self.clamp_cursor(cursor_moved_left);
        } else if dir == Direction::Right {
            let cursor_moved_right = self.cursor.saturating_add(1);
            self.cursor = self.clamp_cursor(cursor_moved_right);
        }
    }

    pub(super) fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.text.insert(index, new_char);
        self.move_cursor(Direction::Right);
    }

    fn byte_index(&self) -> usize {
        self.text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.text.len())
    }

    pub(super) fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.text.chars().skip(current_index);

            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor(Direction::Left);
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.text.chars().count())
    }
}
