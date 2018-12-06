// provide get() for pointers
// just to save some code
pub trait Get {
  type Result;
  fn get(self) -> &'static Self::Result;
}

pub trait GetMut {
  type Result;
  fn get(self) -> &'static mut Self::Result;
}

impl<T> Get for *const T {
  type Result = T;

  fn get(self) -> &'static T {
    unsafe { &*self }
  }
}

impl<T> GetMut for *mut T {
  type Result = T;

  fn get(self) -> &'static mut T {
    unsafe { &mut *self }
  }
}

