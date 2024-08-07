use crate::{image::{bitmask::ImageBitmask, ImageLikeUnchecked}, ui::selection::Selection};
use super::{Canvas, Toolbar};

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
        let (oc, or) = canvas.cursor_pos_pix_u();
        let tolerance = toolbar.get_magic_wand_tolerance();
        let relativity = toolbar.get_magic_wand_relativity();
        // only reference the selected pixel if tolerance is absolute (not relative)
        let tolerance_reference = Some(canvas.active_image().pix_at(or, oc)).filter(|_| !relativity);

        let bitmask = ImageBitmask::from_flood_fill(canvas.active_image(), tolerance, or, oc, tolerance_reference);
        canvas.set_selection(Selection::Bitmask(bitmask));
        canvas.update()
    }

    fn handle_right_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, _toolbar: &mut Toolbar) {
        // deselect
        canvas.set_selection(Selection::NoSelection);
        canvas.update();
    }
}
