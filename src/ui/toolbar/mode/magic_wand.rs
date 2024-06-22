use crate::{image::bitmask::ImageBitmask, ui::selection::Selection};
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
        let (oc, or) = canvas.cursor_pos_pix_u();
        let tolerance = toolbar.get_magic_wand_tolerance();

        let bitmask = ImageBitmask::from_flood_fill(canvas.image().image(), tolerance, or, oc);
        canvas.set_selection(Selection::Bitmask(bitmask));
        canvas.update()
    }
}
