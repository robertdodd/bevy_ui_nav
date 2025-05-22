// use bevy::{platform::collections::HashMap, prelude::*};

// use crate::{
//     focus_node::{FocusNode, FocusTarget},
//     prelude::*,
//     utils::f32_equal,
// };

// #[derive(Debug)]
// pub enum UiSpatialMapEvent {
//     Press(Entity),
//     Release(Entity),
//     Click(Entity),
// }

// #[derive(Debug)]
// pub struct UiSpatialMap {
//     menus: HashMap<Entity, NavMenu>,
//     focusables: HashMap<Entity, FocusNode>,
//     // mouse_only_focusables: HashMap<Entity, FocusNode>,
//     current_menu: Option<Entity>,

//     // current focusable
//     current_focusable: Option<Entity>,
//     is_current_pressed: bool,
//     current_interaction_type: Option<UiNavInteractionType>,

//     // // current mouse-only focusable
//     // current_mouse_focusable: Option<Entity>,
//     _events: Vec<UiSpatialMapEvent>,

//     _original_focusable: Option<Entity>,
//     _original_menu: Option<Entity>,

//     // locked
//     locked: bool,
//     _original_locked: bool,
// }

// impl UiSpatialMap {
//     pub fn new(
//         menu_query: &Query<(Entity, &NavMenu, &Focusables)>,
//         query: &Query<(
//             Entity,
//             &Focusable,
//             &FocusableOf,
//             &ComputedNode,
//             &GlobalTransform,
//             &InheritedVisibility,
//         )>,
//         nav_state: &UiNavState,
//     ) -> Self {
//         // Collect the normal focusables that are not disabled
//         let mut current_focusable = None;
//         let mut is_current_pressed = false;
//         let mut focusables = HashMap::<Entity, FocusNode>::new();
//         query
//             .iter()
//             // Ignore disables nodes, hidden nodes or nodes with 0 size along one dimension, unless the focusable has
//             // priority.
//             // NOTE: We must ignore the size/visibility of focusables with priority because the size and visibility
//             //  won't be computed until after the `PostUpdate` set.
//             .filter(|(_, focusable, _, node, _, visibility)| {
//                 !focusable.is_disabled
//                     && (focusable.is_priority || is_node_visible(visibility.get(), node.size()))
//             })
//             .for_each(
//                 |(entity, focusable, focusable_of, node, global_transform, _)| {
//                     let focus_node = FocusNode {
//                         menu: focusable_of.0,
//                         size: node.size(),
//                         position: global_transform.compute_transform().translation.truncate(),
//                         is_priority: focusable.is_priority,
//                     };
//                     focusables.insert(entity, focus_node);
//                     if focusable.active() {
//                         current_focusable = Some(entity);
//                         is_current_pressed = focusable.is_pressed();
//                     }
//                 },
//             );

//         let menus: HashMap<Entity, NavMenu> = menu_query
//             .iter()
//             .map(|(entity, nav_menu)| (entity, nav_menu.clone()))
//             .collect();

//         let mut ui_spatial_map = Self {
//             menus,
//             focusables,
//             // mouse_only_focusables,
//             current_focusable,
//             current_interaction_type: None,
//             is_current_pressed,
//             current_menu: nav_state.menu,
//             _original_focusable: current_focusable,
//             _original_menu: nav_state.menu,
//             _events: vec![],
//             locked: nav_state.locked,
//             _original_locked: nav_state.locked,
//         };

//         // If there is no current menu then attempt to set the next menu
//         if !ui_spatial_map.locked && ui_spatial_map.current_menu.is_none() {
//             if let Some((menu_entity, _)) = ui_spatial_map
//                 .menus
//                 .iter()
//                 .filter(|(_, menu)| !menu.is_locked)
//                 .reduce(|acc, e| {
//                     if e.1.is_priority && !acc.1.is_priority {
//                         e
//                     } else {
//                         acc
//                     }
//                 })
//             {
//                 ui_spatial_map.set_focus_to_menu(Some(*menu_entity));
//             }
//         }

//         // If there is no current focusable then attempt to find the next focusable
//         if !ui_spatial_map.locked
//             && ui_spatial_map.current_menu.is_some()
//             && ui_spatial_map.current_focusable.is_none()
//         {
//             ui_spatial_map.focus_on_node_in_current_menu();
//         }

//         ui_spatial_map
//     }

//     pub fn events(&mut self) -> &Vec<UiSpatialMapEvent> {
//         &self._events
//     }

