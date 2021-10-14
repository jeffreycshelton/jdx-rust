mod bindings;

pub mod jdx {
    use std::{ptr, result, slice};
    use crate::bindings;

    pub type Label = bindings::JDXLabel;
    pub type Header = bindings::JDXHeader;
    pub type Version = bindings::JDXVersion;

    impl Header {
        pub fn read_from_path<S: Into<String>>(path: S) -> Result<Self> {
            let path_string = path.into();
            let mut header = Header::default(); // Initialization done only to appease the borrow checker

            let libjdx_error = unsafe {
                bindings::JDX_ReadHeaderFromPath(&mut header, path_string.as_ptr())
            };

            match libjdx_error {
                bindings::JDXError::None => Ok(header),
                bindings::JDXError::OpenFile => Err(Error::OpenFile(path_string)),
                bindings::JDXError::ReadFile => Err(Error::ReadFile(path_string)),
                bindings::JDXError::CorruptFile => Err(Error::CorruptFile(path_string)),
                bindings::JDXError::CloseFile => Err(Error::CloseFile(path_string)),
                _ => Err(Error::ReadFile(path_string))
            }
        }
    }

    impl Version {
        pub fn current() -> Self {
            unsafe { bindings::JDX_VERSION }
        }
    }

    #[derive(Clone)]
    pub struct Item {
        pub data: Vec<u8>,

        pub width: u16,
        pub height: u16,
        pub bit_depth: u8,

        pub label: Label,
    }

    impl From<&bindings::JDXItem> for Item {
        fn from(libjdx_item: &bindings::JDXItem) -> Self {
            let image_size =
                libjdx_item.width as usize *
                libjdx_item.height as usize *
                (libjdx_item.bit_depth / 8) as usize;
            
            let image_data = unsafe {
                slice::from_raw_parts(libjdx_item.data, image_size).to_vec()
            };

            Item {
                data: image_data,
                width: libjdx_item.width,
                height: libjdx_item.height,
                bit_depth: libjdx_item.bit_depth,
                label: libjdx_item.label,
            }
        }
    }

    #[derive(Clone, Default)]
    pub struct Dataset {
        pub header: Header,
        pub items: Vec<Item>,
    }

    impl From<bindings::JDXDataset> for Dataset {
        fn from(libjdx_dataset: bindings::JDXDataset) -> Self {
            let items = unsafe {
                slice::from_raw_parts(libjdx_dataset.items, libjdx_dataset.header.item_count as usize)
                    .iter()
                    .map(|libjdx_item| libjdx_item.into())
                    .collect()
            };

            Dataset {
                header: libjdx_dataset.header,
                items: items,
            }
        }
    }

    impl Dataset {
        pub fn read_from_path<S: Into<String>>(path: S) -> Result<Self> {
            let path_string = path.into();
            let mut libjdx_dataset = bindings::JDXDataset { // Initialization done only to appease the borrow checker
                header: Header::default(),
                items: ptr::null_mut(),
            };

            let libjdx_error = unsafe {
                bindings::JDX_ReadDatasetFromPath(&mut libjdx_dataset, path_string.as_ptr())
            };

            let result = match libjdx_error {
                bindings::JDXError::None => Ok(libjdx_dataset.into()),
                bindings::JDXError::OpenFile => Err(Error::OpenFile(path_string)),
                bindings::JDXError::ReadFile => Err(Error::ReadFile(path_string)),
                bindings::JDXError::CorruptFile => Err(Error::CorruptFile(path_string)),
                bindings::JDXError::CloseFile => Err(Error::CloseFile(path_string)),
                _ => Err(Error::ReadFile(path_string))
            };

            unsafe {
                bindings::JDX_FreeDataset(libjdx_dataset);
            }

            return result;
        }

        pub fn write_to_path<S: Into<String>>(&self, path: S) -> Result<()> {
            let path_string = path.into();

            let libjdx_error = unsafe {
                bindings::JDX_WriteDatasetToPath(self.into(), path_string.as_ptr())
            };

            match libjdx_error {
                bindings::JDXError::None => Ok(()),
                bindings::JDXError::OpenFile => Err(Error::OpenFile(path_string)),
                bindings::JDXError::WriteFile => Err(Error::WriteFile(path_string)),
                bindings::JDXError::CloseFile => Err(Error::CloseFile(path_string)),
                _ => Err(Error::WriteFile(path_string))
            }
        }

        pub fn append(&mut self, dataset: &Dataset) -> Result<()> {
            if self.header.image_width != dataset.header.image_width {
                return Err(Error::UnequalWidths)
            } else if self.header.image_height != dataset.header.image_height {
                return Err(Error::UnequalHeights)
            } else if self.header.bit_depth != dataset.header.bit_depth {
                return Err(Error::UnequalBitDepths)
            }

            self.items.append(&mut dataset.items.clone());

            self.header.item_count += dataset.header.item_count;
            Ok(())
        }
    }

    #[derive(Clone)]
    pub enum Error {
        OpenFile(String),
        CloseFile(String),
        ReadFile(String),
        WriteFile(String),
        CorruptFile(String),

        UnequalWidths,
        UnequalHeights,
        UnequalBitDepths,
    }

    pub type Result<T> = result::Result<T, Error>;
}
