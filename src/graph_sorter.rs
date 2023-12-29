use std::collections::HashMap;

use crate::IDepData;

pub struct GraphSorter;

impl GraphSorter {
  pub fn sort<DP: IDepData>(dep_datas: Vec<DP>) -> Vec<DP> {
    let nodes = dep_datas;
    let mut sorted = Vec::<DP>::new();
    let mut marks = HashMap::<&DP, Mark>::new();

    for n in nodes.iter() {
      marks.insert(n, Mark::None);
    }

    let mut sorter = Sorter {
      nodes: &nodes,
      sorted: &mut sorted,
      marks: &mut marks,
    };

    sorter.sort().unwrap();

    sorted
  }
}

enum Mark {
  None,
  Temp,
  Permanent,
}

struct Sorter<'a, DP: IDepData> {
  nodes: &'a Vec<DP>,
  sorted: &'a mut Vec<DP>,
  marks: &'a mut HashMap<&'a DP, Mark>,
}

impl<'a, DP: IDepData> Sorter<'a, DP> {
  pub fn sort(&mut self) -> Result<(), ()> {
    loop {
      if !self
        .marks
        .values()
        .any(|m| matches!(m, Mark::Temp | Mark::None))
      {
        return Ok(());
      }

      let (&d, _) = self
        .marks
        .iter()
        .find(|(_, m)| matches!(m, Mark::None))
        .unwrap();
      self.visit(d)?;
    }
  }

  pub fn visit(&mut self, n: &'a DP) -> Result<(), ()> {
    match self.marks.get(n).unwrap_or(&Mark::None) {
      Mark::Permanent => return Ok(()),
      Mark::Temp => return Err(()),
      Mark::None => {}
    };

    self.marks.insert(n, Mark::Temp);

    for m in self
      .nodes
      .iter()
      .filter(|m| n.children().contains(&m.name()))
    {
      self.visit(m)?
    }

    self.marks.insert(n, Mark::Permanent);

    self.sorted.push(n.clone());

    Ok(())
  }
}
