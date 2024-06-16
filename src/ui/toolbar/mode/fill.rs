use crate::{image::{selection::ImageBitmask, Pixel}, ui::selection::Selection};
use super::{Canvas, MouseModeVariant, Toolbar};

use gtk::gdk::ModifierType;

#[derive(Clone, Copy)]
pub struct FillState;

impl FillState {
    pub fn default(_canvas: &Canvas) -> FillState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> FillState {
        FillState
    }
}

impl super::MouseModeState for FillState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        let (ox, oy) = canvas.cursor_pos_pix();
        let (oc, or) = (ox.floor() as usize, oy.floor() as usize);
        let tolerance = toolbar.get_magic_wand_tolerance();

        let bitmask = ImageBitmask::from_flood_fill(canvas.image().image(), tolerance, or, oc);
        let image = canvas.image();
        let p = Pixel::from_rgba_struct(toolbar.primary_color());

        for (r, c) in bitmask.coords_of_active_bits().iter() {
            *image.pix_at_mut(*r as i32, *c as i32) = p.clone();
        }

        canvas.update()
    }
}
