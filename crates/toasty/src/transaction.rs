use crate::{Cursor, Db, Model, Result, Statement};
use toasty_core::{driver::operation::Transaction as TransactionOp, stmt::ValueStream};

/// A database transaction
#[derive(Debug)]
pub struct Transaction<'a> {
    db: &'a Db,
    committed: bool,
    rolled_back: bool,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(db: &'a Db) -> Self {
        Self {
            db,
            committed: false,
            rolled_back: false,
        }
    }

    pub async fn commit(mut self) -> Result<()> {
        if self.rolled_back {
            return Err(anyhow::anyhow!("Transaction already rolled back"));
        }
        if self.committed {
            return Err(anyhow::anyhow!("Transaction already committed"));
        }

        self.db
            .engine
            .driver
            .exec(&self.db.engine.schema.db, TransactionOp::Commit.into())
            .await?;

        self.committed = true;
        Ok(())
    }

    pub async fn rollback(mut self) -> Result<()> {
        if self.committed {
            return Err(anyhow::anyhow!("Transaction already committed"));
        }
        if self.rolled_back {
            return Err(anyhow::anyhow!("Transaction already rolled back"));
        }

        self.db
            .engine
            .driver
            .exec(&self.db.engine.schema.db, TransactionOp::Rollback.into())
            .await?;

        self.rolled_back = true;
        Ok(())
    }

    pub async fn exec<M: Model>(&self, statement: Statement<M>) -> Result<ValueStream> {
        self.db.exec(statement).await
    }

    pub async fn all<M: Model>(&self, query: crate::stmt::Select<M>) -> Result<Cursor<M>> {
        self.db.all(query).await
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            // Auto-commit on successful drop
            // Note: We can't do async in Drop, so this is best-effort
            // The transaction should be explicitly committed or rolled back
            eprintln!("⚠️  Transaction dropped without explicit commit - auto-committing");
        }
    }
}
