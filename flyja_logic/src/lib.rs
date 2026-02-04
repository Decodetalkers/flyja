#[cfg(test)]
mod tests;
mod utils;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{self, AtomicU64};
pub mod error;

pub use error::FlyjaError as Error;

pub use crate::utils::{Direction, InsertWay, Percentage, Position, Size, SizeAndPos};

use crate::utils::{MapUnit, MinusAbleMatUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The id of the window.
///
/// Internally Iced reserves `window::Id::MAIN` for the first window spawned.
pub struct Id(pub u64);

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Id: {}", self.0))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

static COUNT: AtomicU64 = AtomicU64::new(0);

impl Id {
    /// The reserved window [`Id`] for the first window in an Iced application.
    pub const MAIN: Self = Id(0);

    /// Creates a new unique window [`Id`].
    pub fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }

    /// It is used in unit test
    #[cfg(test)]
    fn next(&self) -> Id {
        Id(self.0 + 1)
    }
}

#[derive(Debug, Clone)]
pub struct TopElementMap<T: MapUnit = f32>(Element<T>);
impl<T: MinusAbleMatUnit> TopElementMap<T> {
    /// create a new [`TopElementMap<T>`]
    pub fn new(size_pos: SizeAndPos<T>) -> Self {
        Self(Element::new(size_pos))
    }

    /// Get the information of size and position
    pub fn size_pos(&self) -> SizeAndPos<T> {
        self.0.size_pos()
    }

    pub fn position(&self) -> Position<T> {
        self.0.position()
    }

    /// return the size of current container
    pub fn size(&self) -> Size<T> {
        self.0.size()
    }

    /// returnt the width of current container
    pub fn width(&self) -> T {
        self.0.width()
    }

    /// return the size of the container
    pub fn height(&self) -> T {
        self.0.height()
    }

    /// check if the container contains a window
    pub fn has_id(&self, target: Id) -> bool {
        self.0.has_id(target)
    }

    /// Find a window with id, and get all information
    pub fn find_window(&self, target: Id) -> Option<&Element<T>> {
        self.0.find_window(target)
    }

    /// Swap two elements
    pub fn swap<F>(&mut self, id: Id, target: Id, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        self.0.swap(id, target, f)
    }

    /// Remap, when the container or the display changed, invoke this function
    pub fn remap<F>(&mut self, c_size_pos: SizeAndPos<T>, f: &mut F)
    where
        F: DispatchCallback<T>,
    {
        self.0.remap(c_size_pos, f);
    }

    /// Delete a window from the map or container. If failed, return a error
    /// It only fails when it cannot find this id
    pub fn delete<F>(&mut self, target: Id, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        self.0.delete(target, f)
    }

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// the result show if the operation is succeeded
    /// It only fails when the target id is not found
    pub fn insert<F>(&mut self, id: Id, target: Id, direction: Direction, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        self.0.insert(id, target, direction, f)
    }

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// the result show if the operation is succeeded
    /// It only fails when the target id is not found
    pub fn insert_new<F>(&mut self, id: Id, target: Id, way: InsertWay, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        self.0.insert_new(id, target, way, f)
    }

