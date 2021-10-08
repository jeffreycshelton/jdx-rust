mod bindings;

pub mod jdx {
    use std::{io, ptr};
    use crate::bindings;

    pub type Version = bindings::JDXVersion;
    pub type ColorType = bindings::JDXColorType;
    
    #[derive(Clone)]
    pub struct Image {
        pub data: Box<[u8]>,

        pub width: i16,
        pub height: i16,
        pub color_type: ColorType,
    }

    pub type Label = i16;

    #[derive(Clone, Copy)]
    pub struct Header {
        pub version: Version,
        pub color_type: ColorType,

        pub image_width: i16,
        pub image_height: i16,
        pub item_count: usize,
    }

    #[derive(Clone)]
    pub struct Dataset {
        pub header: Header,

        pub images: Vec<Image>,
        pub labels: Vec<Label>,
    }

    impl Image {
        pub(super) fn from_c(c_image: bindings::JDXImage) -> Image {
            let image_size = c_image.width as usize * c_image.height as usize * c_image.color_type as usize;
            let boxed_data = unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(c_image.data, image_size)) };

            Image {
                data: boxed_data,
                width: c_image.width,
                height: c_image.height,
                color_type: c_image.color_type
            }
        }

        pub(super) fn to_c(&mut self) -> bindings::JDXImage {
            bindings::JDXImage {
                data: self.data.as_mut_ptr(),
                width: self.width,
                height: self.height,
                color_type: self.color_type
            }
        }
    }
}
