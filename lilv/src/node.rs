use crate::world::InnerWorld;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ffi::CStr;
use std::fmt::Debug;
use std::ptr::NonNull;
use std::sync::Arc;

extern "C" {
    // Needed for Node::get_path as it actually returns
    // things allocated in Serd.
    fn serd_free(ptr: *mut std::os::raw::c_void);
}

unsafe impl Send for Node {}
unsafe impl Sync for Node {}

pub struct Node {
    pub(crate) inner: RwLock<NonNull<lib::LilvNodeImpl>>,
    borrowed: bool,
    world: Arc<InnerWorld>,
}

impl Node {
    pub(crate) fn new(ptr: NonNull<lib::LilvNodeImpl>, world: Arc<InnerWorld>) -> Self {
        Self {
            inner: RwLock::new(ptr),
            borrowed: false,
            world,
        }
    }

    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvNodeImpl>, world: Arc<InnerWorld>) -> Self {
        Self {
            inner: RwLock::new(ptr),
            borrowed: true,
            world,
        }
    }

    /// The value within the node as an enum.
    pub fn to_value(&self) -> Value<'_> {
        if let Some(f) = self.as_f32() {
            Value::Float(f)
        } else if let Some(i) = self.as_i32() {
            Value::Int(i)
        } else if let Some(uri) = self.as_uri() {
            Value::Uri(uri)
        } else if let Some(blank) = self.as_blank() {
            Value::Blank(blank)
        } else if let Some(s) = self.as_str() {
            Value::Str(s)
        } else if let Some(b) = self.as_bool() {
            Value::Bool(b)
        } else {
            Value::Unknown
        }
    }

    /// Returns this value as a Turtle/SPARQL token.
    pub fn turtle_token(&self) -> String {
        let node = self.inner.read().as_ptr();

        unsafe {
            let original = lib::lilv_node_get_turtle_token(node);
            let rusty = CStr::from_ptr(lib::lilv_node_get_turtle_token(self.inner.read().as_ptr()))
                .to_string_lossy()
                .into_owned();
            lib::lilv_free(original as _);
            rusty
        }
    }

    /// Returns whether the value is a URI (resource).
    pub fn is_uri(&self) -> bool {
        unsafe { lib::lilv_node_is_uri(self.inner.read().as_ptr()) }
    }

    /// Returns this value as a URI string.
    pub fn as_uri(&self) -> Option<&str> {
        if self.is_uri() {
            Some(unsafe {
                CStr::from_ptr(lib::lilv_node_as_uri(self.inner.read().as_ptr()))
                    .to_str()
                    .ok()?
            })
        } else {
            None
        }
    }

    /// Returns whether the value is a blank node (resource with no URI).
    pub fn is_blank(&self) -> bool {
        unsafe { lib::lilv_node_is_blank(self.inner.read().as_ptr()) }
    }

    /// Returns this value as a blank node identifier.
    pub fn as_blank(&self) -> Option<&str> {
        if self.is_blank() {
            Some(unsafe {
                CStr::from_ptr(lib::lilv_node_as_blank(self.inner.read().as_ptr()))
                    .to_str()
                    .ok()?
            })
        } else {
            None
        }
    }

    pub fn is_literal(&self) -> bool {
        unsafe { lib::lilv_node_is_literal(self.inner.read().as_ptr()) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { lib::lilv_node_is_string(self.inner.read().as_ptr()) }
    }

    pub fn as_str(&self) -> Option<&str> {
        Some(unsafe {
            CStr::from_ptr(lib::lilv_node_as_string(self.inner.read().as_ptr()))
                .to_str()
                .ok()?
        })
    }

    pub fn get_path(&self) -> Option<(String, String)> {
        let node = self.inner.read().as_ptr();
        let mut hostname = std::ptr::null_mut();
        let path = NonNull::new(unsafe { lib::lilv_node_get_path(node, &mut hostname) })?;

        unsafe {
            let rusty_path = CStr::from_ptr(path.as_ptr()).to_string_lossy().into_owned();
            let rusty_hostname = CStr::from_ptr(hostname).to_string_lossy().into_owned();

            serd_free(path.as_ptr() as _);
            serd_free(hostname as _);

            Some((rusty_path, rusty_hostname))
        }
    }

    pub fn is_f32(&self) -> bool {
        unsafe { lib::lilv_node_is_float(self.inner.read().as_ptr()) }
    }

    pub fn as_f32(&self) -> Option<f32> {
        if self.is_f32() {
            Some(unsafe { lib::lilv_node_as_float(self.inner.read().as_ptr()) })
        } else {
            None
        }
    }

    pub fn is_i32(&self) -> bool {
        unsafe { lib::lilv_node_is_int(self.inner.read().as_ptr()) }
    }

    pub fn as_i32(&self) -> Option<i32> {
        if self.is_i32() {
            Some(unsafe { lib::lilv_node_as_int(self.inner.read().as_ptr()) })
        } else {
            None
        }
    }

    pub fn is_bool(&self) -> bool {
        unsafe { lib::lilv_node_is_bool(self.inner.read().as_ptr()) }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            Some(unsafe { lib::lilv_node_as_bool(self.inner.read().as_ptr()) })
        } else {
            None
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            inner: RwLock::new(
                NonNull::new(unsafe { lib::lilv_node_duplicate(self.inner.read().as_ptr()) })
                    .unwrap(),
            ),
            borrowed: false,
            world: self.world.clone(),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        unsafe { lib::lilv_node_equals(self.inner.read().as_ptr(), other.inner.read().as_ptr()) }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.turtle_token())
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if !self.borrowed {
            unsafe { lib::lilv_node_free(self.inner.write().as_ptr()) }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Uri(&'a str),
    Blank(&'a str),
    Str(&'a str),
    Float(f32),
    Int(i32),
    Bool(bool),
    Unknown,
}
