use crate::image::{bitmask::ImageBitmask, ImageLikeUnchecked, Pixel};
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
        let relativity = toolbar.get_fill_relativity();
        // only reference the selected pixel if tolerance is absolute (not relative)
        let tolerance_reference = Some(canvas.active_image().pix_at(or, oc)).filter(|_| !relativity);

        let bitmask = ImageBitmask::from_flood_fill(canvas.active_image(), tolerance, or, oc, tolerance_reference);
        let image = canvas.active_image_mut();
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
