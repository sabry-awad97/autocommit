use prettytable::{Cell, Row, Table};

pub struct CommitSummary {
    pub branch_name: String,
    pub commit_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub commit_count: usize,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl CommitSummary {
    pub fn get_table(&self) -> anyhow::Result<Table> {
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
        table.set_titles(Row::new(vec![
            Cell::new("Commit Information").style_spec("bFy")
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Branch"),
            Cell::new("Commit Hash"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new(&self.branch_name),
            Cell::new(&self.commit_hash),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Author"),
            Cell::new("Email"),
            Cell::new("Commit Count"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new(&self.author_name),
            Cell::new(&self.author_email),
            Cell::new(&self.commit_count.to_string()),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Files Changed"),
            Cell::new("Insertions"),
            Cell::new("Deletions"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new(&self.files_changed.to_string()),
            Cell::new(&self.insertions.to_string()),
            Cell::new(&self.deletions.to_string()),
        ]));

        Ok(table)
    }
}
