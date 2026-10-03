use super::tab::{Tab, TabId};

pub(super) struct Tabs {
    items: Vec<Tab>,
    active: Option<TabId>,
    hovered: Option<TabId>,
    next_id: u64,
}

pub(super) struct Removal {
    pub(super) tab: Tab,
    pub(super) active_changed: bool,
}

impl Tabs {
    pub(super) fn new() -> Self {
        Self {
            items: Vec::new(),
            active: None,
            hovered: None,
            next_id: 1,
        }
    }

    pub(super) fn reserve_id(&mut self) -> TabId {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("terminal tab IDs exhausted");
        TabId(id)
    }

    pub(super) fn insert(&mut self, tab: Tab) -> TabId {
        let id = tab.id();
        debug_assert!(
            id.0 < self.next_id,
            "tab ID must be reserved before insertion"
        );
        debug_assert!(
            !self.items.iter().any(|item| item.id() == id),
            "tab IDs must be unique"
        );
        let was_empty = self.items.is_empty();
        self.items.push(tab);
        if was_empty {
            self.activate(id);
        }
        id
    }

    pub(super) fn remove(&mut self, id: TabId) -> Option<Removal> {
        let index = self.items.iter().position(|tab| tab.id() == id)?;
        let was_active = self.active == Some(id);
        let tab = self.items.remove(index);

        if self.hovered == Some(id) {
            self.hovered = None;
        }

        if was_active {
            if self.items.is_empty() {
                self.active = None;
            } else {
                let replacement_idx = index.min(self.items.len() - 1);
                let replacement_id = self.items[replacement_idx].id();
                self.activate(replacement_id);
            }
        }

        Some(Removal {
            tab,
            active_changed: was_active,
        })
    }

    pub(super) fn activate(&mut self, id: TabId) -> bool {
        if !self.items.iter().any(|tab| tab.id() == id) {
            return false;
        }
        self.active = Some(id);
        true
    }

    pub(super) fn next_candidate_id(&self) -> Option<TabId> {
        let active = self.active?;
        let index = self.items.iter().position(|tab| tab.id() == active)?;
        Some(self.items[(index + 1) % self.items.len()].id())
    }

    pub(super) fn previous_candidate_id(&self) -> Option<TabId> {
        let active = self.active?;
        let index = self.items.iter().position(|tab| tab.id() == active)?;
        let previous = if index == 0 {
            self.items.len() - 1
        } else {
            index - 1
        };
        Some(self.items[previous].id())
    }

    pub(super) fn set_hovered(&mut self, id: TabId, is_hovered: bool) {
        if !self.items.iter().any(|tab| tab.id() == id) {
            return;
        }
        if is_hovered {
            self.hovered = Some(id);
        } else if self.hovered == Some(id) {
            self.hovered = None;
        }
    }

    pub(super) fn items(&self) -> &[Tab] {
        &self.items
    }

    pub(super) fn items_mut(&mut self) -> &mut [Tab] {
        &mut self.items
    }

    pub(super) fn active_id(&self) -> Option<TabId> {
        self.active
    }

    pub(super) fn hovered_id(&self) -> Option<TabId> {
        self.hovered
    }

    pub(super) fn active_tab(&self) -> Option<&Tab> {
        let id = self.active?;
        self.items.iter().find(|tab| tab.id() == id)
    }

    pub(super) fn get_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.items.iter_mut().find(|tab| tab.id() == id)
    }
}
