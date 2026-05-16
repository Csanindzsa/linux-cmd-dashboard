use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

impl Direction {
    pub fn split_orientation(self) -> SplitOrientation {
        match self {
            Direction::Left | Direction::Right => SplitOrientation::Horizontal,
            Direction::Up | Direction::Down => SplitOrientation::Vertical,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutTree {
    Leaf(PaneId),
    Split {
        orientation: SplitOrientation,
        ratio: f64,
        first: Box<LayoutTree>,
        second: Box<LayoutTree>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    root: LayoutTree,
    focused: PaneId,
    next_id: u64,
}

impl Workspace {
    pub fn new() -> Self {
        let first = PaneId(1);
        Self {
            root: LayoutTree::Leaf(first),
            focused: first,
            next_id: 2,
        }
    }

    pub fn root(&self) -> &LayoutTree {
        &self.root
    }

    pub fn focused(&self) -> PaneId {
        self.focused
    }

    pub fn pane_ids(&self) -> Vec<PaneId> {
        let mut ids = Vec::new();
        self.root.walk_panes(&mut ids);
        ids
    }

    pub fn split_focused(&mut self, orientation: SplitOrientation) -> PaneId {
        let new_id = self.allocate_pane();
        self.root
            .replace_leaf(self.focused, |old| LayoutTree::Split {
                orientation,
                ratio: 0.5,
                first: Box::new(LayoutTree::Leaf(old)),
                second: Box::new(LayoutTree::Leaf(new_id)),
            });
        self.focused = new_id;
        new_id
    }

    pub fn split_focused_toward(&mut self, direction: Direction) -> PaneId {
        let new_id = self.allocate_pane();
        let orientation = direction.split_orientation();
        self.root.replace_leaf(self.focused, |old| {
            let old = LayoutTree::Leaf(old);
            let new = LayoutTree::Leaf(new_id);
            let (first, second) = match direction {
                Direction::Left | Direction::Up => (new, old),
                Direction::Right | Direction::Down => (old, new),
            };
            LayoutTree::Split {
                orientation,
                ratio: 0.5,
                first: Box::new(first),
                second: Box::new(second),
            }
        });
        self.focused = new_id;
        new_id
    }

    pub fn close_focused(&mut self) -> Option<PaneId> {
        let closing = self.focused;
        let remaining = self.pane_ids();
        if remaining.len() <= 1 {
            return None;
        }

        self.root = self
            .root
            .clone()
            .remove_leaf(closing)
            .expect("root remains");
        self.focused = self.nearest_to_removed(closing).unwrap_or_else(|| {
            self.pane_ids()
                .into_iter()
                .next()
                .expect("workspace has a remaining pane")
        });
        Some(closing)
    }

    pub fn focus(&mut self, pane: PaneId) -> bool {
        if self.root.contains(pane) {
            self.focused = pane;
            true
        } else {
            false
        }
    }

    pub fn focus_neighbor(&mut self, direction: Direction) -> Option<PaneId> {
        let neighbor = self.neighbor(self.focused, direction)?;
        self.focused = neighbor;
        Some(neighbor)
    }

    pub fn move_focused(&mut self, direction: Direction) -> Option<PaneId> {
        let neighbor = self.neighbor(self.focused, direction)?;
        self.root.swap_leaves(self.focused, neighbor);
        Some(neighbor)
    }

    pub fn resize_focused(&mut self, direction: Direction, delta: f64) -> bool {
        self.root.resize_path_to(self.focused, direction, delta)
    }

    pub fn rectangles(&self) -> HashMap<PaneId, Rect> {
        let mut rects = HashMap::new();
        self.root.collect_rects(
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            &mut rects,
        );
        rects
    }

    pub fn neighbor(&self, pane: PaneId, direction: Direction) -> Option<PaneId> {
        let rects = self.rectangles();
        let current = *rects.get(&pane)?;
        rects
            .iter()
            .filter(|(id, _)| **id != pane)
            .filter_map(|(id, rect)| {
                let overlap = match direction {
                    Direction::Left | Direction::Right => {
                        axis_overlap(current.y, current.y + current.h, rect.y, rect.y + rect.h)
                    }
                    Direction::Up | Direction::Down => {
                        axis_overlap(current.x, current.x + current.w, rect.x, rect.x + rect.w)
                    }
                };
                if overlap <= 0.0 {
                    return None;
                }

                let gap = match direction {
                    Direction::Left if rect.x + rect.w <= current.x => {
                        current.x - (rect.x + rect.w)
                    }
                    Direction::Right if rect.x >= current.x + current.w => {
                        rect.x - (current.x + current.w)
                    }
                    Direction::Up if rect.y + rect.h <= current.y => current.y - (rect.y + rect.h),
                    Direction::Down if rect.y >= current.y + current.h => {
                        rect.y - (current.y + current.h)
                    }
                    _ => return None,
                };

                let cross_distance = match direction {
                    Direction::Left | Direction::Right => {
                        ((rect.y + rect.h / 2.0) - (current.y + current.h / 2.0)).abs()
                    }
                    Direction::Up | Direction::Down => {
                        ((rect.x + rect.w / 2.0) - (current.x + current.w / 2.0)).abs()
                    }
                };

                Some((*id, gap, -overlap, cross_distance))
            })
            .min_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap()
                    .then_with(|| a.2.partial_cmp(&b.2).unwrap())
                    .then_with(|| a.3.partial_cmp(&b.3).unwrap())
                    .then_with(|| a.0.cmp(&b.0))
            })
            .map(|(id, _, _, _)| id)
    }

    fn allocate_pane(&mut self) -> PaneId {
        let id = PaneId(self.next_id);
        self.next_id += 1;
        id
    }

    fn nearest_to_removed(&self, _removed: PaneId) -> Option<PaneId> {
        self.pane_ids().into_iter().last()
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutTree {
    fn walk_panes(&self, ids: &mut Vec<PaneId>) {
        match self {
            LayoutTree::Leaf(id) => ids.push(*id),
            LayoutTree::Split { first, second, .. } => {
                first.walk_panes(ids);
                second.walk_panes(ids);
            }
        }
    }

    fn contains(&self, pane: PaneId) -> bool {
        match self {
            LayoutTree::Leaf(id) => *id == pane,
            LayoutTree::Split { first, second, .. } => {
                first.contains(pane) || second.contains(pane)
            }
        }
    }

    fn replace_leaf(
        &mut self,
        target: PaneId,
        replacement: impl FnOnce(PaneId) -> LayoutTree,
    ) -> bool {
        let mut replacement = Some(replacement);
        self.replace_leaf_inner(target, &mut replacement)
    }

    fn replace_leaf_inner<F>(&mut self, target: PaneId, replacement: &mut Option<F>) -> bool
    where
        F: FnOnce(PaneId) -> LayoutTree,
    {
        match self {
            LayoutTree::Leaf(id) if *id == target => {
                let replacement = replacement
                    .take()
                    .expect("replacement is available at target");
                *self = replacement(*id);
                true
            }
            LayoutTree::Leaf(_) => false,
            LayoutTree::Split { first, second, .. } => {
                if first.replace_leaf_inner(target, replacement) {
                    true
                } else {
                    second.replace_leaf_inner(target, replacement)
                }
            }
        }
    }

    fn remove_leaf(self, target: PaneId) -> Option<LayoutTree> {
        match self {
            LayoutTree::Leaf(id) if id == target => None,
            LayoutTree::Leaf(_) => Some(self),
            LayoutTree::Split {
                orientation,
                ratio,
                first,
                second,
            } => match (first.remove_leaf(target), second.remove_leaf(target)) {
                (Some(first), Some(second)) => Some(LayoutTree::Split {
                    orientation,
                    ratio,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (Some(remaining), None) | (None, Some(remaining)) => Some(remaining),
                (None, None) => None,
            },
        }
    }

    fn swap_leaves(&mut self, a: PaneId, b: PaneId) -> bool {
        let mut found = 0;
        self.map_leaves(&mut |id| {
            if *id == a {
                *id = b;
                found += 1;
            } else if *id == b {
                *id = a;
                found += 1;
            }
        });
        found == 2
    }

    fn map_leaves(&mut self, f: &mut impl FnMut(&mut PaneId)) {
        match self {
            LayoutTree::Leaf(id) => f(id),
            LayoutTree::Split { first, second, .. } => {
                first.map_leaves(f);
                second.map_leaves(f);
            }
        }
    }

    fn resize_path_to(&mut self, target: PaneId, direction: Direction, delta: f64) -> bool {
        match self {
            LayoutTree::Leaf(id) => *id == target,
            LayoutTree::Split {
                orientation,
                ratio,
                first,
                second,
            } => {
                if first.contains(target) {
                    if *orientation == direction.split_orientation() {
                        match direction {
                            Direction::Right | Direction::Down => *ratio += delta,
                            Direction::Left | Direction::Up => *ratio -= delta,
                        }
                        *ratio = ratio.clamp(0.15, 0.85);
                        true
                    } else {
                        first.resize_path_to(target, direction, delta)
                    }
                } else if second.contains(target) {
                    if *orientation == direction.split_orientation() {
                        match direction {
                            Direction::Right | Direction::Down => *ratio -= delta,
                            Direction::Left | Direction::Up => *ratio += delta,
                        }
                        *ratio = ratio.clamp(0.15, 0.85);
                        true
                    } else {
                        second.resize_path_to(target, direction, delta)
                    }
                } else {
                    false
                }
            }
        }
    }

    fn collect_rects(&self, rect: Rect, rects: &mut HashMap<PaneId, Rect>) {
        match self {
            LayoutTree::Leaf(id) => {
                rects.insert(*id, rect);
            }
            LayoutTree::Split {
                orientation,
                ratio,
                first,
                second,
            } => match orientation {
                SplitOrientation::Horizontal => {
                    let first_w = rect.w * ratio;
                    first.collect_rects(
                        Rect {
                            x: rect.x,
                            y: rect.y,
                            w: first_w,
                            h: rect.h,
                        },
                        rects,
                    );
                    second.collect_rects(
                        Rect {
                            x: rect.x + first_w,
                            y: rect.y,
                            w: rect.w - first_w,
                            h: rect.h,
                        },
                        rects,
                    );
                }
                SplitOrientation::Vertical => {
                    let first_h = rect.h * ratio;
                    first.collect_rects(
                        Rect {
                            x: rect.x,
                            y: rect.y,
                            w: rect.w,
                            h: first_h,
                        },
                        rects,
                    );
                    second.collect_rects(
                        Rect {
                            x: rect.x,
                            y: rect.y + first_h,
                            w: rect.w,
                            h: rect.h - first_h,
                        },
                        rects,
                    );
                }
            },
        }
    }
}

fn axis_overlap(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    a1.min(b1) - a0.max(b0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_focused_adds_new_pane_and_focuses_it() {
        let mut workspace = Workspace::new();

        let second = workspace.split_focused(SplitOrientation::Horizontal);

        assert_eq!(second, PaneId(2));
        assert_eq!(workspace.focused(), PaneId(2));
        assert_eq!(workspace.pane_ids(), vec![PaneId(1), PaneId(2)]);
        assert_eq!(
            workspace.neighbor(PaneId(1), Direction::Right),
            Some(PaneId(2))
        );
    }

    #[test]
    fn close_focused_collapses_parent_split() {
        let mut workspace = Workspace::new();
        workspace.split_focused(SplitOrientation::Horizontal);

        assert_eq!(workspace.close_focused(), Some(PaneId(2)));

        assert_eq!(workspace.pane_ids(), vec![PaneId(1)]);
        assert_eq!(workspace.focused(), PaneId(1));
        assert_eq!(workspace.root(), &LayoutTree::Leaf(PaneId(1)));
    }

    #[test]
    fn neighbor_uses_layout_rectangles() {
        let mut workspace = Workspace::new();
        let right = workspace.split_focused(SplitOrientation::Horizontal);
        workspace.focus(PaneId(1));
        let bottom_left = workspace.split_focused(SplitOrientation::Vertical);

        assert_eq!(
            workspace.neighbor(PaneId(1), Direction::Down),
            Some(bottom_left)
        );
        assert_eq!(workspace.neighbor(PaneId(1), Direction::Right), Some(right));
        assert_eq!(workspace.neighbor(right, Direction::Left), Some(PaneId(1)));
    }

    #[test]
    fn move_focused_swaps_with_neighbor() {
        let mut workspace = Workspace::new();
        workspace.split_focused(SplitOrientation::Horizontal);

        assert_eq!(workspace.move_focused(Direction::Left), Some(PaneId(1)));

        assert_eq!(
            workspace.neighbor(PaneId(2), Direction::Right),
            Some(PaneId(1))
        );
        assert_eq!(workspace.focused(), PaneId(2));
    }

    #[test]
    fn resize_changes_split_ratio() {
        let mut workspace = Workspace::new();
        workspace.split_focused(SplitOrientation::Horizontal);
        workspace.focus(PaneId(1));

        assert!(workspace.resize_focused(Direction::Right, 0.1));

        match workspace.root() {
            LayoutTree::Split { ratio, .. } => assert!((*ratio - 0.6).abs() < f64::EPSILON),
            other => panic!("expected split, got {other:?}"),
        }
    }
}