    // TODO: unit tests
    pub fn drag_resize<F>(
        &mut self,
        transfer: T,
        direction: Direction,
        target: Id,
        f: &mut F,
    ) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        self.0.drag_resize(transfer, direction, target, f)
    }

    /// drag and drop an element
    pub fn drag_and_drop<F>(
        &mut self,
        id: Id,
        target: Id,
        direction: Direction,
        f: &mut F,
    ) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        // NOTE: this will make the size change will only happened once
        let mut ids = HashMap::new();
        self.0
            .drag_and_drop(id, target, direction, &mut |id, new_size_and_pos| {
                ids.insert(id, new_size_and_pos);
            })?;
        for (id, size_pos) in ids.iter() {
            f.callback(*id, *size_pos);
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub enum Element<T: MapUnit = f32> {
    /// Only when the container is empty
    /// It contains the size and position information of the container
    EmptyOutput(SizeAndPos<T>),
    Window {
        /// This id is unique, show the identy of the window
        id: Id,
        /// contains the information of size and position
        size_pos: SizeAndPos<T>,
        // This storage current percentage in the container (if it is in a container)
        percent: Percentage,
    },
    /// A vertical container
    Vertical {
        /// All the [`Element<T>`] in the container
        elements: Vec<Element<T>>,
        /// contains the information of size and position
        size_pos: SizeAndPos<T>,
        // This storage current percentage in the container (if it is in a container)
        percent: Percentage,
    },
    Horizontal {
        /// All the [`Element<T>`] in the container
        elements: Vec<Element<T>>,
        /// contains the information of size and position
        size_pos: SizeAndPos<T>,
        // This storage current percentage in the container (if it is in a container)
        percent: Percentage,
    },
}

pub trait DispatchCallback<T: MapUnit> {
    fn callback(&mut self, id: Id, size_pos: SizeAndPos<T>);
}

impl<F, T: MapUnit> DispatchCallback<T> for F
where
    F: FnMut(Id, SizeAndPos<T>),
{
    fn callback(&mut self, id: Id, size_pos: SizeAndPos<T>) {
        self(id, size_pos)
    }
}

impl<T: MapUnit> DispatchCallback<T> for () {
    fn callback(&mut self, _id: Id, _size_pos: SizeAndPos<T>) {}
}

impl<T: MinusAbleMatUnit> Element<T> {
    /// new a new element with the [`SizeAndPos<T>`]
    pub fn new(size_pos: SizeAndPos<T>) -> Self {
        Self::EmptyOutput(size_pos)
    }

    /// Get the id
    pub fn id(&self) -> Option<Id> {
        match self {
            Self::Window { id, .. } => Some(*id),
            _ => None,
        }
    }

    /// How much space does the element use in current container (the upper one)
    pub fn percent(&self) -> Percentage {
        match self {
            Self::Vertical { percent, .. }
            | Self::Horizontal { percent, .. }
            | Self::Window { percent, .. } => *percent,
            Self::EmptyOutput(_) => Size::whole(),
        }
    }

    /// Get the information of size and position
    pub fn size_pos(&self) -> SizeAndPos<T> {
        match self {
            Self::Vertical { size_pos, .. }
            | Self::Horizontal { size_pos, .. }
            | Self::Window { size_pos, .. }
            | Self::EmptyOutput(size_pos) => *size_pos,
        }
    }

    /// get the information of the position
    pub fn position(&self) -> Position<T> {
        self.size_pos().position
    }

    /// Get the information of size
    pub fn size(&self) -> Size<T> {
        self.size_pos().size
    }

    /// get the information of width
    pub fn width(&self) -> T {
        self.size().width
    }

    /// This will return the size of the container
    pub fn height(&self) -> T {
        self.size().height
    }

    // NOTE: how to design it? what should I do with the size_pos? how does it mean?
    // maybe I need minus
    fn expand<F>(&mut self, change: SizeAndPos<T>, diff_percent: Size, callback: &mut F)
    where
        F: DispatchCallback<T>,
    {
        match self {
            Self::EmptyOutput(_) => {}
            Self::Window {
                size_pos,
                id,
                percent,
            } => {
                *size_pos += change;
                *percent += diff_percent;
                callback.callback(*id, *size_pos);
            }
            // NOTE: this time only change the percent of the top, and the size of the elements
            Self::Vertical {
                elements,
                size_pos,
                percent,
            } => {
                *size_pos += change;
                *percent += diff_percent;
                let container_height = size_pos.size.height;
                let mut current_y = size_pos.position.y;

                for element in elements {
                    let percent_y = element.percent().height;
                    // This will be the width
                    let height = container_height.mul_f32(percent_y);

                    let new_s_and_p: SizeAndPos<T> = SizeAndPos {
                        size: Size {
                            width: size_pos.size.width,
                            height,
                        },
                        position: Position {
                            x: size_pos.position.x,
                            y: current_y,
                        },
                    };
                    // Now we can caculte current height
                    current_y += height;
                    let s_and_p = element.size_pos();
                    let diff_change = new_s_and_p - s_and_p;
                    element.expand(diff_change, Size::zero(), callback);
                }
            }
            Self::Horizontal {
                elements,
                size_pos,
                percent,
            } => {
                *size_pos += change;
                *percent += diff_percent;
                let container_width = size_pos.size.width;
                let mut current_x = size_pos.position.x;

                for element in elements {
                    let percent_x = element.percent().width;
                    // This will be the width
                    let width = container_width.mul_f32(percent_x);

                    let new_s_and_p: SizeAndPos<T> = SizeAndPos {
                        size: Size {
                            width,
                            height: size_pos.size.height,
                        },
                        position: Position {
                            x: current_x,
                            y: size_pos.position.y,
                        },
                    };
                    // Now we can caculte current height
                    current_x += width;
                    let s_and_p = element.size_pos();
                    let diff_change = new_s_and_p - s_and_p;
                    element.expand(diff_change, Size::zero(), callback);
                }
            }
        }
    }

    fn set_percentage(&mut self, c_percent: Size) {
        match self {
            Self::EmptyOutput(_) => {}
            Self::Vertical { percent, .. }
            | Self::Horizontal { percent, .. }
            | Self::Window { percent, .. } => *percent = c_percent,
        }
    }

    fn set_size_and_pos(&mut self, c_size_pos: SizeAndPos<T>) {
        match self {
            Self::EmptyOutput(size_pos)
            | Self::Vertical { size_pos, .. }
            | Self::Horizontal { size_pos, .. }
            | Self::Window { size_pos, .. } => *size_pos = c_size_pos,
        }
    }

    /// check if the container is a window
    pub fn is_window(&self) -> bool {
        matches!(self, Self::Window { .. })
    }

    /// Check if current container contains a window
    pub fn has_id(&self, target: Id) -> bool {
        match self {
            Self::Window { id, .. } => *id == target,
            Self::EmptyOutput(_) => true,
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                for element in elements {
                    if element.has_id(target) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Try to find a window with a id
    pub fn find_window(&self, target: Id) -> Option<&Self> {
        match self {
            Self::EmptyOutput(_) => None,
            Self::Window { id, .. } => (*id == target).then_some(self),
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                for element in elements {
                    let try_find = element.find_window(target);
                    if try_find.is_some() {
                        return try_find;
                    }
                }
                None
            }
        }
    }
    fn find_window_mut(&mut self, target: Id) -> Option<&mut Self> {
        match self {
            Self::EmptyOutput(_) => None,
            Self::Window { id, .. } => (*id == target).then_some(self),
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                for element in elements {
                    let try_find = element.find_window_mut(target);
                    if try_find.is_some() {
                        return try_find;
                    }
                }
                None
            }
        }
    }

    // NOTE: not just find it, but return the insert position
    fn find_duo_windows_mut(
        &mut self,
        target_one: Id,
        target_two: Id,
    ) -> (Option<&mut Self>, Option<&mut Self>) {
        match self {
            Self::EmptyOutput(_) => (None, None),
            Self::Window { id, .. } => {
                if *id == target_one {
                    return (Some(self), None);
                }
                if *id == target_two {
                    return (None, Some(self));
                }
                (None, None)
            }
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                let mut find_one = None;
                let mut find_two = None;
                for element in elements {
                    if find_two.is_some() && find_two.is_some() {
                        break;
                    }
                    if find_one.is_none() && find_two.is_none() {
                        let (new_one, new_two) =
                            element.find_duo_windows_mut(target_one, target_two);
                        find_one = new_one;
                        find_two = new_two;
                        continue;
                    }
                    if find_one.is_none() && find_two.is_some() {
                        find_one = element.find_window_mut(target_one);
                        continue;
                    }
                    find_two = element.find_window_mut(target_two);
                }
                (find_one, find_two)
            }
        }
    }

    /// Swap two elements
    pub fn swap<F>(&mut self, id: Id, target: Id, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let (Some(element_one), Some(element_two)) = self.find_duo_windows_mut(id, target) else {
            return Err(Error::ElementNotFound);
        };
        // == swap size_pos ==
        let size_pos_one = element_one.size_pos();
        let size_pos_two = element_two.size_pos();
        element_one.set_size_and_pos(size_pos_two);
        element_two.set_size_and_pos(size_pos_one);
        // == swap percent ==
        let percent_one = element_one.percent();
        let percent_two = element_two.percent();
        element_one.set_percentage(percent_two);
        element_two.set_percentage(percent_one);
        f.callback(id, element_one.size_pos());
        f.callback(target, element_two.size_pos());
        Ok(())
    }

    /// Remap, when the container or the display changed, invoke this function
    pub fn remap<F>(&mut self, c_size_pos: SizeAndPos<T>, f: &mut F)
    where
        F: DispatchCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(size_pos) => *size_pos = c_size_pos,
            Self::Window { id, size_pos, .. } => {
                *size_pos = c_size_pos;
                f.callback(*id, *size_pos);
            }
            Self::Vertical {
                elements, size_pos, ..
            }
            | Self::Horizontal {
                elements, size_pos, ..
            } => {
                *size_pos = c_size_pos;
                let mut current_x = size_pos.position.x;
                let mut current_y = size_pos.position.y;
                let con_width = size_pos.size.width;
                let con_height = size_pos.size.height;
                for element in elements {
                    let Percentage {
                        width: w_p,
                        height: h_p,
                    } = element.percent();
                    let new_width = con_width.mul_f32(w_p);
                    let new_height = con_height.mul_f32(h_p);
                    let c_size_pos = SizeAndPos {
                        size: Size {
                            width: new_width,
                            height: new_height,
                        },
                        position: Position {
                            x: current_x,
                            y: current_y,
                        },
                    };
                    element.remap(c_size_pos, f);
                    match fit_way {
                        InsertWay::Vertical => current_y += new_height,
                        InsertWay::Horizontal => current_x += new_width,
                    }
                }
            }
        }
    }

    // I will do it tomorror
    // This function is used to check if the windows is on the right edge.
    // bool means contains the target,
    fn edge_check(&self, direction: Direction, target: Id) -> Option<bool> {
        match self {
            Self::EmptyOutput(_) => None,
            Self::Window { id, .. } => (*id == target).then_some(true),
            Self::Vertical { elements, .. } => {
                // NOTE: if the direction is fit current container, then it should be the first or
                // last one
                // else, just check if the child fit the drag check
                match direction {
                    Direction::Left | Direction::Right => {
                        for element in elements {
                            match element.edge_check(direction, target) {
                                Some(val) => {
                                    return Some(val);
                                }
                                None => {
                                    continue;
                                }
                            }
                        }
                        None
                    }
                    Direction::Top | Direction::Bottom => {
                        let mut check_index = 0;
                        if direction == Direction::Bottom {
                            let len = elements.len();
                            check_index = len - 1;
                        }
                        elements[check_index].edge_check(direction, target)
                    }
                }
            }
            Self::Horizontal { elements, .. } => {
                // NOTE: if the direction is fit current container, then it should be the first or
                // last one
                // else, just check if the child fit the drag check
                match direction {
                    Direction::Top | Direction::Bottom => {
                        for element in elements {
                            match element.edge_check(direction, target) {
                                Some(val) => {
                                    return Some(val);
                                }
                                None => {
                                    continue;
                                }
                            }
                        }
                        None
                    }
                    Direction::Left | Direction::Right => {
                        let mut check_index = 0;
                        if direction == Direction::Right {
                            let len = elements.len();
                            check_index = len - 1;
                        }
                        elements[check_index].edge_check(direction, target)
                    }
                }
            }
        }
    }

    fn drag_neighbors(
        &mut self,
        direction: Direction,
        target: Id,
    ) -> Option<(&mut Element<T>, &mut Element<T>)> {
        match self {
            // NOTE: output and window only contains zero or one window, so it cannot return two
            // elements
            Self::EmptyOutput(_) | Self::Window { .. } => None,
            Self::Vertical { elements, .. } => {
                // NOTE: if the direction is what it wants, then try to find it
                // else, it enter inside
                match direction {
                    Direction::Left | Direction::Right => {
                        for element in elements {
                            let try_find = element.drag_neighbors(direction, target);
                            if try_find.is_some() {
                                return try_find;
                            }
                        }
                        None
                    }
                    Direction::Top | Direction::Bottom => {
                        // NOTE:  Then we need to find the important logic
                        // If the windows is in the list, we need to return its neighbors
                        // But when it is in a container? how to do?
                        // This time, we need to check if component is in the top or bottom of the
                        // container. So, good, another function
                        //
                        // Is it right? even the direction is well, we still need downgrade to
                        // search all
                        //
                        // NOTE: When the drag_edge_check is false, means that place contains the
                        // element, but the place maybe inside it. so we can not for loop all elements
                        // just loop that one
                        let mut position = None;
                        let mut deep_position = None;
                        for (index, element) in elements.iter().enumerate() {
                            match element.edge_check(direction, target) {
                                Some(true) => {
                                    position = Some(index);
                                    break;
                                }
                                Some(false) => {
                                    deep_position = Some(index);
                                    break;
                                }
                                None => {
                                    continue;
                                }
                            }
                        }
                        let position = match (position, deep_position) {
                            (Some(index), None) => index,
                            (None, Some(try_position)) => {
                                return elements[try_position].drag_neighbors(direction, target);
                            }
                            (None, None) => {
                                return None;
                            }
                            _ => unreachable!(),
                        };

                        let len = elements.len();
                        if direction == Direction::Top {
                            if position == 0 {
                                return None;
                            }
                            let (slice_a, slice_b) = elements.split_at_mut(position);
                            // [......position -1, position,...] => [.....position -1], [position,...]
                            Some((&mut slice_a[position - 1], &mut slice_b[0]))
                        } else {
                            if position == len - 1 {
                                return None;
                            }
                            // [...., position,position+1,...] => [.....position], [position+1,...]
                            let (slice_a, slice_b) = elements.split_at_mut(position + 1);
                            Some((&mut slice_a[position], &mut slice_b[0]))
                        }
                    }
                }
            }
            Self::Horizontal { elements, .. } => {
                // NOTE: if the direction is what it wants, then try to find it
                // else, it enter inside
                match direction {
                    Direction::Top | Direction::Bottom => {
                        for element in elements {
                            let try_find = element.drag_neighbors(direction, target);
                            if try_find.is_some() {
                                return try_find;
                            }
                        }
                        None
                    }
                    Direction::Left | Direction::Right => {
                        // NOTE:  Then we need to find the important logic
                        // If the windows is in the list, we need to return its neighbors
                        // But when it is in a container? how to do?
                        // This time, we need to check if component is in the top or bottom of the
                        // container. So, good, another function
                        //
                        // NOTE: When the drag_edge_check is false, means that place contains the
                        // element, but the place maybe inside it. so we can not for loop all elements
                        // just loop that one
                        let mut position = None;
                        let mut deep_position = None;
                        for (index, element) in elements.iter().enumerate() {
                            match element.edge_check(direction, target) {
                                Some(true) => {
                                    position = Some(index);
                                    break;
                                }
                                Some(false) => {
                                    deep_position = Some(index);
                                    break;
                                }
                                None => {
                                    continue;
                                }
                            }
                        }
                        let position = match (position, deep_position) {
                            (Some(index), None) => index,
                            (None, Some(try_position)) => {
                                return elements[try_position].drag_neighbors(direction, target);
                            }
                            (None, None) => {
                                return None;
                            }
                            _ => unreachable!(),
                        };

                        let len = elements.len();
                        if direction == Direction::Left {
                            if position == 0 {
                                return None;
                            }
                            let (slice_a, slice_b) = elements.split_at_mut(position);
                            // [......position -1, position,...] => [.....position -1], [position,...]
                            Some((&mut slice_a[position - 1], &mut slice_b[0]))
                        } else {
                            // NOTE: if it is in the end, then of course, we cannot find a
                            // position+1
                            if position == len - 1 {
                                return None;
                            }
                            // [...., position,position+1,...] => [.....position], [position+1,...]
                            let (slice_a, slice_b) = elements.split_at_mut(position + 1);
                            Some((&mut slice_a[position], &mut slice_b[0]))
                        }
                    }
                }
            }
        }
    }

    pub fn drag_resize<F>(
        &mut self,
        transfer: T,
        direction: Direction,
        target: Id,
        f: &mut F,
    ) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        // NOTE: First we need to find the two neighhor with the direction and target
        // Then apply the change to them
        // I got wrong here. We need to use the direction to decided who is first, who is next
        let Some((element_a, element_b)) = self.drag_neighbors(direction, target) else {
            // Here means we did not find the element
            return Err(Error::ElementNotFound);
        };
        let (change_one, change_two) = match direction {
            Direction::Left | Direction::Top => (
                SizeAndPos::drag_change(transfer, direction.opposite()),
                SizeAndPos::drag_change(transfer, direction),
            ),
            Direction::Right | Direction::Bottom => (
                SizeAndPos::drag_change(transfer, direction),
                SizeAndPos::drag_change(transfer, direction.opposite()),
            ),
        };
        // NOTE: this means the drag is illegal, we need to stop it
        if !(element_a.size() + change_one.size).size_legal()
            || !(element_b.size() + change_two.size).size_legal()
        {
            // TODO: we need add a new element for this error
            return Err(Error::DragIllegal);
        }

        let pos_size_a = element_a.size_pos() + change_one;
        element_a.remap(pos_size_a, f);
        let pos_size_b = element_b.size_pos() + change_two;
        element_b.remap(pos_size_b, f);

        // NOTE: then we need to update the new percent, with the diff change
        let percent_a = element_a.percent();
        let size_a = element_a.size();
        let percent_b = element_b.percent();
        let size_b = element_b.size();
        // NOTE: remap won't change the percent of the element, so
        match direction {
            Direction::Top | Direction::Bottom => {
                // NOTE: up and down
                // Here we can make sure the percent of width is 1.0
                let total_h_percent = percent_a.height + percent_b.height;
                let h_a = size_a.height;
                let h_b = size_b.height;
                let h_total = h_a + h_b;
                let relative_percent_ha = h_a.to_f32() / h_total.to_f32();
                let relative_percent_hb = h_b.to_f32() / h_total.to_f32();
                let percent_ha = relative_percent_ha * total_h_percent;
                let percent_hb = relative_percent_hb * total_h_percent;
                element_a.set_percentage(Size {
                    width: 1.,
                    height: percent_ha,
                });
                element_b.set_percentage(Size {
                    width: 1.,
                    height: percent_hb,
                });
            }
            Direction::Left | Direction::Right => {
                // NOTE: left and right
                // Here we can make sure the percent of height is 1.0
                let total_w_percent = percent_a.width + percent_b.width;
                let w_a = size_a.width;
                let w_b = size_b.width;
                let w_total = w_a + w_b;
                let relative_percent_wa = w_a.to_f32() / w_total.to_f32();
                let relative_percent_wb = w_b.to_f32() / w_total.to_f32();
                let percent_wa = relative_percent_wa * total_w_percent;
                let percent_wb = relative_percent_wb * total_w_percent;
                element_a.set_percentage(Size {
                    width: percent_wa,
                    height: 1.,
                });
                element_b.set_percentage(Size {
                    width: percent_wb,
                    height: 1.,
                });
            }
        }
        Ok(())
    }
    /// It is used to delete a window from current container
    pub fn delete<F>(&mut self, target: Id, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(_) => Err(Error::ElementNotFound),
            // NOTE: this logic only comes when there is only one window exist
            Self::Window { id, size_pos, .. } => {
                if *id != target {
                    return Err(Error::ElementNotFound);
                }
                *self = Self::EmptyOutput(*size_pos);
                Ok(())
            }
            Self::Vertical {
                elements,
                percent,
                size_pos,
            }
            | Self::Horizontal {
                elements,
                percent,
                size_pos,
            } => {
                let mut position: Option<usize> = None;
                let mut window_s_a_p: Option<SizeAndPos<T>> = None;
                // We use the percent to record how much the size is expand
                // if it is +, means it expand
                // else it shrink
                let mut target_percent: Option<Size> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let Self::Window {
                        id,
                        size_pos,
                        percent,
                    } = element
                        && *id == target
                    {
                        position = Some(index);
                        window_s_a_p = Some(*size_pos);
                        target_percent = Some(*percent);
                        break;
                    }
                    if element.delete(target, f).is_ok() {
                        return Ok(());
                    }
                }
                let (Some(pos), Some(disappear_info), Some(target_percent)) =
                    (position, window_s_a_p, target_percent)
                else {
                    return Err(Error::ElementNotFound);
                };
                elements.remove(pos);

                let start = pos == 0;
                let adjust_pos = if start { 0 } else { pos - 1 };
                let expand_way = Direction::expend_way(fit_way, start);

                let element = &mut elements[adjust_pos];

                // use the direction to calculate the diff change
                let changed_size_and_pos = disappear_info.change_disappear(expand_way);
                element.expand(
                    changed_size_and_pos,
                    target_percent.change_expand(fit_way),
                    f,
                );
                // NOTE: now we need a new function to expand the element.
                // And we need a enum contains four fields
                let _ = element;
                // Then we remove the deleted guy

                // it the element only one existed, downgrade it
                if elements.len() == 1 {
                    let o_percent = *percent;
                    let o_size_pos = *size_pos;
                    // first, we clone all the information in the element[0]
                    *self = elements[0].clone();
                    // Since it means it replace all of the information, then the size and position
                    // of container won't change
                    // So we just give these to it
                    self.set_percentage(o_percent);
                    self.set_size_and_pos(o_size_pos);
                }

                Ok(())
            }
        }
    }

    /// Drag a window from map or other place and drop it
    pub fn drag_and_drop<F>(
        &mut self,
        id: Id,
        target: Id,
        direction: Direction,
        f: &mut F,
    ) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let _ = self.delete(id, f);
        self.insert(id, target, direction, f)
    }

    /// This is insert in for directions
    pub fn insert<F>(&mut self, id: Id, target: Id, direction: Direction, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        let fit_way = match self {
            Self::Vertical { .. } => InsertWay::Vertical,
            // NOTE: it will only be used in pattern three, so,
            // this part will always be Self::Horizontal
            _ => InsertWay::Horizontal,
        };
        match self {
            Self::EmptyOutput(size) => {
                f.callback(id, *size);
                *self = Self::Window {
                    id,
                    size_pos: *size,
                    percent: Size {
                        width: 1.,
                        height: 1.,
                    },
                };
                Ok(())
            }
            Self::Window {
                size_pos,
                id: o_id,
                percent,
            } => {
                if *o_id != target {
                    return Err(Error::ElementNotFound);
                }
                let origin_size_pos = *size_pos;
                let new_size_pos = size_pos.split(direction);
                let old_percent = *percent;
                f.callback(*o_id, *size_pos);
                f.callback(id, new_size_pos);
                // NOTE: because it is will not become a window anymore, it will be the half of the
                // Vertical or Horizontal
                let new_percent = Size::whole().split(2., direction);
                *percent = new_percent;
                let elements = if direction.is_end() {
                    vec![
                        self.clone(),
                        Element::Window {
                            id,
                            size_pos: new_size_pos,
                            percent: new_percent,
                        },
                    ]
                } else {
                    vec![
                        Element::Window {
                            id,
                            size_pos: new_size_pos,
                            percent: new_percent,
                        },
                        self.clone(),
                    ]
                };
                *self = match direction {
                    Direction::Bottom | Direction::Top => Element::Vertical {
                        elements,
                        size_pos: origin_size_pos,
                        percent: old_percent,
                    },
                    Direction::Left | Direction::Right => Element::Horizontal {
                        elements,
                        size_pos: origin_size_pos,
                        percent: old_percent,
                    },
                };
                Ok(())
            }
            Self::Vertical { elements, .. } | Self::Horizontal { elements, .. } => {
                let mut to_insert_index: Option<usize> = None;
                let mut to_return: Option<SizeAndPos<T>> = None;
                let mut new_percent: Option<Size> = None;
                for (index, element) in elements.iter_mut().enumerate() {
                    if let Element::Window {
                        id: o_id,
                        size_pos,
                        percent,
                    } = element
                        && *o_id == target
                    {
                        if fit_way.fit_direction(direction) {
                            let new_size_pos = size_pos.split(direction);
                            *percent = percent.split(2., direction);
                            to_return = Some(new_size_pos);
                            to_insert_index = Some(index);
                            new_percent = Some(*percent);
                            f.callback(*o_id, *size_pos);
                            break;
                        }
                        return element.insert(id, target, direction, f);
                    }
                    let insert_result = element.insert(id, target, direction, f);
                    if insert_result.is_ok() {
                        return Ok(());
                    }
                }
                if let (Some(index), Some(to_return), Some(percent)) =
                    (to_insert_index, to_return, new_percent)
                {
                    let size_pos = to_return;
                    let window = Self::Window {
                        id,
                        size_pos,
                        percent,
                    };
                    let index = if direction.is_end() { index + 1 } else { index };
                    f.callback(id, size_pos);
                    elements.insert(index, window);
                    return Ok(());
                }
                Err(Error::ElementNotFound)
            }
        }
    }

    /// The return shows the new inserted position. it should be saved. but you can know it during
    /// the result show if the operation is succeeded
    pub fn insert_new<F>(&mut self, id: Id, target: Id, way: InsertWay, f: &mut F) -> Result<()>
    where
        F: DispatchCallback<T>,
    {
        self.insert(id, target, way.into(), f)
    }
}
