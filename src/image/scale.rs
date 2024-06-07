use super::undo::action::{DoableAction, StaticDoableAction, ActionName};
use super::{Image, ImageLike, Pixel, UnifiedImage};

#[derive(Clone)]
pub enum ScaleMethod {
    NearestNeighbor,
    Bilinear,
}

#[derive(Clone)]
pub struct Scale {
    method: ScaleMethod,
    w: usize,
    h: usize,
}

impl StaticDoableAction for Scale {
    fn dyn_clone(&self) -> Box<dyn DoableAction> {
        Box::new(self.clone())
    }
}

impl DoableAction for Scale {
    fn name(&self) -> ActionName {
        ActionName::Scale
    }

    fn exec(&self, image: &mut UnifiedImage) {
        match self.method {
            ScaleMethod::NearestNeighbor => todo!(),
            ScaleMethod::Bilinear => todo!(),
        }
    }
}