//     /// Set focus to a new entity.
//     pub fn set_focus(&mut self, entity: Entity, interaction_type: UiNavInteractionType) {
//         if let (Some(new_menu), false) = (
//             self.focusables
//                 .get(&entity)
//                 .map(|focus_node| focus_node.menu),
//             Some(entity) == self.current_focusable,
//         ) {
//             // if `entity` is a focusable, then set focus to the entity, and change the current menu if necessary
//             self.set_focus_to_focusable(Some(entity), interaction_type);
//             if Some(new_menu) != self.current_menu {
//                 self.set_focus_to_menu(Some(new_menu));
//             }
//         } else if self.menus.contains_key(&entity) && Some(entity) != self.current_menu {
//             // if `entity` is a menu, then change the current menu and try focus on the first focusable in that menu
//             self.set_focus_to_menu(Some(entity));
//             self.focus_on_node_in_current_menu();
//         }
//     }

//     fn set_focus_to_focusable(
//         &mut self,
//         new_focusable: Option<Entity>,
//         interaction_type: UiNavInteractionType,
//     ) {
//         self.cancel_press();
//         self.current_focusable = new_focusable;
//         self.is_current_pressed = false;
//         self.current_interaction_type = Some(interaction_type);
//     }

//     fn set_focus_to_menu(&mut self, menu: Option<Entity>) {
//         self.cancel_press();
//         self.current_menu = menu;
//     }

//     fn focus_on_node_in_current_menu(&mut self) {
//         // change focusable to another in this menu
//         let priority_focusable = self
//             .focusables
//             .iter()
//             // ignore focusables outside the current menu and non-prioritized focusables
//             .find(|(_, focus_node)| {
//                 Some(focus_node.menu) == self.current_menu && focus_node.is_priority
//             })
//             .map(|(entity, _)| *entity);

//         // use the priority focusable if found, otherwise the first focusable in the menu
//         let focusable = priority_focusable.or_else(|| {
//             self.focusables
//                 .iter()
//                 // ignore focusables outside the current menu
//                 .find(|(_, focus_node)| Some(focus_node.menu) == self.current_menu)
//                 .map(|(entity, _)| *entity)
//         });

//         self.set_focus_to_focusable(focusable, UiNavInteractionType::Auto);
//     }

//     fn can_move(&self) -> bool {
//         !self.locked && self.current_focusable.is_some() && !self.is_current_pressed
//     }

//     pub fn get_new_menu(&self) -> Option<Option<Entity>> {
//         if self.current_menu != self._original_menu {
//             Some(self.current_menu)
//         } else {
//             None
//         }
//     }

//     pub fn menu(&self) -> Option<Entity> {
//         self.current_menu
//     }

//     pub fn get_new_focusable(&self) -> Option<(Option<Entity>, UiNavInteractionType)> {
//         if self.current_focusable != self._original_focusable {
//             Some((
//                 self.current_focusable,
//                 self.current_interaction_type
//                     .unwrap_or(UiNavInteractionType::Auto),
//             ))
//         } else {
//             None
//         }
//     }

//     pub fn get_new_locked(&self) -> Option<bool> {
//         if self.locked != self._original_locked {
//             Some(self.locked)
//         } else {
//             None
//         }
//     }

//     pub fn press(&mut self) -> Option<Entity> {
//         if self.locked {
//             return None;
//         }
//         // ignore if we are currently pressing a button, or there is no current focusable
//         if let (false, Some(current_focusable)) = (self.is_current_pressed, self.current_focusable)
//         {
//             self.is_current_pressed = true;
//             self._events
//                 .push(UiSpatialMapEvent::Press(current_focusable));
//             self.current_focusable
//         } else {
//             None
//         }
//     }

//     pub fn release(&mut self) -> Option<Entity> {
//         if self.locked {
//             return None;
//         }
//         // ignore if we are currently pressing a button, or there is no current focusable
//         if let (true, Some(current_focusable)) = (self.is_current_pressed, self.current_focusable) {
//             self.is_current_pressed = false;
//             self._events
//                 .push(UiSpatialMapEvent::Release(current_focusable));
//             self._events
//                 .push(UiSpatialMapEvent::Click(current_focusable));
//             self.current_focusable
//         } else {
//             None
//         }
//     }

//     /// Cancel a pressed button without emitting a click event.
//     pub fn cancel_press(&mut self) {
//         // ignore if we are currently pressing a button, or there is no current focusable
//         if let (true, Some(current_focusable)) = (self.is_current_pressed, self.current_focusable) {
//             self._events
//                 .push(UiSpatialMapEvent::Release(current_focusable));
//             self.is_current_pressed = false;
//         }
//     }

//     /// Lock UI navigation
//     pub fn lock(&mut self) {
//         self.locked = true;
//     }

//     /// Unlock UI navigation
//     pub fn unlock(&mut self) {
//         self.locked = false;
//     }

//     pub fn apply_movement(&mut self, direction: UiNavDirection) {
//         if !self.can_move() {
//             return;
//         }

//         let current_entity = self.current_focusable.unwrap();
//         let current = self.focusables.get(&current_entity).unwrap();

