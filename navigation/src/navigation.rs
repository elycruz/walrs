// use std::borrow::Cow;

pub trait NavigationItem<'a> {
  // fn get_uri(&self) -> Option<Cow<'a, str>>;
  // fn get_label() -> Cow<'a, str>;
  fn add(&mut self, item: NavItem) -> isize;
  fn remove(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<NavItem>;
  fn find(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<&'a NavItem>;

  /// Gets number of nav items in nav tree.
  fn size(&mut self) -> isize;
}

pub struct NavItem {
  pub active: bool,
  pub attributes: Option<Vec<String>>,
  pub children_only: bool,
  pub fragment: Option<String>,
  pub items: Option<Vec<NavItem>>,
  pub label: String,
  pub order: u64,
  pub privilege: Option<String>,
  pub resource: Option<String>,
  pub uri: Option<String>,

  _stored_size: isize,
  _reevaluate_active_states: bool,
  _reevaluate_order: bool,
  _reevaluate_size: bool,
}

impl<'a> NavigationItem<'a> for NavItem {
  // fn get_uri(&self) -> Option<Cow<'a, str>> {
  //   self.uri.map(|uri| Cow::Borrowed(&uri.as_str()))
  // }

  fn add(&mut self, item: NavItem) -> isize {
    self._reevaluate_size = true;
    todo!()
  }

  fn remove(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<NavItem> {
    self._reevaluate_size = true;
    todo!()
  }

  fn find(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<&'a NavItem> {
    todo!()
  }

  fn size(&mut self) -> isize {
    if !self._reevaluate_size {
      return self._stored_size;
    }

    let mut size = 1;
    if let Some(items) = self.items.as_deref_mut() {
      for item in items {
        size += item.size();
      }
    }
    self._stored_size = size;
    size
  }
}
