use crate::{image::selection::bfs_for_bitmask, ui::form::Form};
use super::{Canvas, MouseModeVariant, Toolbar};

use gtk::gdk::ModifierType;

// Unit struct for now: there is no state other than the
// selection, which is stored in the canvas if it exists
// (to avoid copying the bitset)
#[derive(Clone, Copy)]
pub struct MagicWandState;

impl MagicWandState {
    pub fn default(_canvas: &Canvas) -> MagicWandState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> MagicWandState {
        MagicWandState
    }
}

impl super::MouseModeState for MagicWandState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let (ox, oy) = canvas.cursor_pos_pix();
        let (oc, or) = (ox.floor() as usize, oy.floor() as usize);
        let tolerance = toolbar.get_magic_wand_tolerance();

        let bitmask = bfs_for_bitmask(canvas.image().image(), tolerance, or, oc);
        canvas.set_selection(crate::ui::selection::Selection::Bitmask(bitmask));
    }
}