//         let is_current_menu_wrap = self
//             .current_menu
//             .and_then(|menu_entity| self.menus.get(&menu_entity))
//             .is_some_and(|menu| menu.is_wrap);

//         // find the nearest, and furthest nodes in the direction of travel
//         let (nearest, furthest) = self
//             .focusables
//             .iter()
//             // ignore the current focusable and any focusables outside the current menu
//             .filter(|(entity, focus_node)| {
//                 **entity != current_entity && Some(focus_node.menu) == self.current_menu
//             })
//             // map to a `FocusTarget` type
//             .map(|(entity, focus_node)| {
//                 let distance = current.distance_to(focus_node);
//                 FocusTarget {
//                     entity: *entity,
//                     position: focus_node.position,
//                     is_in_direction: distance.is_in_direction(direction),
//                     is_in_axis: distance.is_along_axis(direction),
//                     // Only prefer movement along direct axes. It doesn't matter when moving diagonally.
//                     is_prefer: match direction {
//                         UiNavDirection::Up | UiNavDirection::Down => distance.is_overlap_y,
//                         UiNavDirection::Left | UiNavDirection::Right => distance.is_overlap_x,
//                         _ => false,
//                     },
//                     overlap: match direction {
//                         UiNavDirection::Up | UiNavDirection::Down => distance.overlap_x,
//                         UiNavDirection::Left | UiNavDirection::Right => distance.overlap_y,
//                         _ => 0.,
//                     },
//                     distance,
//                 }
//             })
//             // Remove any nodes that do not lie along the axis of the movement event. If wrapping is enabled,
//             // allow any nodes along the axis. Otherwise, only allow nodes in the direction of the movement event.
//             .filter(|focus_target| {
//                 if is_current_menu_wrap {
//                     focus_target.is_in_axis
//                 } else {
//                     focus_target.is_in_direction
//                 }
//             })
//             .fold(
//                 (None, None),
//                 #[allow(clippy::type_complexity)]
//                 |(acc_nearest, acc_furthest), e| -> (Option<FocusTarget>, Option<FocusTarget>) {
//                     let e_is_in_direction = e.is_in_direction;

//                     // Fold the nearest focus node in the direction of the movement event
//                     let nearest = if let Some(acc_nearest) = acc_nearest {
//                         // Prefer `e` if it lies in the correct direction and is closer than `acc_nearest`
//                         if e_is_in_direction
//                             && ((acc_nearest.is_prefer == e.is_prefer
//                                 && (e.overlap > 0. && acc_nearest.overlap <= 0.
//                                     || e.distance.total < acc_nearest.distance.total))
//                                 || (!acc_nearest.is_prefer && e.is_prefer))
//                         {
//                             Some(e.clone())
//                         } else {
//                             Some(acc_nearest)
//                         }
//                     } else if e_is_in_direction {
//                         // set the initial nearest node
//                         Some(e.clone())
//                     } else {
//                         None
//                     };

//                     // Fold the furthest focus node
//                     let furthest = if !is_current_menu_wrap {
//                         // skip if wrapping is disabled
//                         None
//                     } else if let Some(acc_furthest) = acc_furthest {
//                         // Prefer `e` if it is further than `acc_furthest` and does not lie in the dirction of the
//                         // movement event.
//                         if !e_is_in_direction
//                             && ((acc_furthest.is_prefer == e.is_prefer
//                                 && (e.overlap > 0. && acc_furthest.overlap <= 0.
//                                     || e.distance.total > acc_furthest.distance.total
//                                     || (f32_equal(e.distance.total, acc_furthest.distance.total)
//                                         && f32_equal(e.overlap, acc_furthest.overlap)
//                                         && e.position.x < acc_furthest.position.x)))
//                                 || (!acc_furthest.is_prefer && e.is_prefer))
//                         {
//                             Some(e.clone())
//                         } else {
//                             Some(acc_furthest)
//                         }
//                     } else if !e_is_in_direction {
//                         // set the initial furthest node if it does not lie in the direction of the movement event
//                         Some(e.clone())
//                     } else {
//                         None
//                     };

//                     (nearest, furthest)
//                 },
//             );

//         if let Some(nearest) = nearest {
//             self.current_focusable = Some(nearest.entity);
//             self.current_interaction_type = Some(UiNavInteractionType::Button);
//         } else if let (Some(furthest), true) = (furthest, is_current_menu_wrap) {
//             // No nearest, wrapping around
//             self.current_focusable = Some(furthest.entity);
//             self.current_interaction_type = Some(UiNavInteractionType::Button);
//         }
//     }
// }

// fn is_node_visible(is_visible: bool, size: Vec2) -> bool {
//     is_visible && size.x > f32::EPSILON && size.y > f32::EPSILON
// }
