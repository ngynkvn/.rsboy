pub trait Controller {
    type Output;

    fn cycle(&mut self) -> Result<Self::Output, String> {
        Err(String::from("Controller not initialized."))
    }
}

impl <F, O> Controller for F
    where F: FnMut() -> Result<O, String> {
    type Output = O;
    fn cycle(&mut self) -> Result<O, String> {
        self()
    }
}
