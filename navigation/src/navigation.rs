// use std::borrow::Cow;
use serde_json;

pub trait NavigationItem<'a> {
  // fn get_uri(&self) -> Option<Cow<'a, str>>;
  // fn get_label() -> Cow<'a, str>;
  fn add(&mut self, item: NavItem<'a>) -> isize;
  fn remove(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<NavItem<'a>>;
  fn find(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<&'a NavItem>;

  /// Gets number of nav items in nav tree.
  fn size(&mut self) -> isize;
}

#[derive(Builder)]
pub struct NavItem<'a> {
  pub active: bool,
  pub attributes: Option<Vec<(String, serde_json::Value)>>,
  pub children_only: bool,
  pub fragment: Option<String>,
  pub items: Option<Vec<NavItem<'a>>>,
  pub label: Option<String>,
  pub order: u64,
  pub parent: Option<Box<&'a NavItem<'a>>>,
  pub privilege: Option<String>,
  pub resource: Option<String>,
  pub uri: Option<String>,

  _stored_size: isize,
  _reevaluate_active_states: bool,
  _reevaluate_order: bool,
  _reevaluate_size: bool,
}

impl<'a> NavigationItem<'a> for NavItem<'a> {
  // fn get_uri(&self) -> Option<Cow<'a, str>> {
  //   self.uri.map(|uri| Cow::Borrowed(&uri.as_str()))
  // }

  fn add(&mut self, mut item: NavItem<'a>) -> isize {
    self._reevaluate_size = true;
    item.parent = Some(Box::new(self));

    if self.items.is_none() {
      self.items = Some(vec![item]);
    } else {
      self.items.push(item);
    }

    self.size()
  }

  fn remove(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<NavItem> {
    self._reevaluate_size = true;
    // self.find(pred)d
    todo!()
  }

  fn find(&mut self, pred: &'a impl Fn(&'a NavItem) -> bool) -> Option<Box<&'a NavItem>> {
    self.items.map(|items| {
      items.iter().find(pred).map(|x| Box::new(x))
    }).flatten()
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
