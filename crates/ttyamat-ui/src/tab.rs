#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TabId(pub u64);

#[derive(Debug, Clone)]
pub(crate) struct Tab {
    pub id: TabId,
    pub title: String,
}
