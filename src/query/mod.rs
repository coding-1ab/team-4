pub mod lexer;
pub mod parser;

pub struct TxId(pub u64);
pub struct TableId(pub u64);
pub struct ColumnId(pub u64);
pub struct RowId(pub u64);

struct QueryPlan {
    tx_id: TxId,
    command: Command,
}

enum Command {
    // Table operations
    CreateTable {
        name: String,
    },
    DropTable {
        table_id: TableId,
    },
    // Column operations
    CreateColumn {
        table_id: TableId,
        col_name: String,
        col_type: String,
    },
    DropColumn {
        table_id: TableId,
        col_id: ColumnId,
    },
    // Row operations
    InsertRow {
        table_id: TableId,
    },
    UpdateRow {
        table_id: TableId,
        row_id: RowId,
    },
    DeleteRow {
        table_id: TableId,
        row_id: RowId,
    },
}
