use crate::image::{bitmask::ImageBitmask, Pixel};
use super::{Canvas, Toolbar};
use crate::image::undo::action::ActionName;
use crate::image::TrackedLayeredImage;

use gtk::gdk::{RGBA, ModifierType};

#[derive(Clone, Copy)]
pub struct FillState;

impl FillState {
    pub fn default(_canvas: &Canvas) -> FillState {
        Self::default_no_canvas()
    }

    pub const fn default_no_canvas() -> FillState {
        FillState
    }

    fn do_fill(canvas: &mut Canvas, toolbar: &mut Toolbar, color: RGBA) {
        let (oc, or) = canvas.cursor_pos_pix_u();
        let tolerance = toolbar.get_fill_tolerance();

        let bitmask = ImageBitmask::from_flood_fill(canvas.image().image(), tolerance, or, oc);
        let image = canvas.image();
        let p = Pixel::from_rgba_struct(color);

        for (r, c) in bitmask.coords_of_active_bits() {
            *image.pix_at_mut(r as i32, c as i32) = p.clone();
        }

        canvas.commit_changes(ActionName::Fill);
        canvas.update()
    }
}

impl super::MouseModeState for FillState {
    fn handle_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        FillState::do_fill(canvas, toolbar, toolbar.primary_color());
    }

    fn handle_right_drag_start(&mut self, _mod_keys: &ModifierType, canvas: &mut Canvas, toolbar: &mut Toolbar) {
        FillState::do_fill(canvas, toolbar, toolbar.secondary_color());
    }
}
