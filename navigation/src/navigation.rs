// use std::borrow::Cow;
// use serde_json;

pub trait NavigationItem<'a> {
  // fn get_uri(&self) -> Option<Cow<'a, str>>;
  // fn get_label() -> Cow<'a, str>;
  fn add(&mut self, item: NavItem) -> isize;
  fn remove(&mut self, pred: impl Fn(&NavItem) -> bool) -> Option<NavItem>;
  fn find(&self, pred: impl Fn(&NavItem) -> bool) -> Option<NavItem>;

  /// Gets number of nav items in nav tree.
  fn size(&mut self) -> isize;
}

#[derive(Clone, Builder)]
pub struct NavItem {
  pub active: bool,
  pub attributes: Option<Vec<(String, serde_json::Value)>>,
  pub children_only: bool,
  pub fragment: Option<String>,
  pub items: Option<Vec<NavItem>>,
  pub label: Option<String>,
  pub order: u64,
  pub parent: Option<Box<NavItem>>,
  pub privilege: Option<String>,
  pub resource: Option<String>,
  pub uri: Option<String>,

  _stored_size: isize,
  _reevaluate_active_states: bool,
  _reevaluate_order: bool,
  _reevaluate_size: bool,
}

impl NavItem {
  pub fn new() -> Self {
    NavItem {
      active: false,
      attributes: None,
      children_only: false,
      fragment: None,
      items: None,
      label: None,
      order: 0,
      parent: None,
      privilege: None,
      resource: None,
      uri: None,

      _stored_size: 1,
      _reevaluate_active_states: false,
      _reevaluate_order: false,
      _reevaluate_size: false,
    }
  }
}

impl Default for NavItem {
  fn default() -> Self {
    NavItem::new()
  }
}

impl<'a> NavigationItem<'a> for NavItem {
  // fn get_uri(&self) -> Option<Cow<'a, str>> {
  //   self.uri.map(|uri| Cow::Borrowed(&uri.as_str()))
  // }

  fn add(&mut self, item: NavItem) -> isize {
    self._reevaluate_size = true;

    if self.items.is_none() {
      self.items = Some(vec![item]);
    } else {
      self.items.as_mut().unwrap().push(item);
    }

    self.size()
  }

  fn remove(&mut self, _pred: impl Fn(&'a NavItem) -> bool) -> Option<NavItem> {
    self._reevaluate_size = true;
    // self.find(pred)d
    todo!()
  }

  fn find(&self, pred: impl Fn(&NavItem) -> bool) -> Option<NavItem> {
    self
      .items
      .as_deref()
      .map(|items| items.iter().find(|item| pred(*item)).map(|x| x.clone()))
      .flatten()
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

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_nav_item() {
    let mut nav = NavItem::default();
    assert_eq!(nav.size(), 1);

    let mut nav2 = NavItem::default();
    nav2.add(NavItem::default());
    nav2.add(NavItem::default());
    nav.add(nav2);
    assert_eq!(nav.size(), 4);
  }
}
