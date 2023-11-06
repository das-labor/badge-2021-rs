use core::cell::RefCell;
use embedded_hal::digital::v2::OutputPin;

pub struct SharedPin<P: 'static + OutputPin>(pub &'static RefCell<P>);

impl<P: OutputPin> OutputPin for SharedPin<P> {
    type Error = ();

    /// Borrows the RefCell and calls set_low() on the pin
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.borrow_mut().set_low().map_err(|_e| ())
    }

    /// Borrows the RefCell and calls set_high() on the pin
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.borrow_mut().set_high().map_err(|_e| ())
    }
}
