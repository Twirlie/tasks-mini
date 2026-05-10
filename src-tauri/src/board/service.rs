use crate::board::types::Board;
use crate::column::types::Column;

pub fn default_board() -> Board {
    let columns = vec![
        Column::new("Backlog".to_string(), 0).unwrap(),
        Column::new("Todo".to_string(), 1).unwrap(),
        Column::new("In Progress".to_string(), 2).unwrap(),
        Column::new("Done".to_string(), 3).unwrap(),
    ];
    Board::new("My Board".to_string(), columns).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_board_creates_board_with_4_columns() {
        let board = default_board();
        assert_eq!(board.columns.len(), 4);
        assert_eq!(board.columns[0].name, "Backlog");
        assert_eq!(board.columns[0].order, 0);
        assert_eq!(board.columns[1].name, "Todo");
        assert_eq!(board.columns[1].order, 1);
        assert_eq!(board.columns[2].name, "In Progress");
        assert_eq!(board.columns[2].order, 2);
        assert_eq!(board.columns[3].name, "Done");
        assert_eq!(board.columns[3].order, 3);
    }
}
