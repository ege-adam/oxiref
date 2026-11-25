use iced::{Rectangle, widget::image::Handle};

pub struct Item {
    pub z_index: usize,
    pub handle: Handle,
    pub bounds: Rectangle,
}

impl Item {
    pub fn load_image(path: &std::path::Path) -> Result<(Handle, u32, u32), image::ImageError> {
        let img = image::open(path)?.into_rgba8();
        let (w, h) = img.dimensions();
        let img_handle = Handle::from_rgba(w, h, img.into_raw());
        Ok((img_handle, w, h))
    }
}
