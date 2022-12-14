use std::error::Error;

use ostree::{gio::Cancellable, glib, glib::GString, MutableTree, Repo};

pub fn app_id_from_ref(refstring: &str) -> String {
    refstring.split('/').nth(1).unwrap().to_string()
}

pub fn mtree_lookup(
    mtree: &MutableTree,
    path: &[&str],
) -> Result<(Option<GString>, Option<MutableTree>), Box<dyn Error>> {
    match path {
        [file] => mtree.lookup(file).map_err(Into::into),
        [subdir, rest @ ..] => mtree_lookup(
            &mtree
                .lookup(subdir)?
                .1
                .ok_or_else(|| "subdirectory not found".to_string())?,
            rest,
        ),
        [] => Err("no path given".into()),
    }
}

pub fn mtree_lookup_file(mtree: &MutableTree, path: &[&str]) -> Result<GString, Box<dyn Error>> {
    mtree_lookup(mtree, path)?
        .0
        .ok_or_else(|| "file not found".into())
}

/// Wrapper for OSTree transactions that automatically aborts the transaction when dropped if it hasn't been committed.
pub struct Transaction<'a> {
    repo: &'a Repo,
    finished: bool,
}

impl<'a> Transaction<'a> {
    pub fn new(repo: &'a Repo) -> Result<Self, glib::Error> {
        repo.prepare_transaction(Cancellable::NONE)?;
        Ok(Self {
            repo,
            finished: false,
        })
    }

    pub fn commit(mut self) -> Result<(), glib::Error> {
        self.repo.commit_transaction(Cancellable::NONE)?;
        self.finished = true;
        Ok(())
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.repo
                .abort_transaction(Cancellable::NONE)
                .expect("Aborting the transaction should not fail");
        }
    }
}
